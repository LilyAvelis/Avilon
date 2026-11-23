using System;
using UnityEngine;
using UnityEngine.Rendering;
using UnityEngine.Rendering.HighDefinition;

public sealed partial class WaterLaboratory
{
    [Serializable]
    public struct SceneMaterialPair
    {
        public Renderer renderer;
        public Material builtin;
        public Material hdrp;
    }

    public HDRenderPipelineAsset hdrpPipeline;
    public SceneMaterialPair[] hdrpSceneMaterials;
    public WaterSurface hdrpSurface;
    public WaterDecal hdrpImpacts;
    public Volume hdrpEnvironment;

    private Material hdrpDecalMaterial;
    private GradientSky hdrpSky;
    private Fog hdrpFog;
    private ScreenSpaceReflection hdrpReflections;
    private WaterLabHdrpRainStreaks hdrpRainStreaks;
    private Shader builtinSeabed, hdrpSeabed;
    private RenderPipelineAsset initialDefaultPipeline, initialQualityPipeline;
    private bool pipelineInitialized;
    private bool hdrpAvailable;
    private bool restoreBuiltinLighting;
    private AmbientMode builtinAmbientMode;
    private SphericalHarmonicsL2 builtinAmbientProbe;
    private Material builtinSkybox;
    private Texture builtinReflection;
    private float builtinReflectionIntensity;
    private bool builtinPhysicalRain;
    private int builtinVSync;
    private bool hdrpActive;
    private bool hdrpWaveTextureDirty;
    private bool hdrpPanelOpen;
    private int lastBuiltinMode;
    private bool hdrpCausticsEnabled = true;
    private bool hdrpReflectionsEnabled = true;

    private void OnEnable()
    {
        initialDefaultPipeline = GraphicsSettings.defaultRenderPipeline;
        initialQualityPipeline = QualitySettings.renderPipeline;
        pipelineInitialized = true;
        RenderPipelineManager.activeRenderPipelineDisposed += RestoreBuiltinLighting;
        QualitySettings.renderPipeline = null;
        GraphicsSettings.defaultRenderPipeline = null;
    }

    private void InitializeHdrp()
    {
        hdrpAvailable = hdrpPipeline != null && hdrpImpacts != null && hdrpSurface != null &&
            hdrpEnvironment != null && hdrpImpacts.material != null && sceneSun != null &&
            hdrpSceneMaterials != null && Array.TrueForAll(hdrpSceneMaterials,
                pair => pair.renderer != null && pair.builtin != null && pair.hdrp != null);
        if (hdrpAvailable)
        {
            VolumeProfile profile = hdrpEnvironment.profile;
            hdrpAvailable = profile.TryGet(out hdrpSky) && profile.TryGet(out hdrpFog) &&
                profile.TryGet(out hdrpReflections);
        }
        hdrpSeabed = Resources.Load<Shader>("Mira/HdrpSeabed");
        if (!hdrpAvailable || hdrpSeabed == null)
        {
            hdrpAvailable = false;
            Debug.LogWarning("HDRP water references are incomplete. The Built-in experiments are available.");
            return;
        }
        hdrpDecalMaterial = new Material(hdrpImpacts.material) { hideFlags = HideFlags.HideAndDontSave };
        hdrpImpacts.material = hdrpDecalMaterial;
        builtinSeabed = seabedMaterial.shader;
        hdrpRainStreaks = new WaterLabHdrpRainStreaks(view, rainMaterial != null ? rainMaterial.color : Color.white);
    }

    private void UpdateRenderingBackend()
    {
        bool next = mode == HdrpMode;
        if (next && !hdrpAvailable)
        {
            SelectMode(lastBuiltinMode);
            return;
        }
        if (hdrpActive == next)
            return;

        ApplyRenderingBackend(next);
    }

    private void ApplyRenderingBackend(bool next)
    {
        if (next)
        {
            CaptureBuiltinLighting();
            builtinPhysicalRain = physicalRain;
            builtinVSync = QualitySettings.vSyncCount;
            hdrpWaveTextureDirty = true;
            physicalRain = true;
            rainStreaks.SetAttached(false);
        }
        else
        {
            physicalRain = builtinPhysicalRain;
            QualitySettings.vSyncCount = builtinVSync;
            restoreBuiltinLighting = true;
        }

        QualitySettings.renderPipeline = next ? hdrpPipeline : null;

        SetBuiltInSkyVisible(!next);
        laboratoryWaterRenderer.enabled = !next;
        hdrpSurface.gameObject.SetActive(next);
        hdrpImpacts.gameObject.SetActive(next && waveField.IsActive);
        hdrpEnvironment.gameObject.SetActive(next);
        foreach (var pair in hdrpSceneMaterials)
            pair.renderer.sharedMaterial = next ? pair.hdrp : pair.builtin;
        seabedMaterial.shader = next ? hdrpSeabed : builtinSeabed;
        seabedMaterial.SetFloat(SeabedProfileProperty, seabedProfileIndex);
        ApplyLighting();
        hdrpActive = next;
        if (!next && restoreBuiltinLighting)
            ApplyBuiltinLightingSnapshot();
    }

    private void LateUpdate()
    {
        if (hdrpActive)
            UpdateHdrp();
        else if (RenderPipelineManager.currentPipeline == null)
        {
            RestoreBuiltinLighting();
            rainStreaks.SetAttached(true);
        }
    }

    private void CaptureBuiltinLighting()
    {
        RestoreBuiltinLighting();
        builtinAmbientMode = RenderSettings.ambientMode;
        builtinAmbientProbe = RenderSettings.ambientProbe;
        builtinSkybox = RenderSettings.skybox;
        builtinReflection = RenderSettings.customReflectionTexture;
        builtinReflectionIntensity = RenderSettings.reflectionIntensity;
    }

    private void RestoreBuiltinLighting()
    {
        if (!restoreBuiltinLighting)
            return;
        restoreBuiltinLighting = false;
        ApplyBuiltinLightingSnapshot();
    }

    private void ApplyBuiltinLightingSnapshot()
    {
        RenderSettings.skybox = builtinSkybox;
        RenderSettings.ambientProbe = builtinAmbientProbe;
        RenderSettings.ambientMode = builtinAmbientMode;
        RenderSettings.customReflectionTexture = builtinReflection;
        RenderSettings.reflectionIntensity = builtinReflectionIntensity;
        ApplyLighting();
    }

    private void UpdateHdrp()
    {
        // Retain display synchronization without letting a high-refresh panel
        // override the lab's frame budget.
        if (builtinVSync > 0 && Application.targetFrameRate > 0)
        {
            double refresh = Screen.currentResolution.refreshRateRatio.value;
            int interval = Mathf.Clamp((int)Math.Ceiling(refresh / Application.targetFrameRate), builtinVSync, 4);
            if (QualitySettings.vSyncCount != interval)
                QualitySettings.vSyncCount = interval;
        }
        float windAngle = Mathf.Repeat(90f - WindHeading, 360f);
        hdrpSurface.largeWindSpeed = windSpeed * 3.6f;
        hdrpSurface.largeOrientationValue = windAngle;
        hdrpSurface.largeBand0Multiplier = Mathf.InverseLerp(0f, 12f, windSpeed) * 0.31f;
        hdrpSurface.largeBand1Multiplier = Mathf.InverseLerp(0f, 12f, windSpeed) * 0.62f;
        hdrpSurface.ripples = windSpeed > 0.001f;
        hdrpSurface.ripplesWindSpeed = Mathf.Min(15f, windSpeed * 3.6f);
        hdrpSurface.ripplesOrientationValue = windAngle;
        hdrpSurface.ripplesChaos = 0.40f;
        // The ball already samples the shared field. Read HDRP swell only while
        // the ball is in play, since script interactions also read back the decal.
        hdrpSurface.scriptInteractions = falling && windSpeed > 0.001f;
        hdrpSurface.caustics = hdrpCausticsEnabled && windSpeed > 0.001f && !poincareOverview;
        hdrpReflections.enabledTransparent.value = hdrpReflectionsEnabled;
        hdrpSurface.maxRefractionDistance = 0.44f;
        hdrpSurface.absorptionDistance = 34f * (Liquids[0].Extinction / Current.Extinction);
        hdrpSurface.refractionColor = Color.Lerp(new Color(.38f, .66f, .74f), Current.Tint, .12f);
        hdrpSurface.scatteringColor = Color.Lerp(new Color(.05f, .15f, .21f), Current.Tint, .25f);

        Color horizon = material.GetColor(HorizonProperty);
        hdrpSky.middle.value = horizon;
        hdrpSky.bottom.value = horizon * .62f;
        hdrpSky.top.value = material.GetColor(ZenithProperty);
        hdrpFog.meanFreePath.value = poincareOverview ? 800f : 47f;
        sceneSun.intensity = 2.2f * material.GetFloat(SunPowerProperty);

        bool active = waveField.IsActive;
        if (hdrpImpacts.gameObject.activeSelf != active)
            hdrpImpacts.gameObject.SetActive(active);
        if (active && hdrpWaveTextureDirty)
        {
            hdrpDecalMaterial.SetTexture("_GpuWaveMap", waveField.HeightTexture);
            hdrpDecalMaterial.SetVector("_GpuWaveGrid", ActiveWaveGrid);
            hdrpImpacts.RequestUpdate();
            hdrpWaveTextureDirty = false;
        }
    }

    private float SampleHdrpHeight(Vector3 position, float localHeight)
    {
        // Query the HDRP swell separately. The lab already knows its own impact height.
        var search = new WaterSearchParameters
        {
            targetPositionWS = position,
            startPositionWS = position,
            error = 0.015f,
            maxIterations = 5,
            includeDeformation = false
        };
        return hdrpSurface.ProjectPointOnWaterSurface(search, out WaterSearchResult result)
            ? localHeight + result.projectedPositionWS.y : localHeight;
    }

    private void OnDisable()
    {
        if (hdrpActive)
            ApplyRenderingBackend(false);
        RestoreBuiltinLighting();
        rainStreaks?.SetAttached(false);
        if (pipelineInitialized)
        {
            pipelineInitialized = false;
            RenderPipelineManager.activeRenderPipelineDisposed -= RestoreBuiltinLighting;
            QualitySettings.renderPipeline = initialQualityPipeline;
            GraphicsSettings.defaultRenderPipeline = initialDefaultPipeline;
        }
    }

    private void DisposeHdrp()
    {
        OnDisable();
        hdrpRainStreaks?.Dispose();
        if (hdrpDecalMaterial != null)
            Destroy(hdrpDecalMaterial);
    }
}
