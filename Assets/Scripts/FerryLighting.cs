using UnityEngine;
using UnityEngine.Rendering;
using UnityEngine.Rendering.HighDefinition;

public sealed class FerryLighting : MonoBehaviour
{
    Transform carriage;
    Transform hangingCord;
    Material lanternPaper, lanternLaps, lanternCap;
    VolumeProfile profile;
    float phase;

    public static readonly Color OchreBack=new Color(.91f,.61f,.40f);

    public static Material FoldedMineral(Color pigment)
    {
        var mat=new Material(Resources.Load<Shader>("Mineral/FoldPigment")){name="折纸岩彩"};
        mat.SetColor("_Pigment",pigment);
        mat.SetTexture("_Fibres",Resources.Load<Texture2D>("Mineral/MulberryDetail"));
        mat.SetTexture("_MineralWash",Resources.Load<Texture2D>("Mineral/Pigments"));
        HDMaterial.ValidateMaterial(mat);
        return mat;
    }

    public static Material Mineral(string family, Color color)
    {
        var material = new Material(Shader.Find("HDRP/Lit")) { name=family };
        material.SetColor("_BaseColor",color);
        material.SetTexture("_BaseColorMap",Resources.Load<Texture2D>("Mineral/Mulberry"));
        bool quill=family=="Quill";
        material.SetFloat("_UVBase",quill?5:0);
        material.SetFloat("_ObjectSpaceUVMapping",1f);
        material.SetTextureScale("_BaseColorMap",Vector2.one*.65f);
        material.SetTexture("_DetailMap",Resources.Load<Texture2D>("Mineral/MulberryDetail"));
        material.SetTextureScale("_DetailMap",Vector2.one*.64f);
        material.SetFloat("_DetailAlbedoScale",quill?.24f:.38f);
        material.SetFloat("_DetailNormalScale",quill?.055f:.12f);
        material.SetFloat("_DetailSmoothnessScale",0);
        material.SetFloat("_Metallic",0);
        material.SetFloat("_Smoothness",family=="Lacquer"?.24f:.11f);
        // A small amount of relief keeps the paper tactile at arm's length.
        if(family=="Paper"){
            material.SetTexture("_NormalMap",Resources.Load<Texture2D>("Mineral/PaperNormal"));
            material.SetFloat("_NormalScale",.08f);
        }
        HDMaterial.ValidateMaterial(material);
        return material;
    }

    static Light AddLight(Transform parent, string name, Vector3 position, LightType type,
        Color color, LightUnit unit, float intensity)
    {
        var light = new GameObject(name).AddComponent<Light>();
        light.transform.SetParent(parent, false);
        light.transform.localPosition = position;
        light.type = type;
        light.gameObject.AddComponent<HDAdditionalLightData>();
        light.color = color;
        light.lightUnit = unit;
        light.intensity = intensity;
        return light;
    }

    Light Spot(string name, Vector3 origin, Vector3 aim, Color color, float candela,
        float angle, float range)
    {
        var light = AddLight(transform, name, origin, LightType.Spot, color, LightUnit.Candela, candela);
        light.transform.localRotation = Quaternion.LookRotation(aim - origin);
        light.spotAngle = angle;
        light.innerSpotAngle = angle * .58f;
        light.range = range;
        return light;
    }

    HDAdditionalLightData sunShadows;
    public void ApplyQuality()=>sunShadows.SetShadowResolution(GameSettings.Current.shadowResolution);

    public void Initialize(Camera eyes, Transform exitLamp)
    {
        LightRooms();
        LightLanterns(exitLamp);
        ConfigureViews();
        PositionLantern();
    }

    public void LightGeneratedRooms(WorkshopLevel level)
    {
        foreach(var beam in level.fixedBricks){
            if(beam.label!="回廊檐梁")continue;
            var center=((Vector3)beam.start+(Vector3)beam.size*.5f)*PaperTerrain.Cell;
            center.y=beam.start.y*PaperTerrain.Cell-.55f;
            var along=beam.size.x>beam.size.z?Vector3.right:Vector3.forward;
            int bays=Mathf.Max(beam.size.x,beam.size.z);
            for(float offset=2.5f;offset<bays;offset+=4){
                var hanger=new GameObject("檐下纸灯").transform;
                hanger.SetParent(transform,false);
                hanger.localPosition=center+along*((offset-bays*.5f)*PaperTerrain.Cell);
                hanger.localScale=Vector3.one*1.45f;
                FoldedPaper.Lantern(hanger,lanternPaper,lanternLaps,lanternCap);
            }
        }
        if(!System.Array.Exists(level.fixedBricks,b=>b.label=="房间顶棚"))return;
        foreach(var pad in level.fixedBricks){
            if(pad.label!="起步台" && pad.label!="起步侧台" && pad.label!="连接走台" &&
               pad.label!="目标高台" && !(pad.label?.StartsWith("落脚台 ",System.StringComparison.Ordinal)??false))continue;
            var center=((Vector3)pad.start+(Vector3)pad.size*.5f)*PaperTerrain.Cell;
            center.y=(pad.start.y+pad.size.y)*PaperTerrain.Cell+2.8f;
            var light=AddLight(transform,"室内纸光",center,LightType.Point,new Color(1f,.92f,.79f),LightUnit.Lumen,3700);
            light.range=16f;
        }
    }

    void LightRooms()
    {
        // The real west aperture and its muntins shape the directional light through the
        // non-shadow-casting window paper.
        var sun=AddLight(transform,"西窗日光",Vector3.zero,LightType.Directional,
            new Color(1f,.94f,.82f),LightUnit.Lux,1300);
        sun.transform.localRotation=Quaternion.LookRotation(new Vector3(1f,-.67f,.28f));
        sun.shadows=LightShadows.Soft;
        sun.shadowNearPlane=.08f;
        var hdSun=sun.GetComponent<HDAdditionalLightData>();
        hdSun.angularDiameter=.34f;
        hdSun.dirLightPCSSMaxPenumbraSize=.055f;
        hdSun.dirLightPCSSMinFilterSizeTexels=1f;
        hdSun.shadowDimmer=.9f;
        sunShadows=hdSun;
        ApplyQuality();

        Spot("窗纸漫射",new Vector3(-5.73f,4.2f,-1.5f),new Vector3(.2f,1.8f,4.8f),
            new Color(1f,.94f,.85f),20500f,130f,22);
        Spot("暖纸反照",new Vector3(3.4f,5.6f,-4.7f),new Vector3(-1f,2.1f,4.8f),
            new Color(.83f,.92f,1f),15000f,124f,22);
        Spot("窗下纸色回光",new Vector3(-2.8f,3.6f,-5.5f),new Vector3(-3.4f,.9f,-1.5f),
            new Color(.88f,.85f,.99f),1900f,125f,10f);
        Spot("井壁回光",new Vector3(3.8f,1.8f,2f),new Vector3(.2f,-4f,3f),
            new Color(.77f,.90f,.98f),2500f,112f,12);
    }

    void LightLanterns(Transform exitLamp)
    {
        carriage=new GameObject("巡游纸灯").transform;
        carriage.SetParent(transform,false);
        lanternPaper=LampPigment(new Color(.98f,.73f,.32f),new Color(150,75,20));
        lanternLaps=LampPigment(new Color(.86f,.43f,.18f),new Color(44,17,3));
        lanternCap=Mineral("Paper",new Color(.48f,.17f,.12f));
        carriage.localScale=Vector3.one*1.35f;
        FoldedPaper.Lantern(carriage,lanternPaper,lanternLaps,lanternCap);
        var cord=GameObject.CreatePrimitive(PrimitiveType.Cylinder);
        cord.name="巡灯吊线";
        hangingCord=cord.transform;
        hangingCord.SetParent(transform,false);
        var collider=cord.GetComponent<Collider>();
        collider.enabled=false;
        Destroy(collider);
        cord.GetComponent<MeshRenderer>().sharedMaterial=lanternCap;
        FoldedPaper.Beacon(exitLamp,lanternPaper,lanternLaps,lanternCap);
        var moving = AddLight(carriage, "灯下暖光", Vector3.down*.38f, LightType.Point,
            new Color(1f,.70f,.43f), LightUnit.Lumen, 1650f);
        moving.range=9;
        moving.shadows=LightShadows.Soft;
        moving.shapeRadius=.30f;
        var beacon = AddLight(exitLamp, "船灯反光", Vector3.down*.48f, LightType.Point,
            new Color(1f,.72f,.45f), LightUnit.Lumen, 1200f);
        beacon.range = 5f;
        beacon.shapeRadius = .35f;
    }

    void ConfigureViews()
    {
        var volume=gameObject.AddComponent<Volume>();
        volume.isGlobal=true;
        volume.priority=100;
        profile=volume.profile;
        profile.Add<DiffusionProfileList>(true).diffusionProfiles.Override(
            new[]{Resources.Load<DiffusionProfileSettings>("Mineral/WindowPaperProfile")});
        var exposure=profile.Add<Exposure>(true);
        exposure.mode.Override(ExposureMode.Fixed);
        exposure.fixedExposure.Override(6.6f);
        var indirect=profile.Add<IndirectLightingController>(true);
        indirect.indirectDiffuseLightingMultiplier.Override(.19f);
        indirect.reflectionLightingMultiplier.Override(.29f);
        var fog=profile.Add<Fog>(true);
        fog.enabled.Override(false);
        profile.Add<HDShadowSettings>().maxShadowDistance.Override(44f);
        var occlusion = profile.Add<ScreenSpaceAmbientOcclusion>();
        occlusion.intensity.Override(.65f);
        occlusion.radius.Override(.45f);
        occlusion.directLightingStrength.Override(.16f);
        var bloom = profile.Add<Bloom>();
        bloom.threshold.Override(1.1f);
        bloom.intensity.Override(.12f);
        profile.Add<MotionBlur>().intensity.Override(0f);
        foreach(var cam in GetComponentsInChildren<Camera>(true)){
            var view=cam.GetComponent<HDAdditionalCameraData>()??cam.gameObject.AddComponent<HDAdditionalCameraData>();
            view.volumeLayerMask=-1;
            view.antialiasing = HDAdditionalCameraData.AntialiasingMode.SubpixelMorphologicalAntiAliasing;
            view.SMAAQuality = HDAdditionalCameraData.SMAAQualityLevel.High;
            cam.allowMSAA=false;
        }
    }

    static Material LampPigment(Color tint,Color transmitted)
    {
        var mat=Mineral("Fold",tint);
        mat.SetColor("_EmissiveColor",transmitted);
        HDMaterial.ValidateMaterial(mat);
        return mat;
    }

    void Update()
    {
        phase=Mathf.Repeat(phase+Time.deltaTime*.42f,2*Mathf.PI);
        PositionLantern();
    }

    void PositionLantern()
    {
        var center=new Vector3(.66f,5.3f,-2.15f);
        carriage.localPosition=center+new Vector3(.78f*Mathf.Cos(phase),.20f*Mathf.Sin(phase*2),.83f*Mathf.Sin(phase));
        carriage.localRotation=Quaternion.Euler(0,phase*Mathf.Rad2Deg,4*Mathf.Sin(phase));
        var crown=transform.InverseTransformPoint(carriage.TransformPoint(Vector3.up*.39f));
        float length=7.5f-crown.y;
        hangingCord.localPosition=crown+Vector3.up*length*.5f;
        hangingCord.localScale=new Vector3(.018f,length*.5f,.018f);
    }

    void OnDestroy(){if(lanternPaper)Destroy(lanternPaper);if(lanternLaps)Destroy(lanternLaps);if(lanternCap)Destroy(lanternCap);if(profile)Destroy(profile);}
}
