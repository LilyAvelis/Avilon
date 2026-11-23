using System;
using UnityEditor;
using UnityEngine;

public static class TiltGardenSecondLevel
{
    private const string MaterialPath = "Assets/Garden Materials";
    private const string NaturePath = "Assets/Kenney Nature Kit";

    public static (Rigidbody tray, Transform start, Transform goal, Renderer goalRing, Renderer deck) Create()
    {
        var tray = new GameObject("Switchback garden tray").AddComponent<Rigidbody>();
        tray.isKinematic = true;
        tray.interpolation = RigidbodyInterpolation.Interpolate;
        tray.collisionDetectionMode = CollisionDetectionMode.ContinuousSpeculative;
        Transform root = tray.transform;

        Material enamel = Asset<Material>($"{MaterialPath}/Dark green enamel.mat");
        Material sand = Asset<Material>($"{MaterialPath}/Sand.mat");
        Material cedar = Asset<Material>($"{MaterialPath}/Cedar rim.mat");
        Material trim = Asset<Material>($"{MaterialPath}/Cedar trim.mat");
        Material brass = Asset<Material>($"{MaterialPath}/Brass detail.mat");
        Material moss = Asset<Material>($"{MaterialPath}/Dark moss.mat");
        Material socket = Asset<Material>($"{MaterialPath}/Socket.mat");

        Box(root, "Square plinth", new Vector3(0f, -0.35f, 0f),
            new Vector3(12.2f, 0.44f, 12.2f), enamel, false);
        Renderer deck = Box(root, "Square sand surface", new Vector3(0f, -0.12f, 0f),
            new Vector3(12f, 0.24f, 12f), sand).GetComponent<Renderer>();

        Wall(root, "North rail", 0f, 5.89f, 12f, 0.22f, cedar, trim);
        Wall(root, "South rail", 0f, -5.89f, 12f, 0.22f, cedar, trim);
        Wall(root, "West lower rail", -5.89f, -3.85f, 0.22f, 4.08f, cedar, trim);
        Wall(root, "West upper rail", -5.89f, 3.17f, 0.22f, 5.44f, cedar, trim);
        Wall(root, "East lower rail", 5.89f, -2.69f, 0.22f, 6.40f, cedar, trim);
        Wall(root, "East upper rail", 5.89f, 4.41f, 0.22f, 2.96f, cedar, trim);
        Box(root, "West spillway sill", new Vector3(-5.91f, 0.012f, -0.69f),
            new Vector3(0.16f, 0.018f, 2.24f), brass, false);
        Box(root, "East spillway sill", new Vector3(5.91f, 0.012f, 1.72f),
            new Vector3(0.16f, 0.018f, 2.40f), brass, false);

        Wall(root, "First divider", -2.86f, -1.97f, 0.30f, 7.84f, cedar, trim);
        Wall(root, "Second divider", 0f, 1.97f, 0.30f, 7.84f, cedar, trim);
        Wall(root, "Third divider", 2.86f, -1.78f, 0.30f, 8.22f, cedar, trim);

        Rock(root, "West lane boulder", -4.89f, 1.35f, 0.57f, 20f, moss);
        Rock(root, "Inner lane boulder", -0.92f, 0.64f, 0.61f, -24f, moss);
        Rock(root, "Middle lane boulder", 1.03f, -0.64f, 0.53f, 40f, moss);
        Rock(root, "East lane boulder", 4.96f, 2.51f, 0.44f, -31f, moss);

        Nature(root, "tree_pineDefaultA", new Vector3(-4.89f, 0.14f, 1.35f), 1.08f, 20f);
        Nature(root, "tree_pineRoundC", new Vector3(1.03f, 0.14f, -0.64f), 0.91f, -26f);
        Nature(root, "plant_bushSmall", new Vector3(-3.25f, 0.025f, -4.96f), 0.32f, -12f);
        Nature(root, "flower_yellowA", new Vector3(-1.52f, 0.025f, 4.92f), 0.27f, 35f);
        Nature(root, "flower_yellowA", new Vector3(1.29f, 0.025f, -4.92f), 0.27f, 90f);
        Nature(root, "grass_large", new Vector3(3.84f, 0.025f, 4.96f), 0.40f, -15f);

        Transform start = Marker(root, "Start", new Vector3(-4.69f, 0.266f, -4.54f));
        Transform goal = Marker(root, "Goal", new Vector3(4.70f, 0.276f, 4.12f));
        Disc(root, "Launch mark", start.localPosition, 0.88f, trim);
        Disc(root, "Catching bowl", goal.localPosition, 1.38f, socket);

        Transform light = Marker(root, "Goal light", new Vector3(4.70f, 0.082f, 4.12f));
        light.gameObject.AddComponent<MeshFilter>().sharedMesh = Asset<Mesh>($"{MaterialPath}/Goal Ring.asset");
        var goalRing = light.gameObject.AddComponent<MeshRenderer>();
        goalRing.sharedMaterial = Asset<Material>($"{MaterialPath}/Light ring.mat");
        return (tray, start, goal, goalRing, deck);
    }

    private static T Asset<T>(string path) where T : UnityEngine.Object
    {
        T asset = AssetDatabase.LoadAssetAtPath<T>(path);
        return asset != null ? asset : throw new InvalidOperationException($"Missing garden asset: {path}");
    }

    private static Transform Marker(Transform root, string name, Vector3 position)
    {
        var marker = new GameObject(name).transform;
        marker.SetParent(root, false);
        marker.localPosition = position;
        return marker;
    }

    private static GameObject Box(Transform root, string name, Vector3 position, Vector3 size,
        Material material, bool collision = true)
    {
        GameObject box = GameObject.CreatePrimitive(PrimitiveType.Cube);
        box.name = name;
        box.transform.SetParent(root, false);
        box.transform.localPosition = position;
        box.transform.localScale = size;
        box.GetComponent<Renderer>().sharedMaterial = material;
        if (!collision)
            UnityEngine.Object.DestroyImmediate(box.GetComponent<Collider>());
        return box;
    }

    private static void Wall(Transform root, string name, float x, float z, float width, float length,
        Material cedar, Material trim)
    {
        Box(root, name, new Vector3(x, 0.23f, z), new Vector3(width, 0.46f, length), cedar);
        Box(root, $"{name} cap", new Vector3(x, 0.47f, z),
            new Vector3(width + 0.025f, 0.035f, length), trim, false);
    }

    private static void Disc(Transform root, string name, Vector3 position, float diameter, Material material)
    {
        GameObject disc = GameObject.CreatePrimitive(PrimitiveType.Cylinder);
        disc.name = name;
        disc.transform.SetParent(root, false);
        disc.transform.localPosition = new Vector3(position.x, 0.011f, position.z);
        disc.transform.localScale = new Vector3(diameter, 0.012f, diameter);
        disc.GetComponent<Renderer>().sharedMaterial = material;
        UnityEngine.Object.DestroyImmediate(disc.GetComponent<Collider>());
    }

    private static void Rock(Transform root, string name, float x, float z, float radius, float yaw, Material moss)
    {
        Transform obstacle = Marker(root, name, new Vector3(x, 0f, z));
        Disc(obstacle, "Moss island", Vector3.zero, radius * 2.35f, moss);
        Nature(obstacle, "rock_largeB", Vector3.up * 0.02f, radius, yaw);
        CapsuleCollider barrier = obstacle.gameObject.AddComponent<CapsuleCollider>();
        barrier.center = Vector3.up * (radius * 0.60f);
        barrier.radius = radius * 0.77f;
        barrier.height = radius * 1.60f;
    }

    private static void Nature(Transform root, string name, Vector3 position, float height, float yaw)
    {
        Transform anchor = Marker(root, name, position);
        GameObject prefab = Asset<GameObject>($"{NaturePath}/{name}.fbx");
        var model = (GameObject)PrefabUtility.InstantiatePrefab(prefab, anchor);
        model.transform.localPosition = Vector3.zero;
        model.transform.localRotation = Quaternion.Euler(0f, yaw, 0f);

        Renderer[] renderers = model.GetComponentsInChildren<Renderer>();
        Bounds bounds = renderers[0].bounds;
        foreach (Renderer renderer in renderers)
            bounds.Encapsulate(renderer.bounds);
        float scale = height / Mathf.Max(0.001f, bounds.size.y);
        Vector3 center = anchor.InverseTransformPoint(bounds.center);
        model.transform.localScale *= scale;
        model.transform.localPosition = new Vector3(-center.x, bounds.extents.y - center.y, -center.z) * scale;
        foreach (Collider collider in model.GetComponentsInChildren<Collider>())
            UnityEngine.Object.DestroyImmediate(collider);
    }
}
