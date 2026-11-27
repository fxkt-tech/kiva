using UnityEngine;
using UnityEngine.Rendering.Universal;
using UnityEngine.Rendering;
using System.Collections.Generic;

public sealed class PortalWindow : MonoBehaviour
{
    const int WindowLayer = 2; // The player's aperture is hidden from all virtual cameras.
    static readonly Quaternion HalfTurn = Quaternion.Euler(0, 180, 0);
    readonly Plane[] frustum = new Plane[6];
    readonly List<View> views = new List<View>();
    Material paper;
    Shader windowShader;
    Mesh mesh;
    MeshRenderer surface;

    PortalFolds folds,closing;
    bool placed, needsFirstFrame;

    sealed class View
    {
        public Camera camera;
        public Material display;
        public RenderTexture target;
    }

    public void Place(Transform panel,Vector3 position,bool paired)
    {
        bool moved=!placed || Vector3.SqrMagnitude(transform.position-position)>.000001f ||
            Quaternion.Angle(transform.rotation,panel.rotation)>.01f;
        if(placed && moved)closing.CloseAt(transform,folds);
        transform.SetParent(panel,true);
        transform.SetPositionAndRotation(position,panel.rotation);
        gameObject.SetActive(true);
        folds.Open(moved,paired);
        placed=true;
    }

    public void Clear()
    {
        placed=false;
        closing.gameObject.SetActive(false);
        gameObject.SetActive(false);
    }

    public void Build(int sides,Material ink,Material reverse,bool notched)
    {
        var band=new PaperMesh.Builder(2);
        float border=WorkshopPortals.OuterRadius-WorkshopPortals.Radius;
        Vector3 Ring(float angle,float radius,float depth) => new Vector3(
            Mathf.Sin(angle)*radius,Mathf.Cos(angle)*radius,depth);
        for(int i=0;i<sides;i++){
            float a=i*2*Mathf.PI/sides,b=(i+1)*2*Mathf.PI/sides;
            float outerA=WorkshopPortals.Radius+border*(notched && i%12==0?.46f:.93f);
            float outerB=WorkshopPortals.Radius+border*(notched && (i+1)%12==0?.46f:.93f);
            var inside=Ring(a,WorkshopPortals.Radius,.029f);
            var insideB=Ring(b,WorkshopPortals.Radius,.029f);
            var ridge=Ring(a,WorkshopPortals.Radius+border*.25f,.045f);
            var ridgeB=Ring(b,WorkshopPortals.Radius+border*.25f,.045f);
            band.Quad(0,Vector3.forward,inside,insideB,ridgeB,ridge);
            band.Quad(0,Vector3.forward,ridge,ridgeB,Ring(b,outerB,.024f),Ring(a,outerA,.024f));
            band.Quad(1,Vector3.forward,Ring(a,outerA,.024f),Ring(b,outerB,.024f),
                Ring(b,outerB+border*.07f,.014f),Ring(a,outerA+border*.07f,.014f));
        }
        band.Mount(transform,"圆形折纸门缘",ink,reverse).layer=gameObject.layer;
        folds=PortalFolds.Create(transform,Mathf.Min(sides,16),ink,reverse);
        folds.gameObject.layer=gameObject.layer;
        closing=PortalFolds.Create(transform.parent,Mathf.Min(sides,16),ink,reverse);
        closing.gameObject.layer=transform.parent.gameObject.layer;
        closing.gameObject.SetActive(false);
        var face = new GameObject("门内视野", typeof(MeshFilter), typeof(MeshRenderer));
        face.transform.SetParent(transform,false);
        face.layer = WindowLayer;
        var vertices = new Vector3[sides+1];
        var triangles = new int[sides*3];
        for(int i=0;i<sides;i++)
        {
            float angle = i*2*Mathf.PI/sides;
            vertices[i+1] = new Vector3(Mathf.Sin(angle)*WorkshopPortals.Radius,
                Mathf.Cos(angle)*WorkshopPortals.Radius,0);
            triangles[i*3+1]=(i+1)%sides+1;
            triangles[i*3+2]=i+1;
        }
        mesh = new Mesh { vertices=vertices, triangles=triangles };
        var uv=new Vector2[vertices.Length];
        for(int i=0;i<vertices.Length;i++)uv[i]=(Vector2)vertices[i]/(2*WorkshopPortals.Radius)+Vector2.one*.5f;
        mesh.uv=uv;
        mesh.RecalculateNormals();
        mesh.RecalculateTangents();
        var bounds=mesh.bounds;
        bounds.Expand(new Vector3(0,0,.2f)); // Keep the disc visible inside the eye's near plane.
        mesh.bounds=bounds;
        face.GetComponent<MeshFilter>().sharedMesh=mesh;
        surface=face.GetComponent<MeshRenderer>();
        paper=reverse;
        surface.sharedMaterial=paper;
        surface.shadowCastingMode=ShadowCastingMode.Off;
        windowShader=Resources.Load<Shader>("Portals/Window");
    }

    View CreateView(Camera player,int depth)
    {
        var cameraObject=new GameObject($"对侧视角 · {depth}");
        cameraObject.transform.SetParent(transform,false);
        var camera=cameraObject.AddComponent<Camera>();
        camera.enabled=false;
        camera.allowMSAA=player.allowMSAA;
        // Apply the display transform once, after the player camera has composited
        // the HDR view through the opening and the surrounding paper rim.
        camera.allowHDR=true;
        var data=camera.GetUniversalAdditionalCameraData();
        data.renderPostProcessing=false;
        data.volumeLayerMask=player.GetUniversalAdditionalCameraData().volumeLayerMask;
        var display=new Material(windowShader);
        display.SetFloat("_ClampNearPlane",depth==1?1:0);
        return new View { camera=camera, display=display };
    }

    public void PrepareTarget(Camera player)
    {
        // Allocate on level entry, depth changes or resize, never based on the projected aperture size.
        int width=player.pixelWidth,height=player.pixelHeight;
        if(width<1 || height<1)return;
        int depth=GameSettings.Current.portalDepth;
        while(views.Count>depth){
            ReleaseView(views[views.Count-1]);
            views.RemoveAt(views.Count-1);
        }
        float scale=Mathf.Min(.75f,1600f/Mathf.Max(width,height));
        for(int i=0;i<depth;i++,scale*=.5f){
            if(i==views.Count)views.Add(CreateView(player,i+1));
            var pass=views[i];
            int w=Mathf.Max(1,Mathf.RoundToInt(width*scale)),h=Mathf.Max(1,Mathf.RoundToInt(height*scale));
            var texture=pass.target;
            if(texture && texture.width==w && texture.height==h && texture.IsCreated())continue;
            ReleaseTarget(pass);
            texture=new RenderTexture(w,h,24,RenderTextureFormat.DefaultHDR){
                name=$"{name} · 第 {i+1} 层",filterMode=FilterMode.Bilinear,wrapMode=TextureWrapMode.Clamp
            };
            texture.Create();
            pass.target=texture;
            pass.camera.targetTexture=texture;
            pass.display.mainTexture=texture;
            if(i==0)needsFirstFrame=true;
        }
    }

    public void Draw(Camera player,Transform exit,bool paired)
    {
        if(paired)PrepareTarget(player);
        var observer=player;
        var aperture=new Rect(0,0,1,1);
        int active=0;
        bool rootVisible=false;
        // With a pair of portals the exit is behind its own clip plane. Only this entry can recur.
        while(paired && VisibleFrom(observer,ref aperture,out float distance)){
            var display=active<views.Count ? views[active].display : paper;
            float offset=Mathf.Min(.012f,distance*.25f);
            if(active==0){
                surface.transform.localPosition=Vector3.forward*offset;
                if(surface.sharedMaterial!=display)surface.sharedMaterial=display;
                rootVisible=true;
            }else{
                // Bind the child texture only for this observer; the opposite portal has its own chain.
                var draw=new RenderParams(display){camera=observer,layer=gameObject.layer,
                    shadowCastingMode=ShadowCastingMode.Off,receiveShadows=false};
                var matrix=transform.localToWorldMatrix*Matrix4x4.Translate(Vector3.forward*offset);
                Graphics.RenderMesh(draw,mesh,0,matrix);
            }
            if(active==views.Count)break;
            var next=views[active++].camera;
            ConfigureView(next,observer,player,exit,active);
            next.enabled=true;
            observer=next;
        }
        surface.enabled=rootVisible;
        // Prime the other direction once when a pair opens, before the player crosses.
        if(paired && needsFirstFrame && views.Count>0){
            if(active==0){
                var first=views[active++].camera;
                ConfigureView(first,player,player,exit,1);
                first.projectionMatrix=player.projectionMatrix;
                first.enabled=true;
            }
            needsFirstFrame=false;
        }
        for(int i=active;i<views.Count;i++)views[i].camera.enabled=false;
    }

    bool VisibleFrom(Camera observer,ref Rect aperture,out float distance)
    {
        distance=Vector3.Dot(observer.transform.position-transform.position,transform.forward);
        if(distance<=0)return false;
        GeometryUtility.CalculateFrustumPlanes(observer,frustum);
        if(!GeometryUtility.TestPlanesAABB(frustum,surface.bounds))return false;

        // Intersect projected bounds with all ancestor apertures in the same screen coordinates.
        var min=new Vector2(float.PositiveInfinity,float.PositiveInfinity);
        var max=new Vector2(float.NegativeInfinity,float.NegativeInfinity);
        for(int i=0;i<4;i++){
            var corner=new Vector3((i&1)==0?-1:1,(i&2)==0?-1:1,0)*WorkshopPortals.Radius;
            var point=observer.WorldToViewportPoint(transform.TransformPoint(corner));
            if(point.z<=0)return true; // Eye intersects the bounding square; keep the parent bounds.
            min=Vector2.Min(min,point);
            max=Vector2.Max(max,point);
        }
        aperture=Rect.MinMaxRect(Mathf.Max(min.x,aperture.xMin),Mathf.Max(min.y,aperture.yMin),
            Mathf.Min(max.x,aperture.xMax),Mathf.Min(max.y,aperture.yMax));
        return aperture.width>0 && aperture.height>0;
    }

    void ConfigureView(Camera view,Camera observer,Camera player,Transform exit,int depth)
    {
        var offset=HalfTurn*transform.InverseTransformPoint(observer.transform.position);
        view.transform.SetPositionAndRotation(exit.TransformPoint(offset),
            exit.rotation*HalfTurn*Quaternion.Inverse(transform.rotation)*observer.transform.rotation);
        view.depth=player.depth-depth; // Unity renders the deepest target before its consumer.
        view.clearFlags=player.clearFlags;
        view.backgroundColor=player.backgroundColor;
        view.cullingMask=(player.cullingMask | (1<<VermilionBird.ModelLayer)) & ~(1<<WindowLayer) & ~(1<<exit.gameObject.layer);
        view.fieldOfView=player.fieldOfView;
        view.aspect=player.aspect;
        view.nearClipPlane=player.nearClipPlane;
        view.farClipPlane=player.farClipPlane;
        view.projectionMatrix=player.projectionMatrix;
        var normal=view.worldToCameraMatrix.MultiplyVector(exit.forward).normalized;
        var point=view.worldToCameraMatrix.MultiplyPoint(exit.position+exit.forward*.02f);
        view.projectionMatrix=view.CalculateObliqueMatrix(new Vector4(normal.x,normal.y,normal.z,-Vector3.Dot(point,normal)));
    }

    static void ReleaseTarget(View pass)
    {
        var target=pass.target;
        if(!target)return;
        pass.display.mainTexture=null;
        if(pass.camera)pass.camera.targetTexture=null;
        target.Release();
        Destroy(target);
        pass.target=null;
    }

    static void ReleaseView(View pass)
    {
        if(pass.camera)pass.camera.enabled=false;
        ReleaseTarget(pass);
        Destroy(pass.display);
        if(pass.camera)Destroy(pass.camera.gameObject);
    }

    void OnDestroy()
    {
        foreach(var pass in views)ReleaseView(pass);
        if(mesh)Destroy(mesh);
    }
}
