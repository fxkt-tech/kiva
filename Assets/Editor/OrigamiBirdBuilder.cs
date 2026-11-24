using System.Collections.Generic;
using UnityEditor;
using UnityEngine;

// Faceted paper solids. The four meshes share their palette and hinge origins with the UI portrait.
public static class OrigamiBirdBuilder
{
    sealed class Folds
    {
        readonly Vector3 pivot;
        readonly List<Vector3> vertices=new List<Vector3>();
        readonly List<Vector2> uv=new List<Vector2>();
        readonly List<int>[] groups;
        public Folds(Vector3 pivot,int colors){
            this.pivot=pivot;groups=new List<int>[colors];
            for(int i=0;i<colors;i++)groups[i]=new List<int>();
        }
        void Face(int color,Vector3 a,Vector3 b,Vector3 c,Vector3 inside){
            var normal=Vector3.Cross(b-a,c-a);
            if(Vector3.Dot(normal,a-inside)<0){var temp=b;b=c;c=temp;}
            foreach(var point in new[]{a,b,c}){
                groups[color].Add(vertices.Count);
                vertices.Add(point-pivot);
                uv.Add(new Vector2(point.z+point.x*.35f,point.y)*2f);
            }
        }
        public void Diamond(int top,int underside,Vector3 rear,Vector3 tip,Vector3 left,Vector3 right,Vector3 ridge,Vector3 belly){
            var ring=new[]{rear,left,tip,right};
            var inside=(ridge+belly)*.5f;
            for(int i=0;i<4;i++){
                Face(top,ring[i],ring[(i+1)%4],ridge,inside);
                Face(underside,ring[(i+1)%4],ring[i],belly,inside);
            }
        }
        // Each section is a horizontal octagon (height, fore-aft centre, half-width, half-length).
        public void Body(Vector4[] sections,bool head){
            const int sides=8;
            var rings=new List<Vector3[]>();
            foreach(var s in sections){
                var ring=new Vector3[sides];
                for(int i=0;i<sides;i++){
                    float a=i*2*Mathf.PI/sides;
                    ring[i]=new Vector3(s.z*Mathf.Sin(a),s.x,s.y+s.w*Mathf.Cos(a));
                }
                rings.Add(ring);
            }
            for(int j=0;j<rings.Count-1;j++)for(int i=0;i<sides;i++){
                int n=(i+1)%sides;
                var a=rings[j][i];var b=rings[j][n];var c=rings[j+1][n];var d=rings[j+1][i];
                var inside=new Vector3(0,(a.y+d.y)*.5f,(sections[j].y+sections[j+1].y)*.5f);
                bool front=i<=1 || i>=6;
                int baseInk=head ? (j==0 && front ? 3 : 0) : (front ? 2 : 0);
                Face(baseInk,a,b,d,inside);
                Face(!front && (i+j)%3==0?1:baseInk,b,c,d,inside);
            }
            for(int i=1;i<sides-1;i++){
                var low=rings[0];var high=rings[rings.Count-1];
                Face(2,low[0],low[i],low[i+1],low[0]+Vector3.up);
                Face(0,high[0],high[i],high[i+1],high[0]-Vector3.up);
            }
        }
        public void Save(BirdSpecies species,ref Mesh target,string label){
            if(!target){target=new Mesh();AssetDatabase.AddObjectToAsset(target,species);}
            target.Clear();target.name=label;target.SetVertices(vertices);target.SetUVs(0,uv);
            var masks=new Color32[vertices.Count];
            for(int i=0;i<masks.Length;i++)masks[i]=Color.white;
            target.SetColors(masks);target.subMeshCount=groups.Length;
            for(int i=0;i<groups.Length;i++)target.SetTriangles(groups[i],i);
            target.RecalculateNormals();target.RecalculateTangents();target.RecalculateBounds();
            EditorUtility.SetDirty(target);
        }
    }

    [MenuItem("CONTEXT/BirdSpecies/重建折纸模型")]
    static void Rebuild(MenuCommand command){Bake((BirdSpecies)command.context);AssetDatabase.SaveAssets();}
    public static void Bake(BirdSpecies s)
    {
        bool king=!s.bobsTail && !s.forkedTail;
        var body=new Folds(Vector3.zero,s.pigments.Length);
        float w=king?.115f:.089f,z=king?.075f:.105f;
        body.Body(new[]{new Vector4(.065f,0,.032f,.045f),new Vector4(.12f,-.012f,w*.82f,.13f),
           new Vector4(.204f,0,w,.15f),new Vector4(.278f,.026f,w*.84f,.115f),new Vector4(.31f,.05f,.055f,.07f)},false);
        body.Body(new[]{new Vector4(.246f,z,.062f,.074f),new Vector4(.288f,z+.003f,.091f,.097f),
           new Vector4(.361f,z-.006f,.094f,.095f),new Vector4(king?.409f:.389f,z-.015f,.057f,.066f)},true);
        body.Diamond(4,4,new Vector3(0,.31f,z+.06f),new Vector3(0,.315f,z+(king?.315f:.175f)),
           new Vector3(-.034f,.318f,z+.085f),new Vector3(.034f,.318f,z+.085f),
           new Vector3(0,.340f,z+.10f),new Vector3(0,.298f,z+.10f));
        for(int side=-1;side<=1;side+=2){
            var e=new Vector3(side*.083f,.337f,z+.047f);
            body.Diamond(4,4,e-Vector3.forward*.016f,e+Vector3.forward*.013f,e-Vector3.up*.013f,e+Vector3.up*.012f,e+Vector3.right*side*.012f,e-Vector3.right*side*.012f);
            var foot=new Vector3(side*.039f,.016f,.016f);
            body.Diamond(4,2,foot+Vector3.up*.084f,foot-Vector3.up*.011f,foot-Vector3.right*.008f,foot+Vector3.right*.008f,foot+Vector3.forward*.009f,foot-Vector3.forward*.008f);
            body.Diamond(4,2,foot-Vector3.forward*.028f,foot+Vector3.forward*.044f,foot-Vector3.right*.014f,foot+Vector3.right*.014f,foot+Vector3.up*.009f,foot-Vector3.up*.006f);
        }
        body.Save(s,ref s.body,"身与喙");
        for(int part=1;part<=2;part++){
            int sign=part==1?-1:1;
            var wing=new Folds(s.Pivot(part),s.pigments.Length);
            Vector3 P(float x,float y,float f)=>new Vector3(sign*x,y,f);
            wing.Diamond(0,3,P(w*.67f,.277f,.064f),P(w*1.1f,.097f,s.forkedTail?-.264f:-.199f),P(w*.85f,.245f,-.124f),P(w*1.03f,.159f,.051f),P(w*1.23f,.209f,-.030f),P(w*.72f,.20f,-.032f));
            wing.Diamond(1,0,P(w*1.16f,.234f,.01f),P(w*1.18f,.108f,-.176f),P(w*1.19f,.21f,-.08f),P(w*1.24f,.178f,-.014f),P(w*1.29f,.19f,-.051f),P(w*1.12f,.19f,-.051f));
            if(part==1)wing.Save(s,ref s.leftWing,"左翼");else wing.Save(s,ref s.rightWing,"右翼");
        }
        var tail=new Folds(s.Pivot(3),s.pigments.Length);
        for(int k=-1;k<=1;k+=2){
            float spread=s.forkedTail?.12f:.036f;
            tail.Diamond(1,3,new Vector3(0,.181f,-.066f),new Vector3(k*spread,king?.086f:.119f,king?-.228f:-.376f),
                new Vector3(k*.064f,.17f,-.166f),new Vector3(k*.008f,.159f,-.156f),new Vector3(k*.037f,.181f,-.165f),new Vector3(k*.034f,.154f,-.168f));
        }
        tail.Save(s,ref s.tail,"尾褶");EditorUtility.SetDirty(s);
    }
}
