#!/usr/bin/env python3
"""Validate source scalar-curve extraction; this does NOT solve Humanoid rotations."""
from pathlib import Path
import bisect, json, math, struct
ROOT=Path(__file__).resolve().parent

def run():
    p=ROOT/'male_hengdao_attack_light_01.anim.probe.json'
    d=json.loads(p.read_text());tree=d['tree'];mc=tree['m_MuscleClip'];clip=mc['m_Clip']['data']
    stream=clip['m_StreamedClip'];raw=struct.pack('<'+'I'*len(stream['data']),*stream['data']);at=0;frames=[]
    while at<len(raw):
        time,count=struct.unpack_from('<fi',raw,at);at+=8
        if not 0<=count<=stream['curveCount']:raise ValueError('stream count')
        keys=[]
        for _ in range(count):
            idx,*coeff=struct.unpack_from('<i4f',raw,at);at+=20
            if not 0<=idx<stream['curveCount'] or not all(math.isfinite(x) for x in coeff):raise ValueError('stream coefficient')
            keys.append((idx,coeff))
        frames.append((time,keys))
    if at!=len(raw):raise ValueError('stream EOF')
    dense=clip['m_DenseClip'];const=clip['m_ConstantClip']['data'];cc=dense['m_CurveCount']
    if len(dense['m_SampleArray'])!=cc*dense['m_FrameCount']:raise ValueError('dense count')
    scalar_count=stream['curveCount']+cc+len(const)
    binding_width=sum(({1:3,2:4,3:3}.get(b['attribute'],1) if b['typeID']==4 else 1) for b in tree['m_ClipBindingConstant']['genericBindings'])
    if binding_width!=scalar_count or scalar_count!=len(mc['m_ValueArrayDelta']):raise ValueError('binding scalar count')
    def sample(time):
        values=[None]*stream['curveCount']
        for ft,keys in frames:
            if ft>time:break
            for idx,coeff in keys:
                delta=max(0.,time-ft) if ft>-1e20 else 0.
                values[idx]=((coeff[0]*delta+coeff[1])*delta+coeff[2])*delta+coeff[3]
        if any(v is None for v in values):raise ValueError('unbound streamed curve')
        f=(time-dense['m_BeginTime'])*dense['m_SampleRate'];lo=min(dense['m_FrameCount']-1,max(0,math.floor(f)));hi=min(dense['m_FrameCount']-1,lo+1);mix=max(0.,min(1.,f-lo))
        values.extend(dense['m_SampleArray'][lo*cc+i]*(1-mix)+dense['m_SampleArray'][hi*cc+i]*mix for i in range(cc));values.extend(const)
        if not all(math.isfinite(v) for v in values):raise ValueError('nonfinite output')
        return values
    samples={str(t):sample(t) for t in [0.,mc['m_StopTime']/2,mc['m_StopTime']]}
    start_error=max(abs(x-y['m_Start']) for x,y in zip(sample(0),mc['m_ValueArrayDelta']))
    stop_error=max(abs(x-y['m_Stop']) for x,y in zip(sample(mc['m_StopTime']),mc['m_ValueArrayDelta']))
    result=dict(source=d['path'],source_object_sha256=d['raw_sha256'],duration=mc['m_StopTime'],sample_rate=tree['m_SampleRate'],
                stream_frames=[dict(time=t if math.isfinite(t) else str(t),keys=len(k)) for t,k in frames],stream_count=stream['curveCount'],dense_count=cc,
                dense_frames=dense['m_FrameCount'],constant_count=len(const),scalar_count=scalar_count,binding_width=binding_width,
                humanoid_muscle_binding_count=sum(b['typeID']==95 and 42<=b['attribute']<=136 for b in tree['m_ClipBindingConstant']['genericBindings']),
                start_reference_max_abs_error=start_error,stop_reference_max_abs_error=stop_error,
                samples=samples,limitation='Validated original scalar payload and endpoint reference agreement, not HumanPose-to-bone rotations or original rendered motion.')
    (ROOT/'animation-curves-validation.json').write_text(json.dumps(result,indent=2)+'\n')
    print({k:v for k,v in result.items() if k!='samples'})

if __name__=='__main__':run()
