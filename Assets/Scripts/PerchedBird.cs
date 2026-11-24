using UnityEngine;

// The anchor and pickup stay fixed even when the swallow leaves its perch.
public sealed class PerchedBird : MonoBehaviour
{
    public WorkshopLevel.Perch Perch { get; private set; }
    public Vector3 Chest => transform.position + Vector3.up*.25f;
    public bool Available => !returning && model.gameObject.activeSelf;
    Transform model, feather;
    readonly Transform[] parts=new Transform[4];
    AudioSource voice;
    bool returning;
    float elapsed, phase;
    Vector3 startOffset;
    Quaternion homeRotation;

    public void Build(WorkshopLevel.Perch perch, Material[] palette)
    {
        Perch=perch;
        transform.localPosition=perch.feet;
        homeRotation=Quaternion.Euler(0,perch.yaw,0);
        model=new GameObject(perch.species.birdName).transform;
        model.SetParent(transform,false);
        for(int i=0;i<parts.Length;i++){
            var part=new GameObject(perch.species.Part(i).name,typeof(MeshFilter),typeof(MeshRenderer));
            parts[i]=part.transform;
            parts[i].SetParent(model,false);
            parts[i].localPosition=perch.species.Pivot(i);
            part.GetComponent<MeshFilter>().sharedMesh=perch.species.Part(i);
            part.GetComponent<MeshRenderer>().sharedMaterials=palette;
        }
        var folded=new PaperMesh.Builder(1);
        var spine=new Vector3(.02f,.055f,0);
        var tip=new Vector3(.13f,.018f,.16f);
        folded.Triangle(0,Vector3.up,new Vector3(-.10f,.015f,-.10f),spine,tip);
        folded.Triangle(0,Vector3.up,spine,new Vector3(.10f,.012f,.012f),tip);
        feather=folded.Mount(transform,"折羽",palette[3]).transform;
        voice=gameObject.AddComponent<AudioSource>();
        voice.playOnAwake=false;voice.spatialBlend=1;voice.minDistance=1.5f;voice.maxDistance=9;
        voice.rolloffMode=AudioRolloffMode.Linear;voice.clip=perch.species.call;voice.volume=.47f;
        phase=Mathf.Repeat(perch.feet.x*1.7f,7);
    }

    public void Refresh(bool collected)
    {
        returning=false;elapsed=0;voice.Stop();
        model.gameObject.SetActive(!collected);feather.gameObject.SetActive(collected);
        model.localPosition=Vector3.zero;model.localRotation=homeRotation;model.localScale=Vector3.one;
    }
    public void Call() { if(Available && voice.clip) voice.Play(); }
    public void ReturnToScroll(Vector3 visitor)
    {
        returning=true;elapsed=0;startOffset=model.localPosition;
        var face=Vector3.ProjectOnPlane(visitor-model.position,Vector3.up);
        if(face.sqrMagnitude>.0001f) model.rotation=Quaternion.LookRotation(face);
        voice.Stop();
    }
    void Update()
    {
        if(!model.gameObject.activeSelf || Time.deltaTime<=0)return;
        elapsed+=Time.deltaTime;phase+=Time.deltaTime*2.7f;
        float flight=0;
        if(returning){
            float t=Mathf.Clamp01(elapsed/.7f);
            model.localPosition=Vector3.Lerp(startOffset,Vector3.up*1.15f,t*t);
            model.localScale=Vector3.one*(1f-t*t);flight=1;
            if(t>=1){Refresh(true);return;}
        }else{
            float t=Mathf.Clamp01((Mathf.Repeat(phase/2.7f,9f)-7f)/2f);
            float arc=Mathf.Sin(Mathf.PI*t);
            flight=Perch.flightOffset.sqrMagnitude>0 ? arc : 0;
            model.localPosition=Perch.flightOffset*arc+Vector3.up*(.006f*Mathf.Sin(phase));
            model.localRotation=homeRotation*Quaternion.Euler(0,9*Mathf.Sin(phase*.39f),0);
        }
        for(int i=1;i<parts.Length;i++)parts[i].localRotation=Perch.species.Pose(i,phase,flight);
    }
}
