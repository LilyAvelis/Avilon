using System;
using System.Collections.Generic;
using System.IO;
using UnityEditor;
using UnityEditor.Build;
using UnityEditor.Build.Reporting;
using UnityEditor.SceneManagement;
using UnityEngine;
using UnityEngine.Rendering;
using UnityEngine.SceneManagement;

public static class KinetkaScenes
{
    private const string Art = "Assets/Kinetka Materials";
    private const string SquareScene = "Assets/Clock Square.unity";
    private const string DominoScene = "Assets/Domino Run.unity";

    public static void VerifyAndBuild()
    {
        if (!AssetDatabase.IsValidFolder(Art))
            AssetDatabase.CreateFolder("Assets", "Kinetka Materials");
        Directory.CreateDirectory(OutputFolder);
        AssembleSquare();
        AssembleDomino();
        AssetDatabase.SaveAssets();
        Build(SquareScene, "ClockSquare", "Площадь от рассвета до ночи");
        Build(DominoScene, "DominoRun", "Дорожка домино");
    }

    public static void BuildDomino()
    {
        if (!AssetDatabase.IsValidFolder(Art))
            AssetDatabase.CreateFolder("Assets", "Kinetka Materials");
        Directory.CreateDirectory(OutputFolder);
        AssembleDomino();
        AssetDatabase.SaveAssets();
        Build(DominoScene, "DominoRun", "Дорожка домино");
    }

    private static string OutputFolder => Path.GetFullPath(
        Path.Combine(Application.dataPath, "..", "..", "..", "target", "unity-kinetka"));

    private static void AssembleSquare()
    {
        Scene scene = NewScene();
        var platform = Ink("Square stone", "#8A7370");
        var coping = Ink("Square rose edge", "#B08A7E");
        Material[] pavers = { Ink("Square paver pink", "#C8A393"), Ink("Square paver pale", "#D6B39D"),
            Ink("Square paver grey", "#B99D93"), Ink("Square paver warm", "#C1A28C") };
        var mortar = Ink("Square mortar", "#967D70");
        var trim = Ink("Square limestone", "#D4B99C");
        var bronze = Ink("Square bronze", "#A86E49", 0.49f, 0.37f);
        var water = Ink("Square water", "#337476", 0.83f, 0.17f);
        var glass = Ink("Square window glass", "#244450", 0.63f);
        glass.EnableKeyword("_EMISSION");
        glass.SetColor("_EmissionColor", new Color(0.014f, 0.024f, 0.029f));
        var leaf = Ink("Square trees", "#697963");
        var foliageDark = Ink("Square dark leaves", "#505C48");
        var timber = Ink("Square furniture", "#5E4B54");

        Piece("Plaza plinth", PrimitiveType.Cube, null, new(0, -0.39f, 0), new(10.9f, 0.66f, 6.6f), platform);
        Piece("Plaza lip", PrimitiveType.Cube, null, new(0, -0.09f, 0), new(11.02f, 0.14f, 6.71f), coping);
        Piece("Mortar bed", PrimitiveType.Cube, null, new(0, -0.035f, 0), new(10.8f, 0.07f, 6.46f), mortar);
        Transform stones = Group("Plaza paving");
        for (int row = 0; row < 12; row++)
        for (int col = 0; col < 20; col++)
        {
            int tone = (row * 17 + col * 7 + row * col * 3) % 13;
            Piece("Paver", PrimitiveType.Cube, stones, new(-5.13f + col * 0.54f, -0.011f, -2.97f + row * 0.54f),
                new(0.522f, 0.047f, 0.522f), pavers[tone < 6 ? 0 : tone < 9 ? 1 : tone < 12 ? 2 : 3]);
        }

        var windows = new List<Renderer>();
        House("Bakery", new(-3.48f, 0f, 1.53f), 2.18f, 1.44f, 2.03f,
            Ink("Square peach plaster", "#C88074"), Ink("Square red roof", "#814D56"),
            Ink("Square sage shutters", "#5A736C"), trim, glass, windows);
        House("Clockmaker", new(-0.91f, 0f, 1.66f), 2.26f, 1.55f, 2.55f,
            Ink("Square honey plaster", "#C4A06E"), Ink("Square midnight roof", "#4C4D68"),
            Ink("Square aubergine shutters", "#654D58"), trim, glass, windows);
        House("Corner house", new(1.87f, 0f, 1.76f), 2.55f, 1.63f, 1.75f,
            Ink("Square sage plaster", "#799489"), Ink("Square copper roof", "#945D5D"),
            Ink("Square dusk shutters", "#425F61"), trim, glass, windows);
        Tree(new(-4.53f, 0.04f, -1.17f), 1.64f, "tree_pineRoundC", foliageDark, timber);
        Tree(new(4.04f, 0.04f, 1.38f), 1.92f, "tree_pineRoundC", leaf, timber);
        Tree(new(4.67f, 0.04f, 0.86f), 1.14f, "tree_pineDefaultB", foliageDark, timber);
        Bench(new(-2.79f, 0.02f, -1.56f), 24f, timber, bronze);
        Bench(new(2.76f, 0.02f, -1.33f), -20f, timber, bronze);
        Light[] lanterns = { Lantern(new(-2.10f, 0f, 0.04f), bronze, glass, windows),
            Lantern(new(2.30f, 0f, -0.25f), bronze, glass, windows) };

        Transform fountain = Group("Fountain and ground clock", null, new(0.11f, 0f, -1.27f));
        Piece("Fountain base", PrimitiveType.Cylinder, fountain, new(0, 0.082f, 0), new(2.04f, 0.086f, 2.04f), trim);
        Piece("Basin stone", PrimitiveType.Cylinder, fountain, new(0, 0.205f, 0), new(1.82f, 0.17f, 1.82f), trim);
        Piece("Still water", PrimitiveType.Cylinder, fountain, new(0, 0.378f, 0), new(1.53f, 0.010f, 1.53f), water);
        Piece("Clock island", PrimitiveType.Cylinder, fountain, new(0, 0.414f, 0), new(1.11f, 0.044f, 1.11f), platform);
        for (int i = 0; i < 24; i++)
        {
            float angle = i * 15f;
            Quaternion rotation = Quaternion.Euler(0, angle, 0);
            var tick = Piece("Hour marker", PrimitiveType.Cube, fountain,
                Vector3.up * 0.467f + rotation * Vector3.forward * 0.46f,
                i % 6 == 0 ? new(0.034f, 0.012f, 0.128f) : new(0.020f, 0.012f, 0.067f), trim);
            tick.transform.localRotation = rotation;
        }
        Transform hand = Group("Clock hand", fountain, new(0, 0.486f, 0));
        Piece("Bronze hand", PrimitiveType.Cube, hand, new(0, 0, 0.18f), new(0.05f, 0.027f, 0.40f), bronze);
        Piece("Axle", PrimitiveType.Cylinder, fountain, new(0, 0.496f, 0), new(0.13f, 0.035f, 0.13f), bronze);

        Material sky = Sky("Square sky", "#536983", "#7F9EAE");
        Camera view = CameraAt(new(5.0f, 10.2f, -13.6f), new(0.39f, 0.91f, 0.02f), 4.86f);
        view.transform.position += view.transform.right * 0.43f - view.transform.up * 0.20f;
        var controller = new GameObject("Square time controls").AddComponent<SquareClock>();
        controller.sun = Directional("Sun", Color.white, 1.4f, new(53, 240, 0), true);
        controller.moon = Directional("Moon", Hex("#A3B9FF"), 0.40f, new(40, 143, 0));
        controller.lanterns = lanterns;
        controller.windows = windows.ToArray();
        controller.clockHand = hand;
        controller.sky = sky;
        controller.view = view;
        controller.SetHour(12);
        Capture(view, "square-noon");
        Vector3 noonSun = controller.sun.transform.forward;
        controller.SetHour(17);
        if (Vector3.Angle(noonSun, controller.sun.transform.forward) < 25f || controller.sun.intensity <= 0f)
            throw new InvalidOperationException("Square: the sun must move before sunset.");
        Capture(view, "square-evening");
        controller.SetHour(22);
        if (controller.sun.intensity > 0.01f || controller.lanterns[0].intensity < 1f)
            throw new InvalidOperationException("Square: midnight lighting did not turn on.");
        Capture(view, "square-night");
        controller.SetHour(11.5f);
        EditorSceneManager.SaveScene(scene, SquareScene);
        Debug.Log("Kinetka: square daylight, moving sun and illuminated night checked.");
    }

    private static void House(string name, Vector3 position, float width, float depth, float height,
        Material plaster, Material roof, Material shutters, Material trim, Material glass, List<Renderer> windows)
    {
        Transform house = Group(name, null, position);
        Piece("Plaster", PrimitiveType.Cube, house, new(0, height / 2f, 0), new(width, height, depth), plaster);
        Piece("Stone plinth", PrimitiveType.Cube, house, new(0, 0.09f, 0),
            new(width + 0.09f, 0.18f, depth + 0.09f), trim);
        const float rise = 0.59f;
        float roofHalf = depth / 2f + 0.11f;
        Mesh gable = SaveMesh(name + " gable", RoofPrism(width, depth, rise));
        var gableObject = Group("Gable", house, Vector3.up * (height - 0.02f)).gameObject;
        gableObject.AddComponent<MeshFilter>().sharedMesh = gable;
        gableObject.AddComponent<MeshRenderer>().sharedMaterial = plaster;
        foreach (int side in new[] { -1, 1 })
        {
            var slope = Piece("Roof slope", PrimitiveType.Cube, house,
                new(0, height + rise / 2f, side * roofHalf / 2f),
                new(width + 0.24f, 0.095f, Mathf.Sqrt(roofHalf * roofHalf + rise * rise) + 0.045f), roof);
            slope.transform.localRotation = Quaternion.Euler(side * Mathf.Atan2(rise, roofHalf) * Mathf.Rad2Deg, 0, 0);
        }
        Piece("Chimney", PrimitiveType.Cube, house, new(width * 0.3f, height + 0.48f, 0.21f),
            new(0.26f, 0.68f, 0.24f), plaster);
        Piece("Chimney cap", PrimitiveType.Cube, house, new(width * 0.3f, height + 0.83f, 0.21f),
            new(0.33f, 0.12f, 0.32f), roof);
        Piece("Wooden door", PrimitiveType.Cube, house, new(0, 0.37f, -depth / 2f - 0.032f),
            new(0.38f, 0.67f, 0.078f), shutters);
        Piece("Doorstep", PrimitiveType.Cube, house, new(0, 0.049f, -depth / 2f - 0.16f),
            new(0.57f, 0.093f, 0.37f), trim);
        foreach (int side in new[] { -1, 1 })
        {
            Window(house, new(side * width * 0.28f, 0.59f, -depth / 2f), 0,
                shutters, trim, glass, windows);
            Window(house, new(side * width * 0.29f, height - 0.52f, -depth / 2f), 0,
                shutters, trim, glass, windows);
        }
        Window(house, new(width / 2f, height - 0.57f, 0), -90, shutters, trim, glass, windows);
        Window(house, new(-width / 2f, height - 0.57f, 0), 90, shutters, trim, glass, windows);
    }

    private static void Window(Transform house, Vector3 position, float rotation,
        Material shutters, Material trim, Material glass, List<Renderer> windows)
    {
        Transform root = Group("Window", house, position);
        root.localRotation = Quaternion.Euler(0, rotation, 0);
        Piece("Window frame", PrimitiveType.Cube, root, new(0, 0, -0.038f), new(0.40f, 0.43f, 0.057f), trim);
        windows.Add(Piece("Glass pane", PrimitiveType.Cube, root, new(0, 0, -0.074f),
            new(0.317f, 0.355f, 0.035f), glass).GetComponent<Renderer>());
        Piece("Vertical muntin", PrimitiveType.Cube, root, new(0, 0, -0.097f), new(0.023f, 0.355f, 0.032f), trim);
        Piece("Horizontal muntin", PrimitiveType.Cube, root, new(0, 0, -0.097f), new(0.317f, 0.022f, 0.032f), trim);
        Piece("Left shutter", PrimitiveType.Cube, root, new(-0.255f, 0, -0.051f), new(0.12f, 0.39f, 0.059f), shutters);
        Piece("Right shutter", PrimitiveType.Cube, root, new(0.255f, 0, -0.051f), new(0.12f, 0.39f, 0.059f), shutters);
    }

    private static void Tree(Vector3 position, float height, string modelName, Material leaves, Material timber)
    {
        Transform root = Group("Tree", null, position);
        GameObject asset = AssetDatabase.LoadAssetAtPath<GameObject>($"Assets/Kenney Nature Kit/{modelName}.fbx");
        if (asset == null)
            throw new InvalidOperationException($"Missing Kenney model {modelName}.");
        var model = (GameObject)PrefabUtility.InstantiatePrefab(asset, root);
        model.transform.localPosition = Vector3.zero;
        Renderer[] renderers = model.GetComponentsInChildren<Renderer>();
        Bounds bounds = renderers[0].bounds;
        foreach (Renderer renderer in renderers)
        {
            bounds.Encapsulate(renderer.bounds);
            renderer.sharedMaterial = leaves;
        }
        float scale = height / bounds.size.y;
        model.transform.localScale *= scale;
        model.transform.localPosition = Vector3.up * (0.14f - (bounds.min.y - position.y) * scale);
        Piece("Tree trunk", PrimitiveType.Cylinder, root, new(0, 0.16f, 0), new(0.16f, 0.17f, 0.16f), timber);
    }

    private static void Bench(Vector3 position, float rotation, Material wood, Material metal)
    {
        Transform bench = Group("Bench", null, position);
        bench.localRotation = Quaternion.Euler(0, rotation, 0);
        Piece("Seat", PrimitiveType.Cube, bench, new(0, 0.25f, 0), new(0.91f, 0.07f, 0.31f), wood);
        Piece("Backrest", PrimitiveType.Cube, bench, new(0, 0.42f, 0.13f), new(0.91f, 0.11f, 0.05f), wood);
        foreach (int side in new[] { -1, 1 })
        {
            Piece("Foot", PrimitiveType.Cube, bench, new(side * 0.34f, 0.12f, 0), new(0.07f, 0.25f, 0.32f), metal);
            Piece("Back support", PrimitiveType.Cube, bench, new(side * 0.34f, 0.34f, 0.13f),
                new(0.059f, 0.24f, 0.055f), metal);
        }
    }

    private static Light Lantern(Vector3 position, Material bronze, Material glass, List<Renderer> windows)
    {
        Transform root = Group("Street lantern", null, position);
        Piece("Post", PrimitiveType.Cylinder, root, new(0, 0.63f, 0), new(0.075f, 0.62f, 0.075f), bronze);
        Piece("Foot", PrimitiveType.Cylinder, root, new(0, 0.055f, 0), new(0.21f, 0.06f, 0.21f), bronze);
        Piece("Lantern shade", PrimitiveType.Cylinder, root, new(0, 1.45f, 0), new(0.26f, 0.045f, 0.26f), bronze);
        windows.Add(Piece("Lantern glass", PrimitiveType.Cube, root, new(0, 1.31f, 0),
            new(0.16f, 0.20f, 0.16f), glass).GetComponent<Renderer>());
        var lamp = Group("Warm light", root, new(0, 1.3f, -0.16f)).gameObject.AddComponent<Light>();
        lamp.type = LightType.Point;
        lamp.color = Hex("#FFAF62");
        lamp.range = 4f;
        lamp.renderMode = LightRenderMode.ForcePixel;
        return lamp;
    }

    private static Mesh RoofPrism(float width, float depth, float rise)
    {
        float x = width / 2f;
        float z = depth / 2f;
        return FlatMesh(new Vector3[]
        {
            new(-x, 0, -z), new(-x, 0, z), new(-x, rise, 0),
            new(x, 0, -z), new(x, 0, z), new(x, rise, 0)
        }, new[] { 0, 1, 2, 3, 5, 4 });
    }

    private static void AssembleDomino()
    {
        Scene scene = NewScene();
        var body = Ink("Domino plum", "#543B64");
        var seam = Ink("Domino copper seam", "#CD856B", 0.40f, 0.14f);
        var surface = Ink("Domino lilac felt", "#956E90");
        var guide = Ink("Domino path", "#C997A4");
        var firstInk = Ink("Domino first coral", "#ED765A", 0.31f);
        var porcelain = Ink("Domino porcelain", "#FFDBBC", 0.36f);
        var rose = Ink("Domino mauve", "#CD8197", 0.36f);
        var pips = Ink("Domino inlay", "#593747");
        var brass = Ink("Domino brass", "#F1A853", 0.51f, 0.30f);
        var light = Ink("Domino bell signal", "#AA5F48");
        light.EnableKeyword("_EMISSION");
        light.SetColor("_EmissionColor", Color.black);
        var physics = AssetDatabase.LoadAssetAtPath<PhysicsMaterial>($"{Art}/Domino grip.physicMaterial");
        if (physics == null)
        {
            physics = new PhysicsMaterial("Domino grip");
            AssetDatabase.CreateAsset(physics, $"{Art}/Domino grip.physicMaterial");
        }
        physics.staticFriction = 0.70f;
        physics.dynamicFriction = 0.55f;
        physics.frictionCombine = PhysicsMaterialCombine.Maximum;
        EditorUtility.SetDirty(physics);

        Piece("Plum base", PrimitiveType.Cube, null, new(0, -0.42f, 0), new(12.58f, 0.56f, 5.52f), body);
        Piece("Copper seam", PrimitiveType.Cube, null, new(0, -0.157f, 0), new(12.60f, 0.043f, 5.54f), seam);
        var deck = Piece("Felt deck", PrimitiveType.Cube, null, new(0, -0.07f, 0), new(12.47f, 0.14f, 5.40f), surface, true);
        deck.GetComponent<Collider>().sharedMaterial = physics;
        foreach (int x in new[] { -1, 1 })
        foreach (int z in new[] { -1, 1 })
            Piece("Corner rivet", PrimitiveType.Cylinder, null, new(x * 5.99f, 0.014f, z * 2.48f),
                new(0.11f, 0.014f, 0.11f), seam);

        Vector3[] path = DominoPath();
        Transform tracks = Group("Painted path");
        Transform blocks = Group("Domino chain");
        var tiles = new Rigidbody[path.Length];
        for (int i = 0; i < path.Length; i++)
        {
            Vector3 tangent = i == path.Length - 1 ? path[i] - path[i - 1] : path[i + 1] - path[i];
            Quaternion facing = Quaternion.LookRotation(tangent.normalized);
            Piece("Guide inlay", PrimitiveType.Cylinder, tracks, path[i] + Vector3.up * 0.004f,
                new(0.73f, 0.004f, 0.73f), i == 0 ? firstInk : guide);
            tiles[i] = Domino(blocks, path[i], facing, i, i == 0 ? firstInk : i % 6 < 3 ? porcelain : rose, pips, physics);
        }

        Vector3 end = path[^1];
        Vector3 forward = (end - path[^2]).normalized;
        Vector3 bellPosition = end + forward * 0.87f;
        Transform stand = Group("Bell stand", null, bellPosition);
        stand.rotation = Quaternion.LookRotation(forward);
        Piece("Bell plinth", PrimitiveType.Cylinder, stand, new(0, 0.04f, 0.13f), new(1.21f, 0.041f, 1.21f), body);
        Renderer signal = Piece("Bell signal", PrimitiveType.Cylinder, stand, new(0, 0.087f, 0.13f),
            new(1.04f, 0.022f, 1.04f), light).GetComponent<Renderer>();
        Piece("Rear post", PrimitiveType.Cylinder, stand, new(0, 0.77f, 0.50f), new(0.116f, 0.72f, 0.116f), body);
        Piece("Bracket", PrimitiveType.Cube, stand, new(0, 1.38f, 0.22f), new(0.15f, 0.12f, 0.68f), seam);
        Transform bell = Group("Bell hinge", stand, new(0, 1.36f, 0));
        Rigidbody bellBody = bell.gameObject.AddComponent<Rigidbody>();
        bellBody.isKinematic = true;
        bellBody.collisionDetectionMode = CollisionDetectionMode.ContinuousSpeculative;
        var dome = Group("Brass bell", bell, new(0, -0.83f, 0)).gameObject;
        Mesh bellMesh = SaveMesh("Brass bell", BellMesh());
        dome.AddComponent<MeshFilter>().sharedMesh = bellMesh;
        dome.AddComponent<MeshRenderer>().sharedMaterial = brass;
        var shell = dome.AddComponent<MeshCollider>();
        shell.convex = true;
        shell.sharedMesh = bellMesh;
        Piece("Clapper", PrimitiveType.Sphere, bell, new(0, -0.90f, 0), new(0.17f, 0.19f, 0.17f), seam);

        Sky("Domino sky", "#43344F", "#241D34");
        RenderSettings.ambientMode = AmbientMode.Trilight;
        RenderSettings.ambientSkyColor = Hex("#D4AFCA") * 0.62f;
        RenderSettings.ambientEquatorColor = Hex("#B887A3") * 0.57f;
        RenderSettings.ambientGroundColor = Hex("#69566C") * 0.49f;
        Directional("Softbox", Hex("#FFE0C7"), 0.91f, new(46f, -28f, 0), true);
        Directional("Violet fill", Hex("#D7C2FF"), 0.23f, new(42f, 146f, 0));
        Camera view = CameraAt(new(1.66f, 9.7f, -12.6f), new(0.18f, 0.29f, 0), 4.62f);
        var controller = bell.gameObject.AddComponent<DominoRun>();
        controller.tiles = tiles;
        controller.bell = bellBody;
        controller.bellSignal = signal;
        controller.view = view;
        controller.sound = controller.gameObject.AddComponent<AudioSource>();
        controller.sound.playOnAwake = false;
        EditorSceneManager.SaveScene(scene, DominoScene);
        Capture(view, "domino-ready");
    }

    private static Vector3[] DominoPath()
    {
        const float gap = 0.425f;
        var points = new List<Vector3>();
        Vector3 last = new(-5.02f, 0, -0.39f);
        for (float x = -5.02f; x <= 4.78f; x += 0.006f)
        {
            float t = Mathf.InverseLerp(-5.02f, 4.78f, x);
            Vector3 point = new(x, 0, -0.39f + 1.02f * Mathf.Sin(t * Mathf.PI * 2f) + 0.64f * t);
            if (points.Count > 0 && Vector3.Distance(last, point) < gap)
                continue;
            points.Add(point);
            last = point;
        }
        return points.ToArray();
    }

    private static Rigidbody Domino(Transform parent, Vector3 position, Quaternion facing, int index,
        Material enamel, Material ink, PhysicsMaterial physics)
    {
        const float height = 0.96f;
        Transform block = Group($"Domino {index + 1:00}", parent, position + Vector3.up * (height / 2f + 0.003f));
        block.rotation = facing;
        var box = block.gameObject.AddComponent<BoxCollider>();
        box.size = new Vector3(0.39f, height, 0.165f);
        box.sharedMaterial = physics;
        var rb = block.gameObject.AddComponent<Rigidbody>();
        rb.mass = 0.30f;
        rb.collisionDetectionMode = CollisionDetectionMode.ContinuousDynamic;
        rb.solverIterations = 12;
        rb.solverVelocityIterations = 5;

        Piece("Enamel", PrimitiveType.Cube, block, Vector3.zero, box.size, enamel);
        Piece("Top ridge", PrimitiveType.Cube, block, new(0, 0.461f, 0), new(0.322f, 0.039f, 0.178f), enamel);
        Piece("Face divider", PrimitiveType.Cube, block, new(0, 0, -0.085f),
            new(0.285f, 0.015f, 0.012f), ink);
        for (int row = -1; row <= 1; row += 2)
        for (int side = -1; side <= 1; side += 2)
        {
            var pip = Piece("Inlaid dot", PrimitiveType.Cylinder, block,
                new(side * 0.082f, row * 0.229f + side * row * 0.081f, -0.086f),
                new(0.068f, 0.005f, 0.068f), ink);
            pip.transform.localRotation = Quaternion.Euler(90, 0, 0);
        }
        if (index % 2 == 0)
        {
            var pip = Piece("Fifth dot", PrimitiveType.Cylinder, block, new(0, 0.229f, -0.086f),
                new(0.068f, 0.005f, 0.068f), ink);
            pip.transform.localRotation = Quaternion.Euler(90, 0, 0);
        }
        return rb;
    }

    private static Mesh BellMesh()
    {
        Vector2[] profile = { new(0.48f, 0.02f), new(0.51f, 0.055f), new(0.47f, 0.13f), new(0.36f, 0.20f),
            new(0.31f, 0.40f), new(0.26f, 0.50f), new(0.19f, 0.57f), new(0.08f, 0.59f), new(0.02f, 0.59f) };
        const int segments = 36;
        var vertices = new Vector3[profile.Length * segments];
        var triangles = new List<int>();
        for (int band = 0; band < profile.Length; band++)
        for (int i = 0; i < segments; i++)
        {
            float angle = i * (Mathf.PI * 2f / segments);
            Vector2 rim = profile[band];
            int index = band * segments + i;
            vertices[index] = new Vector3(Mathf.Cos(angle) * rim.x, rim.y, Mathf.Sin(angle) * rim.x);
            if (band == profile.Length - 1)
                continue;
            int next = band * segments + (i + 1) % segments;
            triangles.AddRange(new[] { index, index + segments, next, next, index + segments, next + segments });
        }
        var mesh = new Mesh { vertices = vertices, triangles = triangles.ToArray() };
        mesh.RecalculateNormals();
        return mesh;
    }

    private static Scene NewScene()
    {
        QualitySettings.shadows = ShadowQuality.All;
        QualitySettings.shadowDistance = 35f;
        QualitySettings.pixelLightCount = 6;
        return EditorSceneManager.NewScene(NewSceneSetup.EmptyScene, NewSceneMode.Single);
    }

    private static Transform Group(string name, Transform parent = null, Vector3 position = default)
    {
        Transform transform = new GameObject(name).transform;
        transform.SetParent(parent, false);
        transform.localPosition = position;
        return transform;
    }

    private static GameObject Piece(string name, PrimitiveType shape, Transform parent,
        Vector3 position, Vector3 size, Material material, bool solid = false)
    {
        GameObject part = GameObject.CreatePrimitive(shape);
        part.name = name;
        part.transform.SetParent(parent, false);
        part.transform.localPosition = position;
        part.transform.localScale = size;
        part.GetComponent<Renderer>().sharedMaterial = material;
        if (!solid)
            UnityEngine.Object.DestroyImmediate(part.GetComponent<Collider>());
        return part;
    }

    private static Mesh FlatMesh(Vector3[] vertices, int[] faces)
    {
        var separated = new Vector3[faces.Length];
        for (int i = 0; i < separated.Length; i++)
        {
            separated[i] = vertices[faces[i]];
            faces[i] = i;
        }
        var mesh = new Mesh { vertices = separated, triangles = faces };
        mesh.RecalculateNormals();
        return mesh;
    }

    private static Mesh SaveMesh(string name, Mesh mesh)
    {
        string path = $"{Art}/{name}.asset";
        mesh.name = name;
        Mesh existing = AssetDatabase.LoadAssetAtPath<Mesh>(path);
        if (existing == null)
        {
            AssetDatabase.CreateAsset(mesh, path);
            return mesh;
        }
        EditorUtility.CopySerialized(mesh, existing);
        UnityEngine.Object.DestroyImmediate(mesh);
        return existing;
    }

    private static Color Hex(string hex) =>
        ColorUtility.TryParseHtmlString(hex, out Color color) ? color : throw new ArgumentException(hex);

    private static Material Ink(string name, string hex, float smoothness = 0.24f, float metal = 0f)
    {
        Material material = MaterialAsset(name, "Standard");
        material.color = Hex(hex);
        material.SetFloat("_Glossiness", smoothness);
        material.SetFloat("_Metallic", metal);
        EditorUtility.SetDirty(material);
        return material;
    }

    private static Material Sky(string name, string topColor, string bottomColor)
    {
        Material sky = MaterialAsset(name, "Mira/Garden Sky");
        sky.SetColor("_Top", Hex(topColor));
        sky.SetColor("_Bottom", Hex(bottomColor));
        EditorUtility.SetDirty(sky);
        RenderSettings.skybox = sky;
        return sky;
    }

    private static Material MaterialAsset(string name, string shader)
    {
        string path = $"{Art}/{name}.mat";
        Material material = AssetDatabase.LoadAssetAtPath<Material>(path);
        if (material != null)
            return material;
        material = new Material(Shader.Find(shader));
        AssetDatabase.CreateAsset(material, path);
        return material;
    }

    private static Camera CameraAt(Vector3 position, Vector3 center, float size)
    {
        Camera camera = Group("Camera", null, position).gameObject.AddComponent<Camera>();
        camera.tag = "MainCamera";
        camera.gameObject.AddComponent<AudioListener>();
        camera.transform.LookAt(center);
        camera.orthographic = true;
        camera.orthographicSize = size;
        camera.clearFlags = CameraClearFlags.Skybox;
        camera.nearClipPlane = 0.1f;
        camera.farClipPlane = 50f;
        return camera;
    }

    private static Light Directional(string name, Color color, float power, Vector3 angles, bool shadows = false)
    {
        var light = new GameObject(name).AddComponent<Light>();
        light.type = LightType.Directional;
        light.color = color;
        light.intensity = power;
        light.shadows = shadows ? LightShadows.Soft : LightShadows.None;
        light.shadowStrength = 0.78f;
        light.transform.rotation = Quaternion.Euler(angles);
        return light;
    }

    internal static void Capture(Camera view, string shot)
    {
        var target = new RenderTexture(1280, 820, 24) { antiAliasing = 4 };
        var image = new Texture2D(target.width, target.height, TextureFormat.RGB24, false);
        RenderTexture previous = RenderTexture.active;
        try
        {
            view.targetTexture = target;
            view.Render();
            RenderTexture.active = target;
            image.ReadPixels(new Rect(0, 0, target.width, target.height), 0, 0);
            File.WriteAllBytes(Path.Combine(OutputFolder, shot + ".png"), image.EncodeToPNG());
        }
        finally
        {
            RenderTexture.active = previous;
            view.targetTexture = null;
            UnityEngine.Object.DestroyImmediate(image);
            UnityEngine.Object.DestroyImmediate(target);
        }
    }

    private static void Build(string scene, string fileName, string title)
    {
        string directory = Path.Combine(OutputFolder, fileName);
        Directory.CreateDirectory(directory);
        string formerName = PlayerSettings.productName;
        try
        {
            PlayerSettings.productName = title;
            BuildReport report = BuildPipeline.BuildPlayer(new BuildPlayerOptions
            {
                scenes = new[] { scene },
                locationPathName = Path.Combine(directory, fileName + ".exe"),
                target = BuildTarget.StandaloneWindows64
            });
            if (report.summary.result != BuildResult.Succeeded)
                throw new InvalidOperationException($"{title}: {report.summary.result}");
            Debug.Log($"Kinetka: built {report.summary.outputPath}");
        }
        finally
        {
            PlayerSettings.productName = formerName;
        }
    }
}
