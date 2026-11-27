using UnityEngine;

// Three alternating folds in each cut sector compress it into the coloured rim.
// The reverse winding has its own pigment, visible as soon as the first fold turns over.
public sealed class PortalFolds : MonoBehaviour
{
    const int Pleats=3;
    Mesh mesh;
    int leaves;
    Vector3[] vertices;
    Vector2[] uv;
    float amount,target;
    bool retire;
    const float Duration=.39f;

    public void Initialize()
    {
        var filter = GetComponent<MeshFilter>();
        mesh = Instantiate(filter.sharedMesh);
        mesh.MarkDynamic();
        filter.sharedMesh = mesh;
        vertices = mesh.vertices;
        uv = mesh.uv;
        leaves = vertices.Length / (Pleats * 8);
    }

    public void Open(bool restart,bool paired)
    {
        if(restart)amount=0;
        retire=false;
        target=paired?1:.35f;
        gameObject.SetActive(true);
        Apply();
    }

    public void CloseAt(Transform oldPortal,PortalFolds original)
    {
        transform.SetPositionAndRotation(oldPortal.position,oldPortal.rotation);
        amount=original.amount;
        target=0;retire=true;
        gameObject.SetActive(true);
        Apply();
    }

    void Update()
    {
        if(amount!=target){
            amount=Mathf.MoveTowards(amount,target,Time.deltaTime/Duration);
            Apply();
        }
        if(retire && amount==0)gameObject.SetActive(false);
    }

    static Vector3 Edge(float angle)=>new Vector3(Mathf.Sin(angle)*WorkshopPortals.Radius,
        Mathf.Cos(angle)*WorkshopPortals.Radius,.055f);

    void Write(int k,Vector3 point,Vector2 paperUV)
    {
        vertices[k]=vertices[k+4]=point;
        uv[k]=uv[k+4]=paperUV;
    }

    void Apply()
    {
        for(int i=0;i<leaves;i++){
            var a=Edge(i*2*Mathf.PI/leaves);
            var b=Edge((i+1)*2*Mathf.PI/leaves);
            var middle=(a+b)*.5f;
            var toward=-new Vector3(middle.x,middle.y,0)/Pleats;
            var axis=(b-a).normalized;
            float stagger=(i%3)*.035f;
            float progress=Mathf.SmoothStep(0,1,Mathf.Clamp01((amount-stagger)/(1-stagger)));
            var hinge=middle;
            for(int j=0;j<Pleats;j++){
                float angle=j==0?97:j==1?-85:79;
                // axis and inward-facing radial direction turn through the viewer-facing half space.
                var next=hinge+Quaternion.AngleAxis(-angle*progress,axis)*toward;
                var half=(b-a)*(.5f*(1-j/(float)Pleats));
                var nextHalf=(b-a)*(.5f*(1-(j+1)/(float)Pleats));
                int k=(i*Pleats+j)*8;
                // UVs follow the original flat sheet through the folds.
                // Tuck the final pleat flat under the circular binding.
                float tuck=Mathf.SmoothStep(0,1,Mathf.InverseLerp(.62f,1,progress));
                Vector3 Stow(Vector3 p,Vector3 edge) => Vector3.Lerp(p,
                    new Vector3(edge.x*1.035f,edge.y*1.035f,.022f),tuck);
                Write(k,Stow(hinge+half,b),(Vector2)b*(1-j/(float)Pleats));
                Write(k+1,Stow(hinge-half,a),(Vector2)a*(1-j/(float)Pleats));
                Write(k+2,Stow(next-nextHalf,a),(Vector2)a*(1-(j+1)/(float)Pleats));
                Write(k+3,Stow(next+nextHalf,b),(Vector2)b*(1-(j+1)/(float)Pleats));
                hinge=next;
            }
        }
        mesh.vertices=vertices;mesh.uv=uv;
        mesh.RecalculateNormals();mesh.RecalculateTangents();mesh.RecalculateBounds();
    }

    void OnDestroy(){if(mesh)Destroy(mesh);}
}
