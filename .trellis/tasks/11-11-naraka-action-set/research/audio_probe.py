#!/usr/bin/env python3
"""Read-only bounded Naraka PCK / string-keyed WwiseBanks research probe."""
import hashlib
import json
from pathlib import Path
import struct

OUT = Path(__file__).resolve().parent
ROOT = Path('/home/asuka/kiva-asset-research/naraka')
AUDIO = Path('/mnt/c/Program Files (x86)/Steam/steamapps/common/NARAKA BLADEPOINT/NarakaBladepoint_Data/StreamingAssets/Audio/GeneratedSoundBanks/Windows')

def fnv(name):
    result = 2166136261
    for value in name.lower().encode():
        result = ((result * 16777619) ^ value) & 0xffffffff
    return result

class Reader:
    def __init__(self, data, at=0): self.data, self.at = data, at
    def take(self, count):
        if not 0 <= count <= len(self.data) - self.at: raise ValueError('bounds')
        value = self.data[self.at:self.at+count]; self.at += count; return value
    def u32(self): return struct.unpack('<I', self.take(4))[0]
    def string(self):
        count = self.u32()
        if count > 4096: raise ValueError('string bound')
        return self.take(count*2).decode('utf-16-le')

def tables():
    data = (ROOT/'association/WwiseBanks.dbbl').read_bytes()
    r = Reader(data)
    assert r.take(8) == b'DDBL\x01\x00\x00\x00' and r.string() == 'WwiseBanks'
    headers = [(r.u32(), r.string()) for _ in range(r.u32())]
    result = {}
    for index, (offset, name) in enumerate(headers):
        r.at = offset
        count, index_size = r.u32(), r.u32()
        index_at = r.at
        rows = [(r.string(), r.u32()) for _ in range(count)]
        assert r.at == index_at + index_size
        records = []
        for key, size in rows:
            offset = r.at
            raw = r.take(size)
            row = Reader(raw)
            assert row.string() == key
            records.append(dict(key=key, offset=offset, bytes=size, raw=raw))
        assert r.at == (headers[index+1][0] if index+1<len(headers) else len(data))
        result[name] = records
    return data, result

def pck(path):
    with path.open('rb') as stream:
        first = stream.read(28)
        magic, header_size, version, lang_size, bank_size, media_size, ext_size = struct.unpack('<4s6I', first)
        assert magic == b'AKPK' and version == 1 and header_size < 4*1024*1024
        tail = stream.read(header_size+8-28)
    data = first + tail
    at = 28 + lang_size
    result = []
    for section, size in [('banks',bank_size),('media',media_size),('external',ext_size)]:
        count = struct.unpack_from('<I', data, at)[0]
        assert size == 4+count*20
        for item in struct.iter_unpack('<5I', data[at+4:at+size]):
            ident, alignment, length, offset, language = item
            physical = offset*alignment
            assert physical+length <= path.stat().st_size
            result.append(dict(section=section,id=ident,alignment=alignment,bytes=length,offset=physical,language=language))
        at += size
    assert at == len(data)
    return dict(file=str(path),file_bytes=path.stat().st_size,header_bytes=len(data),header_sha256=hashlib.sha256(data).hexdigest(),entries=result)

def chunks(data, start=0):
    at=start
    result=[]
    while at<len(data):
        tag,size=struct.unpack_from('<4sI',data,at)
        if at+8+size>len(data): raise ValueError('chunk range')
        result.append((tag,at,data[at+8:at+8+size]));at+=8+size
    assert at==len(data)
    return result

def selected_bank(package, item):
    assert item['bytes'] <= 16*1024*1024
    with Path(package['file']).open('rb') as stream:
        stream.seek(item['offset']);data=stream.read(item['bytes'])
    result=dict(package={k:v for k,v in package.items() if k!='entries'},entry=item,sha256=hashlib.sha256(data).hexdigest(),chunks=[])
    for tag,at,body in chunks(data):
        entry=dict(tag=tag.decode('ascii'),offset=at,bytes=len(body))
        if tag==b'BKHD': entry['version'],entry['bank_id']=struct.unpack_from('<II',body)
        if tag==b'HIRC':
            r=Reader(body);count=r.u32();objects=[]
            for _ in range(count):
                offset=r.at;kind=r.take(1)[0];size=r.u32();raw=r.take(size)
                ident=struct.unpack_from('<I',raw)[0]
                objects.append(dict(offset=offset,type=kind,id=ident,bytes=size,body_hex=raw[4:].hex()))
            assert r.at==len(body)
            entry['objects']=objects
        if tag==b'DIDX': entry['media']=[dict(id=i,offset=o,bytes=n) for i,o,n in struct.iter_unpack('<III',body)]
        result['chunks'].append(entry)
    return result, data

def main():
    raw, tables_ = tables()
    banks = tables_['WwiseBanks_Bank']
    selected_names = {'gman_hengdao','gman_all_jump','gman_drop_to_land','move_jump','gman_dodge','gman_all_weapon_touch','impact_body'}
    selected=[]
    for row in banks:
        if row['key'] in selected_names:
            r=Reader(row['raw']);name=r.string();package=r.string()
            selected.append(dict(name=name,package=package,id=fnv(name),record_offset=row['offset'],record_bytes=row['bytes'],record_sha256=hashlib.sha256(row['raw']).hexdigest(),tail_hex=r.take(len(r.data)-r.at).hex()))
    events=[]
    for row in tables_['WwiseBanks_Event']:
        r=Reader(row['raw']);name=r.string();bank=r.string()
        if bank in selected_names:
            events.append(dict(name=name,bank=bank,id=fnv(name),record_offset=row['offset'],record_bytes=row['bytes'],record_sha256=hashlib.sha256(row['raw']).hexdigest(),tail_hex=r.take(len(r.data)-r.at).hex()))
    packages={name:pck(AUDIO/name) for name in sorted(set(s['package'] for s in selected))}
    extracted=[]
    for bank in selected:
        pkg=packages[bank['package']]
        entries=[e for e in pkg['entries'] if e['section']=='banks' and e['id']==bank['id']]
        assert len(entries)==1,bank
        result,data=selected_bank(pkg,entries[0]);result['name']=bank['name'];extracted.append(result)
    report=dict(dbbl_sha256=hashlib.sha256(raw).hexdigest(),tables=[dict(name=n,records=len(r)) for n,r in tables_.items()],banks=selected,events=events,selected_banks=extracted,packages=list(packages.values()))
    (OUT/'audio_probe.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps(dict(banks=[dict(name=b['name'],bytes=b['entry']['bytes'],chunks=[(c['tag'],c['bytes'],len(c.get('objects',[]))) for c in b['chunks']]) for b in extracted],events=len(events)),ensure_ascii=False,indent=2))

if __name__ == '__main__': main()
