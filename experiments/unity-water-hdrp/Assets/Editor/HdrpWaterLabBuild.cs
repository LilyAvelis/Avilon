using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Security.Cryptography;
using System.Text;
using UnityEditor;
using UnityEditor.Build.Reporting;
using UnityEditor.SceneManagement;
using UnityEngine;
using UnityEngine.Rendering;
using UnityEngine.Rendering.HighDefinition;

public static class HdrpWaterLabBuild
{
    private const string SourceScene = "Assets/Shared/Water Laboratory.unity";
    public const string Scene = "Assets/MiraHdrpGenerated/Water Laboratory HDRP.unity";
    private const string Generated = "Assets/MiraHdrpGenerated";
    private const string PreparationStamp = "Library/MiraWaterPreparation.sha256";

    [MenuItem("Mira/Prepare HDRP Water Laboratory")]
    public static void Prepare()
    {
        VerifySharedAssets();
        Directory.CreateDirectory(Generated);
        AssetDatabase.Refresh();
        PlayerSettings.colorSpace = ColorSpace.Linear;
        PlayerSettings.SetUseDefaultGraphicsAPIs(BuildTarget.StandaloneWindows64, false);
        PlayerSettings.SetGraphicsAPIs(BuildTarget.StandaloneWindows64, new[] { GraphicsDeviceType.Direct3D12 });
        var pipeline = ConfigurePipeline();
        var scene = EditorSceneManager.OpenScene(SourceScene);
        var lab = UnityEngine.Object.FindFirstObjectByType<WaterLaboratory>();
        if (lab == null || lab.referenceBedObjects == null || lab.referenceBedObjects.Length == 0 ||
            Array.Exists(lab.referenceBedObjects, piece => piece == null))
            throw new InvalidOperationException("The source scene needs valid reference bed links.");
        lab.hdrpPipeline = pipeline;
        ConfigureCamera(lab.view);
        Light sun = RenderSettings.sun;
        float originalIntensity = sun.intensity;
        if (!sun.TryGetComponent<HDAdditionalLightData>(out _))
            sun.gameObject.AddComponent<HDAdditionalLightData>();
        // The HDRP camera uses fixed exposure for matching shots. Calibrate the sun to it.
        sun.lightUnit = LightUnit.Lux;
        sun.intensity = originalIntensity;
        lab.hdrpSceneMaterials = PrepareSceneMaterials(scene);
        lab.hdrpEnvironment = CreateEnvironment();

        var water = new GameObject("HDRP water surface").AddComponent<WaterSurface>();
        water.surfaceType = WaterSurfaceType.OceanSeaLake;
        water.geometryType = WaterGeometryType.Infinite;
        water.ripplesMotionMode = WaterPropertyOverrideMode.Custom;
        water.scriptInteractions = false;
        water.deformation = true;
        water.deformationRes = WaterSurface.WaterDecalRegionResolution.Resolution2048;
        water.decalRegionSize = Vector2.one * (2f * WaterLaboratory.InteractionRadius);
        water.decalRegionAnchor = water.transform;
        water.causticsResolution = WaterSurface.WaterCausticsResolution.Caustics512;
        water.causticsIntensity = 0.35f;
        water.causticsBand = 2;
        water.underWater = true;
        water.startSmoothness = 0.99f;
        water.endSmoothness = 0.92f;
        lab.hdrpSurface = water;

        var impactDecal = new GameObject("Mira GPU wave height").AddComponent<WaterDecal>();
        impactDecal.regionSize = water.decalRegionSize;
        impactDecal.resolution = new Vector2Int(2048, 2048);
        impactDecal.amplitude = 1f;
        impactDecal.updateMode = CustomRenderTextureUpdateMode.OnDemand;
        impactDecal.material = MaterialAsset("GPU deformation", "Mira/HDRP Wave Decal");
        lab.hdrpImpacts = impactDecal;
        lab.hdrpSurface.gameObject.SetActive(false);
        lab.hdrpImpacts.gameObject.SetActive(false);
        lab.hdrpEnvironment.gameObject.SetActive(false);

        if (!EditorSceneManager.SaveScene(scene, Scene))
            throw new InvalidOperationException("Could not save the HDRP laboratory.");
        ConfigurePlayer();
        AssetDatabase.SaveAssets();
        File.WriteAllText(PreparationStamp, BuildFingerprint());
        Debug.Log("MIRA HDRP WATER SCENE READY");
    }

    private static string FileDigest(string path)
    {
        using var sha = SHA256.Create();
        using var input = File.OpenRead(path);
        return Convert.ToBase64String(sha.ComputeHash(input));
    }

    private static string[] RelativeFiles(string folder) => Directory.GetFiles(folder, "*", SearchOption.AllDirectories)
        .Select(path => Path.GetRelativePath(folder, path).Replace('\\', '/'))
        .OrderBy(path => path, StringComparer.Ordinal).ToArray();

    private static void VerifySharedAssets()
    {
        const string source = "../unity-water-lab/Assets";
        const string shared = "Assets/Shared";
        var expected = RelativeFiles(source).Where(path => path != "Editor.meta" &&
            !path.StartsWith("Editor/", StringComparison.Ordinal));
        var actual = RelativeFiles(shared);
        if (!expected.SequenceEqual(actual) ||
            actual.Any(path => FileDigest(Path.Combine(source, path)) != FileDigest(Path.Combine(shared, path))))
            throw new InvalidOperationException("Shared water assets are stale. Run sync-shared.ps1 and Prepare.");
    }

    private static string BuildFingerprint()
    {
        var inputs = RelativeFiles("Assets").Select(path => "Assets/" + path)
            .Concat(new[]
            {
                "Packages/manifest.json", "Packages/packages-lock.json", "ProjectSettings/ProjectVersion.txt",
                "ProjectSettings/GraphicsSettings.asset", "ProjectSettings/QualitySettings.asset",
                "ProjectSettings/TimeManager.asset", "ProjectSettings/ProjectSettings.asset"
            });
        using var sha = SHA256.Create();
        var manifest = string.Join("\n", inputs.Select(path => path + " " + FileDigest(path)));
        return Convert.ToBase64String(sha.ComputeHash(Encoding.UTF8.GetBytes(manifest)));
    }

    private static HDRenderPipelineAsset ConfigurePipeline()
    {
        var path = Generated + "/Mira Water HDRP.asset";
        var pipeline = AssetDatabase.LoadAssetAtPath<HDRenderPipelineAsset>(path);
        if (pipeline == null)
        {
            pipeline = ScriptableObject.CreateInstance<HDRenderPipelineAsset>();
            AssetDatabase.CreateAsset(pipeline, path);
        }
        var rendering = pipeline.currentPlatformRenderPipelineSettings;
        rendering.supportWater = true;
        rendering.supportWaterDecals = true;
        rendering.waterDecalAtlasSize = WaterAtlasSize.AtlasSize2048;
        rendering.supportSSR = true;
        rendering.supportSSRTransparent = true;
        pipeline.currentPlatformRenderPipelineSettings = rendering;
        EditorUtility.SetDirty(pipeline);
        // HDRP's editor-side bootstrap populates package resources in Graphics Settings.
        var globals = typeof(HDRenderPipeline).Assembly.GetType(
            "UnityEngine.Rendering.HighDefinition.HDRenderPipelineGlobalSettings");
        global::UnityEngine.Rendering.GraphicsSettings.defaultRenderPipeline = pipeline;
        globals.GetMethod("Ensure", BindingFlags.NonPublic | BindingFlags.Static).Invoke(null, new object[] { true });
        QualitySettings.renderPipeline = null;
        return pipeline;
    }

    private static void ConfigureCamera(Camera view)
    {
        var data = view.GetComponent<HDAdditionalCameraData>() ?? view.gameObject.AddComponent<HDAdditionalCameraData>();
        data.antialiasing = HDAdditionalCameraData.AntialiasingMode.SubpixelMorphologicalAntiAliasing;
        data.customRenderingSettings = true;
        foreach (FrameSettingsField field in new[] { FrameSettingsField.Water, FrameSettingsField.WaterDecals,
            FrameSettingsField.Refraction, FrameSettingsField.SSR, FrameSettingsField.TransparentSSR })
        {
            data.renderingPathCustomFrameSettings.SetEnabled(field, true);
            data.renderingPathCustomFrameSettingsOverrideMask.mask[(uint)field] = true;
        }
    }

    private static Volume CreateEnvironment()
    {
        string path = Generated + "/Water Environment.asset";
        var profile = AssetDatabase.LoadAssetAtPath<VolumeProfile>(path);
        if (profile == null)
        {
            profile = ScriptableObject.CreateInstance<VolumeProfile>();
            AssetDatabase.CreateAsset(profile, path);
        }
        Component<VisualEnvironment>(profile).skyType.Override((int)SkyType.Gradient);
        var sky = Component<GradientSky>(profile);
        sky.top.Override(new Color(.17f, .32f, .48f));
        sky.middle.Override(new Color(.60f, .72f, .72f));
        sky.bottom.Override(new Color(.34f, .46f, .49f));
        sky.gradientDiffusion.Override(1.35f);
        var fog = Component<Fog>(profile);
        fog.enabled.Override(true);
        fog.meanFreePath.Override(47f);
        fog.maxFogDistance.Override(250f);
        var exposure = Component<Exposure>(profile);
        exposure.mode.Override(ExposureMode.Fixed);
        exposure.fixedExposure.Override(0f);
        Component<Bloom>(profile).intensity.Override(0f);
        Component<Tonemapping>(profile).mode.Override(TonemappingMode.Neutral);
        Component<WaterRendering>(profile).enable.Override(true);
        var reflections = Component<ScreenSpaceReflection>(profile);
        reflections.enabled.Override(true);
        reflections.enabledTransparent.Override(true);
        EditorUtility.SetDirty(profile);

        var volume = new GameObject("HDRP sky, fog and water").AddComponent<Volume>();
        volume.sharedProfile = profile;
        volume.isGlobal = true;
        return volume;
    }

    private static T Component<T>(VolumeProfile profile) where T : VolumeComponent
    {
        if (profile.TryGet<T>(out var existing))
            return existing;
        T volumeComponent = profile.Add<T>();
        AssetDatabase.AddObjectToAsset(volumeComponent, profile);
        return volumeComponent;
    }

    private static WaterLaboratory.SceneMaterialPair[] PrepareSceneMaterials(UnityEngine.SceneManagement.Scene scene)
    {
        var materials = new List<WaterLaboratory.SceneMaterialPair>();
        foreach (GameObject root in scene.GetRootGameObjects())
        foreach (MeshRenderer renderer in root.GetComponentsInChildren<MeshRenderer>(true))
        {
            Material source = renderer.sharedMaterial;
            if (source == null || source.shader.name != "Standard")
                continue;
            Material replacement = MaterialAsset(source.name, "HDRP/Lit");
            replacement.SetColor("_BaseColor", source.color);
            replacement.SetFloat("_Smoothness", source.GetFloat("_Glossiness"));
            replacement.SetFloat("_Metallic", source.GetFloat("_Metallic"));
            HDMaterial.ValidateMaterial(replacement);
            materials.Add(new WaterLaboratory.SceneMaterialPair { renderer = renderer, builtin = source, hdrp = replacement });
            EditorUtility.SetDirty(replacement);
        }
        return materials.ToArray();
    }

    private static Material MaterialAsset(string name, string shaderName)
    {
        string path = Generated + "/" + name + ".mat";
        var existing = AssetDatabase.LoadAssetAtPath<Material>(path);
        if (existing != null)
            return existing;
        Shader shader = Shader.Find(shaderName);
        if (shader == null)
            throw new InvalidOperationException($"Missing shader {shaderName}.");
        var material = new Material(shader);
        AssetDatabase.CreateAsset(material, path);
        return material;
    }

    private static void ConfigurePlayer()
    {
        PlayerSettings.companyName = "Mira";
        PlayerSettings.productName = "Water Laboratory · Mira";
        PlayerSettings.bundleVersion = "1.11";
        PlayerSettings.defaultScreenWidth = 1280;
        PlayerSettings.defaultScreenHeight = 820;
        PlayerSettings.fullScreenMode = FullScreenMode.Windowed;
        PlayerSettings.resizableWindow = true;
        PlayerSettings.SplashScreen.show = false;
    }

    public static void PrepareAndBuild()
    {
        Prepare();
        Build();
    }

    [MenuItem("Mira/Build HDRP Water Laboratory")]
    public static void Build()
    {
        if (AssetDatabase.LoadAssetAtPath<SceneAsset>(Scene) == null)
            throw new InvalidOperationException("Prepare the HDRP laboratory first.");
        VerifySharedAssets();
        string fingerprint = BuildFingerprint();
        if (!File.Exists(PreparationStamp) || File.ReadAllText(PreparationStamp) != fingerprint)
            throw new InvalidOperationException("The HDRP preparation does not match its inputs. Run Prepare.");
        string output = Environment.GetEnvironmentVariable("MIRA_WATER_HDRP_OUTPUT")
            ?? Path.GetFullPath("../../target/unity-water-lab-combined");
        Directory.CreateDirectory(output);
        var report = BuildPipeline.BuildPlayer(new BuildPlayerOptions
        {
            scenes = new[] { Scene },
            locationPathName = Path.Combine(output, "WaterLab.exe"),
            target = BuildTarget.StandaloneWindows64
        });
        if (report.summary.result != BuildResult.Succeeded)
            throw new InvalidOperationException("HDRP laboratory build failed.");
        File.WriteAllText(Path.Combine(output, "water-lab-inputs.sha256"), fingerprint);
        Debug.Log("MIRA HDRP WATER PLAYER OK " + report.summary.outputPath);
    }
}
