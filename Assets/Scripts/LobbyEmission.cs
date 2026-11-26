using UnityEngine;
using UnityEngine.Experimental.Rendering;
using UnityEngine.Rendering;
using UnityEngine.Rendering.HighDefinition;
using UnityEngine.UI;

public sealed class LobbyEmission : MonoBehaviour
{
    [SerializeField] Texture2D emissionMask;
    [SerializeField] Shader emissionShader;
    [SerializeField] Shader compositeShader;
    [SerializeField] RawImage[] graphics;

    RenderTexture output;
    Material emission, composite;
    VolumeProfile profile;
    Camera bloomCamera;

    void OnEnable()
    {
        if (!bloomCamera) CreateBloomCamera();
        bloomCamera.enabled = true;
        RenderPipelineManager.endCameraRendering += FinishRender;
    }

    void OnDisable()
    {
        RenderPipelineManager.endCameraRendering -= FinishRender;
        if (bloomCamera) bloomCamera.enabled = false;
    }

    void CreateBloomCamera()
    {
        output = new RenderTexture(emissionMask.width, emissionMask.height, 0, GraphicsFormat.R16G16B16A16_SFloat)
        {
            name = "Lobby Emission HDR",
            wrapMode = TextureWrapMode.Clamp,
            filterMode = FilterMode.Bilinear
        };
        emission = new Material(emissionShader) { mainTexture = emissionMask };
        composite = new Material(compositeShader);
        composite.SetTexture("_EmissionMask", emissionMask);
        foreach (var graphic in graphics)
        {
            graphic.texture = output;
            graphic.material = composite;
        }

        var renderObject = new GameObject("Lobby Bloom Camera");
        renderObject.SetActive(false);
        renderObject.transform.SetParent(transform, false);
        renderObject.layer = gameObject.layer;
        bloomCamera = renderObject.AddComponent<Camera>();
        bloomCamera.depth = -10;
        bloomCamera.targetTexture = output;
        bloomCamera.clearFlags = CameraClearFlags.SolidColor;
        bloomCamera.backgroundColor = Color.black;
        bloomCamera.cullingMask = 0;
        bloomCamera.allowHDR = true;
        bloomCamera.allowMSAA = false;

        var cameraData = renderObject.AddComponent<HDAdditionalCameraData>();
        cameraData.clearColorMode = HDAdditionalCameraData.ClearColorMode.Color;
        cameraData.backgroundColorHDR = Color.black;
        cameraData.volumeLayerMask = 1 << renderObject.layer;
        cameraData.customRenderingSettings = true;
        cameraData.renderingPathCustomFrameSettings.SetEnabled(FrameSettingsField.ExposureControl, false);
        cameraData.renderingPathCustomFrameSettingsOverrideMask.mask[(uint)FrameSettingsField.ExposureControl] = true;

        profile = ScriptableObject.CreateInstance<VolumeProfile>();
        var bloom = profile.Add<Bloom>(true);
        bloom.threshold.Override(1f);
        bloom.intensity.Override(.32f);
        bloom.scatter.Override(.52f);
        bloom.tint.Override(Color.white);
        bloom.anamorphic.Override(false);
        profile.Add<Tonemapping>().mode.Override(TonemappingMode.None);

        var volume = renderObject.AddComponent<Volume>();
        volume.isGlobal = true;
        volume.priority = 100;
        volume.sharedProfile = profile;

        var pass = renderObject.AddComponent<CustomPassVolume>();
        pass.targetCamera = bloomCamera;
        pass.injectionPoint = CustomPassInjectionPoint.BeforePostProcess;
        pass.customPasses.Add(new FullScreenCustomPass
        {
            name = "Lobby Emission",
            fullscreenPassMaterial = emission,
            materialPassName = "Emission",
            targetColorBuffer = CustomPass.TargetBuffer.Camera,
            targetDepthBuffer = CustomPass.TargetBuffer.None
        });
        renderObject.SetActive(true);
    }

    void FinishRender(ScriptableRenderContext context, Camera camera)
    {
        if (camera != bloomCamera) return;
        bloomCamera.enabled = false;
        foreach (var graphic in graphics) graphic.enabled = true;
    }

    void OnDestroy()
    {
        if (output)
        {
            output.Release();
            Destroy(output);
        }
        Destroy(emission);
        Destroy(composite);
        if (profile)
        {
            foreach (var effect in profile.components) Destroy(effect);
            Destroy(profile);
        }
    }
}
