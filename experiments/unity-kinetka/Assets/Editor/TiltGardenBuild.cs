using System;
using System.IO;
using UnityEditor;
using UnityEditor.Build;
using UnityEditor.Build.Reporting;
using UnityEditor.SceneManagement;
using UnityEngine;
using UnityEngine.Rendering;

public static class TiltGardenBuild
{
    private const string ScenePath = "Assets/Tilt Garden.unity";
    private const string ArtPath = "Assets/Garden Materials";
    private const string NaturePath = "Assets/Kenney Nature Kit";

    public static void AssembleAndBuild()
    {
        AssembleScene();
        PlayerSettings.companyName = "Mira";
        PlayerSettings.productName = "Наклонный сад · Мира";
        PlayerSettings.defaultScreenWidth = 1280;
        PlayerSettings.defaultScreenHeight = 820;
        PlayerSettings.fullScreenMode = FullScreenMode.Windowed;
        PlayerSettings.resizableWindow = true;
        PlayerSettings.runInBackground = false;
        PlayerSettings.SplashScreen.show = false;
        PlayerSettings.SetScriptingBackend(NamedBuildTarget.Standalone, ScriptingImplementation.Mono2x);
        EditorBuildSettings.scenes = new[] { new EditorBuildSettingsScene(ScenePath, true) };
        AssetDatabase.SaveAssets();

        string outputFolder = Path.GetFullPath(Path.Combine(Application.dataPath, "..", "..", "..", "target", "unity-kinetka-v2"));
        Directory.CreateDirectory(outputFolder);
        BuildReport report = BuildPipeline.BuildPlayer(new BuildPlayerOptions
        {
            scenes = new[] { ScenePath },
            locationPathName = Path.Combine(outputFolder, "NaklonnySad.exe"),
            target = BuildTarget.StandaloneWindows64,
            options = BuildOptions.None
        });
        if (report.summary.result != BuildResult.Succeeded)
            throw new InvalidOperationException($"Windows build failed: {report.summary.result}");
        Debug.Log($"Tilt Garden built at {report.summary.outputPath}");
    }

    private static void AssembleScene()
    {
        var scene = EditorSceneManager.NewScene(NewSceneSetup.EmptyScene, NewSceneMode.Single);
        if (!AssetDatabase.IsValidFolder(ArtPath))
            AssetDatabase.CreateFolder("Assets", "Garden Materials");

        var body = Surface("Dark green enamel", "#1A3A32", 0.25f);
        var deck = Surface("Sand", "#C7A777", 0.18f);
        var rim = Surface("Cedar rim", "#755941", 0.20f);
        var rimCap = Surface("Cedar trim", "#AA7951", 0.24f);
        var edge = Surface("Brass detail", "#C99454", 0.35f, 0.45f);
        var leaf = Surface("Garden beds", "#566E55", 0.10f);
        var moss = Surface("Dark moss", "#304F40", 0.12f);
        var ball = Surface("Brass marble", "#F4C065", 0.48f, 0.42f);
        var socket = Surface("Socket", "#28534D", 0.28f);
        var mint = Surface("Light ring", "#75D9B9", 0.33f);
        mint.EnableKeyword("_EMISSION");
        mint.SetColor("_EmissionColor", new Color(0.04f, 0.16f, 0.12f));
        EditorUtility.SetDirty(mint);

        var stage = new GameObject("Tilting tray");
        var rigidTray = stage.AddComponent<Rigidbody>();
        rigidTray.isKinematic = true;
        rigidTray.interpolation = RigidbodyInterpolation.Interpolate;
        rigidTray.collisionDetectionMode = CollisionDetectionMode.ContinuousSpeculative;
        Transform tray = stage.transform;
        Background();
        Piece(PrimitiveType.Cube, "Plinth", tray, new Vector3(0, -0.27f, 0),
            new Vector3(9.0f, 0.47f, 5.62f), body);
        Piece(PrimitiveType.Cube, "Inset", tray, new Vector3(0, -0.116f, 0),
            new Vector3(8.7f, 0.232f, 5.24f), deck, true);
        Piece(PrimitiveType.Cube, "Brass seam", tray, new Vector3(0, -0.284f, 0),
            new Vector3(9.03f, 0.035f, 5.65f), edge);

        foreach (int side in new[] { -1, 1 })
        {
            Piece(PrimitiveType.Cube, "Long rail", tray, new Vector3(0, 0.105f, side * 2.67f),
                new Vector3(9.0f, 0.27f, 0.19f), rim);
            Piece(PrimitiveType.Cube, "Short rail", tray, new Vector3(side * 4.39f, 0.105f, 0),
                new Vector3(0.19f, 0.27f, 5.19f), rim);
            GuardRail(tray, new Vector3(0, 0.42f, side * 2.67f), new Vector3(9.04f, 1.02f, 0.19f));
            GuardRail(tray, new Vector3(side * 4.39f, 0.42f, 0), new Vector3(0.19f, 1.02f, 5.53f));
            Piece(PrimitiveType.Cube, "Long trim", tray, new Vector3(0, 0.246f, side * 2.67f),
                new Vector3(9.0f, 0.033f, 0.195f), rimCap);
            Piece(PrimitiveType.Cube, "Short trim", tray, new Vector3(side * 4.39f, 0.246f, 0),
                new Vector3(0.195f, 0.033f, 5.19f), rimCap);
            foreach (int end in new[] { -1, 1 })
                Piece(PrimitiveType.Cylinder, "Corner stud", tray,
                    new Vector3(side * 4.38f, 0.272f, end * 2.67f), new Vector3(0.077f, 0.026f, 0.077f), edge);
        }

        Piece(PrimitiveType.Cylinder, "Pine bed", tray, new Vector3(-2.85f, 0.015f, 1.54f),
            new Vector3(2.52f, 0.035f, 1.65f), leaf);
        Piece(PrimitiveType.Cylinder, "Stone bed", tray, new Vector3(1.56f, 0.017f, -1.63f),
            new Vector3(2.70f, 0.040f, 1.37f), moss);
        Piece(PrimitiveType.Cylinder, "Small bed", tray, new Vector3(-0.48f, 0.016f, 1.76f),
            new Vector3(1.32f, 0.036f, 1.25f), leaf);
        var guide = Piece(PrimitiveType.Cube, "Short guide", tray, new Vector3(-1.75f, 0.078f, 0.86f),
            new Vector3(1.50f, 0.17f, 0.15f), rim, true);
        guide.transform.localRotation = Quaternion.Euler(0f, 18f, 0f);
        guide = Piece(PrimitiveType.Cube, "Stone guide", tray, new Vector3(0.90f, 0.083f, -0.82f),
            new Vector3(1.40f, 0.17f, 0.16f), rim, true);
        guide.transform.localRotation = Quaternion.Euler(0f, -14f, 0f);

        Plant("tree_pineDefaultA", tray, -3.73f, 1.59f, 1.86f, 20);
        Plant("tree_pineDefaultB", tray, -3.02f, 1.96f, 1.48f, -28);
        Plant("tree_pineRoundC", tray, -2.37f, 1.86f, 1.19f, 46);
        Plant("tree_pineDefaultB", tray, -0.55f, 1.98f, 1.32f, 72);
        Plant("rock_largeB", tray, 1.32f, -1.76f, 0.49f, 25);
        Plant("rock_smallC", tray, 2.17f, -1.73f, 0.23f, 76);
        Plant("rock_smallC", tray, -3.11f, 1.16f, 0.23f, -17);
        Plant("plant_bushSmall", tray, -2.47f, 1.31f, 0.33f, 108);
        Plant("plant_bushSmall", tray, 2.50f, -1.61f, 0.35f, -43);
        Plant("flower_yellowA", tray, -0.73f, 1.45f, 0.33f, 43);
        Plant("flower_yellowA", tray, 1.89f, -1.25f, 0.32f, 23);
        Plant("grass_large", tray, -3.86f, 1.17f, 0.31f, 55);
        Plant("grass_large", tray, 0.58f, -1.80f, 0.28f, 78);
        Plant("mushroom_redGroup", tray, -2.84f, 1.19f, 0.25f, 44);

        const float marbleRadius = 0.246f;
        Vector3 startPoint = new(-3.34f, marbleRadius + 0.02f, -1.49f);
        Vector3 goalPoint = new(2.94f, marbleRadius + 0.03f, 1.30f);
        Transform start = Marker("Start", tray, startPoint);
        Transform finish = Marker("Goal", tray, goalPoint);
        Piece(PrimitiveType.Cylinder, "Launch mark", tray, new Vector3(startPoint.x, 0.009f, startPoint.z),
            new Vector3(0.83f, 0.022f, 0.83f), rimCap);
        Piece(PrimitiveType.Cylinder, "Catching bowl", tray, new Vector3(goalPoint.x, 0.027f, goalPoint.z),
            new Vector3(1.37f, 0.052f, 1.37f), socket);
        Renderer ring = Ring(tray, new Vector3(goalPoint.x, 0.082f, goalPoint.z), mint);
        var marbleObject = Piece(PrimitiveType.Sphere, "Brass marble", null, startPoint,
            Vector3.one * (marbleRadius * 2f), ball, true);
        var rigidMarble = marbleObject.AddComponent<Rigidbody>();
        rigidMarble.mass = 0.65f;
        rigidMarble.linearDamping = 0.48f;
        rigidMarble.angularDamping = 0.09f;
        rigidMarble.interpolation = RigidbodyInterpolation.Interpolate;
        rigidMarble.collisionDetectionMode = CollisionDetectionMode.ContinuousDynamic;
        rigidMarble.maxAngularVelocity = 35f;

        var cameraObject = new GameObject("Camera");
        var camera = cameraObject.AddComponent<Camera>();
        cameraObject.tag = "MainCamera";
        camera.orthographic = true;
        camera.orthographicSize = 4.45f;
        camera.backgroundColor = Hex("#142F2D");
        camera.clearFlags = CameraClearFlags.Skybox;
        camera.transform.position = new Vector3(7.8f, 9.7f, -10.7f);
        camera.transform.LookAt(new Vector3(-0.53f, 0.95f, 0.35f));
        cameraObject.AddComponent<AudioListener>();
        QualitySettings.shadows = ShadowQuality.All;
        QualitySettings.shadowDistance = 45;
        RenderSettings.ambientMode = AmbientMode.Trilight;
        RenderSettings.ambientSkyColor = Hex("#C6DDD0") * 0.63f;
        RenderSettings.ambientEquatorColor = Hex("#8DA9A1") * 0.67f;
        RenderSettings.ambientGroundColor = Hex("#4F6C5F") * 0.55f;
        Lamp("Late afternoon", Hex("#FFE3B3"), 1.14f, new Vector3(44f, -25f, 0f), LightShadows.Soft);
        Lamp("Sky fill", Hex("#B4DACF"), 0.30f, new Vector3(34f, 149f, 0f), LightShadows.None);

        var rig = new GameObject("Garden controls");
        var controller = rig.AddComponent<TiltGarden>();
        controller.tray = rigidTray;
        controller.marble = rigidMarble;
        controller.start = start;
        controller.goal = finish;
        controller.goalRing = ring;
        controller.view = camera;
        controller.sound = rig.AddComponent<AudioSource>();
        EditorSceneManager.SaveScene(scene, ScenePath);
    }

    private static Color Hex(string hex)
    {
        ColorUtility.TryParseHtmlString(hex, out Color color);
        return color;
    }

    private static void Background()
    {
        Material sky = MaterialAsset("Garden sky", "Mira/Garden Sky");
        sky.SetColor("_Top", Hex("#53766C"));
        sky.SetColor("_Bottom", Hex("#243C39"));
        EditorUtility.SetDirty(sky);
        RenderSettings.skybox = sky;

        Material shadowInk = MaterialAsset("Soft tray shadow", "Mira/Soft Tray Shadow");
        var shadow = Piece(PrimitiveType.Quad, "Soft shadow", null, new Vector3(0.25f, -1.83f, 0.14f),
            new Vector3(10.8f, 7.3f, 1f), shadowInk);
        shadow.transform.rotation = Quaternion.Euler(90f, 0f, 0f);
        var renderer = shadow.GetComponent<Renderer>();
        renderer.shadowCastingMode = ShadowCastingMode.Off;
        renderer.receiveShadows = false;
    }

    private static void GuardRail(Transform tray, Vector3 position, Vector3 size)
    {
        Transform rail = Marker("Rail collider", tray, position);
        rail.gameObject.AddComponent<BoxCollider>().size = size;
    }

    private static Material Surface(string name, string hex, float smoothness, float metallic = 0f)
    {
        Material material = MaterialAsset(name, "Standard");
        material.color = Hex(hex);
        material.SetFloat("_Glossiness", smoothness);
        material.SetFloat("_Metallic", metallic);
        EditorUtility.SetDirty(material);
        return material;
    }

    private static Material MaterialAsset(string name, string shaderName)
    {
        string path = $"{ArtPath}/{name}.mat";
        Material material = AssetDatabase.LoadAssetAtPath<Material>(path);
        if (material == null)
        {
            Shader shader = Shader.Find(shaderName);
            if (shader == null)
                throw new InvalidOperationException($"Shader is missing: {shaderName}");
            material = new Material(shader);
            AssetDatabase.CreateAsset(material, path);
        }
        return material;
    }

    private static Transform Marker(string name, Transform parent, Vector3 position)
    {
        var marker = new GameObject(name).transform;
        marker.SetParent(parent, false);
        marker.localPosition = position;
        return marker;
    }

    private static GameObject Piece(PrimitiveType kind, string name, Transform parent,
        Vector3 position, Vector3 scale, Material material, bool collision = false)
    {
        GameObject piece = GameObject.CreatePrimitive(kind);
        piece.name = name;
        piece.transform.SetParent(parent, false);
        piece.transform.localPosition = position;
        piece.transform.localScale = scale;
        piece.GetComponent<Renderer>().sharedMaterial = material;
        if (!collision)
            UnityEngine.Object.DestroyImmediate(piece.GetComponent<Collider>());
        return piece;
    }

    private static void Plant(string name, Transform parent, float x, float z, float height, float yaw)
    {
        GameObject prefab = AssetDatabase.LoadAssetAtPath<GameObject>($"{NaturePath}/{name}.fbx");
        if (prefab == null)
            throw new InvalidOperationException($"Nature Kit model is missing: {name}");
        var root = Marker(name, parent, new Vector3(x, 0.055f, z));
        var model = (GameObject)PrefabUtility.InstantiatePrefab(prefab, root);
        model.transform.localPosition = Vector3.zero;
        model.transform.localRotation = Quaternion.Euler(0f, yaw, 0f);
        Renderer[] renderers = model.GetComponentsInChildren<Renderer>();
        Bounds bounds = renderers[0].bounds;
        foreach (Renderer renderer in renderers)
            bounds.Encapsulate(renderer.bounds);
        float factor = height / Mathf.Max(0.001f, bounds.size.y);
        model.transform.localScale *= factor;
        Vector3 center = root.InverseTransformPoint(bounds.center);
        model.transform.localPosition = new Vector3(-center.x * factor,
            (bounds.extents.y - center.y) * factor, -center.z * factor);
    }

    private static Renderer Ring(Transform parent, Vector3 position, Material material)
    {
        const int count = 72;
        var vertices = new Vector3[count * 2];
        var triangles = new int[count * 6];
        for (int i = 0; i < count; i++)
        {
            float angle = 2f * Mathf.PI * i / count;
            Vector3 direction = new(Mathf.Cos(angle), 0f, Mathf.Sin(angle));
            vertices[2 * i] = direction * 0.58f;
            vertices[2 * i + 1] = direction * 0.66f;
            int a = 2 * i;
            int b = 2 * ((i + 1) % count);
            int t = 6 * i;
            triangles[t] = a;
            triangles[t + 1] = b;
            triangles[t + 2] = a + 1;
            triangles[t + 3] = a + 1;
            triangles[t + 4] = b;
            triangles[t + 5] = b + 1;
        }
        var mesh = new Mesh { name = "Circular inlay", vertices = vertices, triangles = triangles };
        mesh.RecalculateNormals();
        const string meshPath = ArtPath + "/Goal Ring.asset";
        var existing = AssetDatabase.LoadAssetAtPath<Mesh>(meshPath);
        if (existing == null)
            AssetDatabase.CreateAsset(mesh, meshPath);
        else
        {
            EditorUtility.CopySerialized(mesh, existing);
            UnityEngine.Object.DestroyImmediate(mesh);
            mesh = existing;
        }
        Transform root = Marker("Goal light", parent, position);
        root.gameObject.AddComponent<MeshFilter>().sharedMesh = mesh;
        var renderer = root.gameObject.AddComponent<MeshRenderer>();
        renderer.sharedMaterial = material;
        return renderer;
    }

    private static void Lamp(string name, Color color, float power, Vector3 euler, LightShadows shadows)
    {
        Light light = new GameObject(name).AddComponent<Light>();
        light.type = LightType.Directional;
        light.color = color;
        light.intensity = power;
        light.shadows = shadows;
        light.transform.rotation = Quaternion.Euler(euler);
        light.shadowStrength = 0.78f;
    }
}
