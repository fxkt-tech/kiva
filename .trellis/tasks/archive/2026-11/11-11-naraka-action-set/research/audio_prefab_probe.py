#!/usr/bin/env python3
"""Inspect exact original Hengdao FX prefab audio components, bounded read-only."""
from pathlib import Path
import hashlib
import json
import sys

ROOT=Path('/home/asuka/kiva-asset-research/naraka')
OUT=Path(__file__).resolve().parent
SOURCE=Path('/mnt/c/Program Files (x86)/Steam/steamapps/common/NARAKA BLADEPOINT/NarakaBladepoint_Data/StreamingAssets')
sys.path.insert(0,str(ROOT/'containers'))
from naraka_sparse import SparseBundle
import UnityPy

def main():
    key='a/9/a9638811245cd210'
    with SparseBundle(SOURCE/key) as bundle:
        env=UnityPy.Environment();cab=next(n['name'] for n in bundle.nodes if n['flags']&4);sf=env.load_file(bundle.open_node(cab),name=cab)
        selected=[]
        for path,ptr in sf.container.items():
            if not (path.endswith('fxattack_male_hengdao_attack_light_01.prefab') or path.endswith('fxattack_male_hengdao_shock_04.prefab')):continue
            root=sf.objects[ptr.m_PathID];seen=set();nodes=[]
            def visit(go):
                if go.path_id in seen:raise ValueError('gameobject cycle')
                seen.add(go.path_id);gt=go.read_typetree(check_read=True);components=[]
                for c in gt['m_Component']:
                    p=c['component'];assert p['m_FileID']==0
                    obj=sf.objects[p['m_PathID']];t=obj.read_typetree(check_read=True)
                    if obj.type.name=='MonoBehaviour':
                        raw=obj.get_raw_data();script=t['m_Script'];klass=None
                        if script['m_FileID']==0:klass=sf.objects[script['m_PathID']].read_typetree(check_read=True)
                        components.append(dict(path_id=obj.path_id,raw_bytes=len(raw),raw_sha256=hashlib.sha256(raw).hexdigest(),script=klass,tree=t))
                    if obj.type.name=='Transform':
                        for child in t['m_Children']:
                            assert child['m_FileID']==0
                            ct=sf.objects[child['m_PathID']].read_typetree(check_read=True);gp=ct['m_GameObject'];assert gp['m_FileID']==0
                            visit(sf.objects[gp['m_PathID']])
                nodes.append(dict(name=gt['m_Name'],path_id=go.path_id,components=components))
            visit(root);selected.append(dict(path=path,path_id=root.path_id,nodes=nodes))
        report=dict(source_key=key,cab=cab,prefabs=selected,proof=bundle.report())
        (OUT/'audio-prefab-source.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
        for p in selected:
            print(p['path'])
            for n in p['nodes']:
                for c in n['components']:
                    print(n['name'],c['script']['m_ClassName'],str(c['tree'])[:1000])

if __name__=='__main__':main()
