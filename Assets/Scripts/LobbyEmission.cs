using UnityEngine;
using UnityEngine.Experimental.Rendering;
using UnityEngine.Rendering;
using UnityEngine.Rendering.Universal;
using UnityEngine.UI;

public sealed class LobbyEmission : MonoBehaviour
{
    [SerializeField] Material emissionMaterial;
    [SerializeField] Shader compositeShader;
    [SerializeField] RawImage[] graphics;

    RenderTexture output;
    Material composite;
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
        var emissionMask = emissionMaterial.mainTexture;
        output = new RenderTexture(emissionMask.width, emissionMask.height, 24, GraphicsFormat.R16G16B16A16_SFloat)
        {
            name = "Lobby Emission HDR",
            wrapMode = TextureWrapMode.Clamp,
            filterMode = FilterMode.Bilinear
        };
        composite = new Material(compositeShader);
        composite.SetTexture("_EmissionMask", emissionMask);
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

        var cameraData = bloomCamera.GetUniversalAdditionalCameraData();
        cameraData.SetRenderer(1); // Lobby Bloom Renderer draws the glyph mask before bloom.
        cameraData.renderPostProcessing = true;
        cameraData.volumeLayerMask = 1 << renderObject.layer;

        profile = ScriptableObject.CreateInstance<VolumeProfile>();
        var bloom = profile.Add<Bloom>(true);
        bloom.threshold.Override(1f);
        bloom.intensity.Override(.32f);
        bloom.scatter.Override(.52f);
        bloom.tint.Override(Color.white);
        profile.Add<Tonemapping>().mode.Override(TonemappingMode.None);

        var volume = renderObject.AddComponent<Volume>();
        volume.isGlobal = true;
        volume.priority = 100;
        volume.sharedProfile = profile;

        renderObject.SetActive(true);
    }

    void FinishRender(ScriptableRenderContext context, Camera camera)
    {
        if (camera != bloomCamera) return;
        bloomCamera.enabled = false;
        foreach (var graphic in graphics)
        {
            graphic.texture = output;
            graphic.material = composite;
        }
    }

    void OnDestroy()
    {
        if (output)
        {
            output.Release();
            Destroy(output);
        }
        Destroy(composite);
        if (profile)
        {
            foreach (var effect in profile.components) Destroy(effect);
            Destroy(profile);
        }
    }
}
