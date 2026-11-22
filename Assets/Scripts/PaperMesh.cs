using System.Collections.Generic;
using UnityEngine;
using UnityEngine.Rendering;

// Geometry for pigment-covered paper. Vertices are split at folds so each side catches the light separately.
public sealed class PaperMesh : MonoBehaviour
{
    Mesh ownedMesh;
    void OnDestroy() { if (ownedMesh) Destroy(ownedMesh); }

    public sealed class Builder
    {
        readonly List<Vector3> vertices = new List<Vector3>();
        readonly List<Vector2> uv = new List<Vector2>();
        readonly List<int>[] triangles;
        readonly List<Color32> masks;
        public Color32 FoldMask { get; set; } = Color.white;
        public Builder(int materials,bool folded=false)
        {
            if(folded)masks=new List<Color32>();
            triangles = new List<int>[materials];
            for (int i=0; i<materials; i++) triangles[i] = new List<int>();
        }
        public void Triangle(int ink, Vector3 normal, Vector3 a, Vector3 b, Vector3 c)
        {
            Vector3 cross = Vector3.Cross(b-a,c-a);
            if (cross.sqrMagnitude < 1e-16f) return;
            if (Vector3.Dot(cross,normal)<0) { var swap=b; b=c; c=swap; }
            int start=vertices.Count;
            vertices.Add(a); vertices.Add(b); vertices.Add(c);
            normal=normal.normalized;
            Vector3 u=Mathf.Abs(normal.y)>.75f ? Vector3.right : Vector3.Cross(Vector3.up, normal).normalized;
            Vector3 v=Vector3.Cross(normal,u);
            for (int k=start;k<start+3;k++)
            {
                var p=vertices[k];
                uv.Add(new Vector2(Vector3.Dot(p,u),Vector3.Dot(p,v))*.76f);
                masks?.Add(FoldMask);
                triangles[ink].Add(k);
            }
        }
        public void Quad(int ink, Vector3 normal, Vector3 a,Vector3 b,Vector3 c,Vector3 d)
        {
            Triangle(ink,normal,a,b,c); Triangle(ink,normal,a,c,d);
        }
        public GameObject Mount(Transform parent,string name,params Material[] materials)
        {
            var go=new GameObject(name,typeof(MeshFilter),typeof(MeshRenderer));
            go.transform.SetParent(parent,false);
            var mesh=new Mesh {name=name};
            if(vertices.Count>65535)mesh.indexFormat=IndexFormat.UInt32;
            mesh.SetVertices(vertices); mesh.SetUVs(0,uv);
            if(masks!=null)mesh.SetColors(masks);
            var visible=new List<Material>();
            for(int i=0;i<triangles.Length;i++)if(triangles[i].Count>0)visible.Add(materials[i]);
            mesh.subMeshCount=visible.Count;
            int slot=0;
            for(int i=0;i<triangles.Length;i++)if(triangles[i].Count>0)mesh.SetTriangles(triangles[i],slot++);
            mesh.RecalculateNormals();mesh.RecalculateTangents();mesh.RecalculateBounds();
            go.GetComponent<MeshFilter>().sharedMesh=mesh;
            go.GetComponent<MeshRenderer>().sharedMaterials=visible.ToArray();
            go.AddComponent<PaperMesh>().ownedMesh=mesh;
            return go;
        }
    }
}
