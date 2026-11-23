using System;
using System.IO;
using UnityEditor;
using UnityEditor.Build.Reporting;
using UnityEditor.SceneManagement;
using UnityEngine;
using UnityEngine.Rendering;
using UnityEngine.SceneManagement;

public static class WaterLabBuild
{
    private const string ScenePath = "Assets/Water Laboratory.unity";
    private const string Materials = "Assets/Materials";

    [MenuItem("Mira/Build Water Laboratory")]
    public static void Build()
    {
        if (AssetDatabase.LoadAssetAtPath<SceneAsset>(ScenePath) == null)
            throw new InvalidOperationException("Missing saved laboratory scene: " + ScenePath);
        if (SceneManager.GetSceneByPath(ScenePath).isDirty)
            throw new InvalidOperationException("Save the laboratory scene before building.");

        PlayerSettings.companyName = "Mira";
        PlayerSettings.productName = "Water Laboratory · Mira";
        PlayerSettings.defaultScreenWidth = 1280;
        PlayerSettings.defaultScreenHeight = 820;
        PlayerSettings.fullScreenMode = FullScreenMode.Windowed;
        PlayerSettings.resizableWindow = true;
        PlayerSettings.SplashScreen.show = false;
        EditorBuildSettings.scenes = new[] { new EditorBuildSettingsScene(ScenePath, true) };

        string output = Environment.GetEnvironmentVariable("MIRA_WATER_LAB_OUTPUT");
        string folder = Path.GetFullPath(output ?? Path.Combine(
            Application.dataPath, "..", "..", "..", "target", "unity-water-lab-builtin"));
        Directory.CreateDirectory(folder);
        BuildReport result = BuildPipeline.BuildPlayer(new BuildPlayerOptions
        {
            scenes = new[] { ScenePath },
            locationPathName = Path.Combine(folder, "WaterLab.exe"),
            target = BuildTarget.StandaloneWindows64
        });
        if (result.summary.result != BuildResult.Succeeded)
            throw new InvalidOperationException("Water laboratory build failed: " + result.summary.result);
        Debug.Log("MIRA WATER LAB BUILD OK: " + result.summary.outputPath);
    }

    [MenuItem("Mira/Regenerate Water Laboratory Scene")]
    public static void RegenerateScene()
    {
        if (!Application.isBatchMode && (!EditorUtility.DisplayDialog("Regenerate Water Laboratory",
                "Replace the laboratory scene and reset its generated materials?", "Replace", "Cancel") ||
            !EditorSceneManager.SaveCurrentModifiedScenesIfUserWantsTo()))
            return;

        var scene = EditorSceneManager.NewScene(NewSceneSetup.EmptyScene, NewSceneMode.Single);
        if (!AssetDatabase.IsValidFolder(Materials))
            AssetDatabase.CreateFolder("Assets", "Materials");

        Vector3 lightEuler = new(38f, 223f, 0f);
        Vector3 sunDirection = -(Quaternion.Euler(lightEuler) * Vector3.forward);
        Color horizon = new Color32(154, 184, 185, 255);
        Color zenith = new Color32(43, 89, 121, 255);
        Material sky = MaterialAsset("Sky", "Mira/Water Lab Sky");
        Material water = MaterialAsset("Water", "Mira/Water Laboratory");
        foreach (Material material in new[] { sky, water })
        {
            material.SetColor("_Horizon", horizon);
            material.SetColor("_Zenith", zenith);
            material.SetVector("_Sun", sunDirection);
            EditorUtility.SetDirty(material);
        }
        RenderSettings.skybox = sky;
        RenderSettings.ambientMode = AmbientMode.Trilight;
        RenderSettings.ambientSkyColor = (Color)new Color32(126, 157, 175, 255) * 0.75f;
        RenderSettings.ambientEquatorColor = (Color)new Color32(119, 144, 148, 255) * 0.62f;
        RenderSettings.ambientGroundColor = (Color)new Color32(91, 103, 99, 255) * 0.65f;
        var sun = new GameObject("Sun").AddComponent<Light>();
        sun.type = LightType.Directional;
        sun.color = new Color32(255, 233, 207, 255);
        sun.intensity = 1.1f;
        sun.transform.rotation = Quaternion.Euler(lightEuler);
        sun.shadows = LightShadows.Soft;
        RenderSettings.sun = sun;

        Material silt = Surface("Silt", new Color32(142, 157, 149, 255), 0.1f);
        Material pale = Surface("Pale stone", new Color32(197, 183, 145, 255), 0.24f);
        Material dark = Surface("Dark stone", new Color32(75, 109, 114, 255), 0.27f);
        Material marker = Surface("Buoy brass", new Color32(200, 122, 62, 255), 0.57f);
        Material pearl = Surface("Test sphere", new Color32(237, 177, 87, 255), 0.67f);

        Piece(PrimitiveType.Cube, "Bottom", new(0f, -2.45f, 0f), new(420f, 0.45f, 420f), silt);
        for (int i = 0; i < 7; i++)
        {
            float depth = -1.35f - i * 0.12f;
            Piece(PrimitiveType.Cube, "Depth reference " + (i + 1), new(-2.70f + i * 0.88f, depth, -2.9f),
                new(0.74f, 0.16f, 0.74f), i % 2 == 0 ? pale : dark);
        }

        for (int i = 0; i < 3; i++)
        {
            float x = -4.0f + 3.8f * i;
            float z = 4.30f + 0.85f * (i % 2);
            Piece(PrimitiveType.Cylinder, "Sounding post", new(x, -0.48f, z), new(0.29f, 0.97f, 0.29f), dark);
            Piece(PrimitiveType.Cylinder, "Brass cap", new(x, 0.465f, z), new(0.31f, 0.041f, 0.31f), marker);
        }
        Piece(PrimitiveType.Cube, "Shallow shelf", new(3.9f, -1.13f, -0.7f), new(2.85f, 0.65f, 2.2f), pale)
            .transform.rotation = Quaternion.Euler(0f, -17f, 0f);
        Piece(PrimitiveType.Cube, "Shelf ledge", new(4.04f, -1.60f, -0.4f), new(3.3f, 0.42f, 2.66f), dark)
            .transform.rotation = Quaternion.Euler(0f, -17f, 0f);
        var referenceBed = Array.FindAll(scene.GetRootGameObjects(), piece => piece.GetComponent<MeshRenderer>() != null);

        var gameSphere = Piece(PrimitiveType.Sphere, "Test sphere", new(0.35f, 2.5f, 0.35f), Vector3.one * 0.64f, pearl);
        gameSphere.AddComponent<SphereCollider>().radius = 0.5f;
        var sphere = gameSphere.AddComponent<Rigidbody>();
        sphere.isKinematic = true;
        sphere.interpolation = RigidbodyInterpolation.Interpolate;

        var view = new GameObject("Laboratory camera").AddComponent<Camera>();
        view.tag = "MainCamera";
        view.fieldOfView = 53f;
        view.farClipPlane = 175f;
        view.nearClipPlane = 0.10f;
        view.clearFlags = CameraClearFlags.Skybox;
        view.depthTextureMode = DepthTextureMode.Depth;
        var lab = new GameObject("Water laboratory").AddComponent<WaterLaboratory>();
        lab.view = view;
        lab.sphere = sphere;
        lab.referenceBedObjects = referenceBed;
        lab.waterMaterial = water;
        lab.rainMaterial = MaterialAsset("Rain", "Unlit/Color");
        lab.rainMaterial.color = new Color32(164, 204, 219, 255);
        EditorUtility.SetDirty(lab.rainMaterial);
        if (!EditorSceneManager.SaveScene(scene, ScenePath))
            throw new IOException("Could not save the regenerated laboratory scene: " + ScenePath);
        AssetDatabase.SaveAssets();
    }

    private static GameObject Piece(PrimitiveType kind, string name, Vector3 position, Vector3 scale, Material material)
    {
        var piece = GameObject.CreatePrimitive(kind);
        piece.name = name;
        piece.transform.SetPositionAndRotation(position, Quaternion.identity);
        piece.transform.localScale = scale;
        piece.GetComponent<Renderer>().sharedMaterial = material;
        UnityEngine.Object.DestroyImmediate(piece.GetComponent<Collider>());
        return piece;
    }

    private static Material Surface(string name, Color color, float gloss)
    {
        Material material = MaterialAsset(name, "Standard");
        material.color = color;
        material.SetFloat("_Glossiness", gloss);
        EditorUtility.SetDirty(material);
        return material;
    }

    private static Material MaterialAsset(string name, string shaderName)
    {
        string path = $"{Materials}/{name}.mat";
        Material material = AssetDatabase.LoadAssetAtPath<Material>(path);
        if (material != null)
            return material;
        Shader shader = Shader.Find(shaderName);
        if (shader == null)
            throw new InvalidOperationException("Missing shader " + shaderName);
        material = new Material(shader);
        AssetDatabase.CreateAsset(material, path);
        return material;
    }
}
