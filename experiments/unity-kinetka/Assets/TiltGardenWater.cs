using UnityEngine;
using UnityEngine.Rendering;

public sealed class TiltGardenWater : MonoBehaviour
{
    private const int GridSize = 41;
    private const int RingSegments = 64;
    private const float SplashDuration = 1.35f;
    private const float DropletDuration = 0.68f;
    private static readonly Color WaterColor = new(0.07f, 0.31f, 0.34f);
    private static readonly Color FoamColor = new(0.39f, 0.73f, 0.69f);
    private static readonly int ColorProperty = Shader.PropertyToID("_Color");
    private static readonly int EmissionProperty = Shader.PropertyToID("_EmissionColor");

    private readonly Vector3[] vertices = new Vector3[GridSize * GridSize];
    private readonly MeshRenderer[] rings = new MeshRenderer[3];
    private readonly Transform[] droplets = new Transform[11];
    private MaterialPropertyBlock foam;
    private Mesh surfaceMesh;
    private Mesh ringMesh;
    private Mesh dropletMesh;
    private Transform surface;
    private Material surfaceMaterial;
    private Material foamMaterial;
    private float waterY;
    private float extent;
    private float splashTime;
    private float splashStrength;
    private float entryTime;
    private Vector3 splashCenter;
    private bool entered;

    public void Initialize(float surfaceY, float halfExtent)
    {
        waterY = surfaceY;
        extent = Mathf.Max(1f, halfExtent);
        if (surfaceMesh == null)
            CreateVisuals();

        surface.position = new Vector3(0f, waterY, 0f);
        UpdateSurface(0f, 0f);
        surfaceMesh.bounds = new Bounds(Vector3.zero, new Vector3(extent * 2f, 0.5f, extent * 2f));
        ResetWater();
    }

    public void ResetWater()
    {
        entered = false;
        entryTime = 0f;
        splashTime = -SplashDuration;
        foreach (MeshRenderer ring in rings)
        {
            if (ring != null)
                ring.enabled = false;
        }
        foreach (Transform droplet in droplets)
        {
            if (droplet != null)
                droplet.gameObject.SetActive(false);
        }
    }

    public bool Step(Rigidbody marble, float radius)
    {
        Vector3 position = marble.position;
        bool overWater = Mathf.Abs(position.x) <= extent + radius && Mathf.Abs(position.z) <= extent + radius;
        float depth = overWater ? waterY + radius - position.y : 0f;
        if (!entered && depth > 0f)
        {
            entered = true;
            splashTime = Time.time;
            splashCenter = new Vector3(
                Mathf.Clamp(position.x, -extent, extent),
                waterY,
                Mathf.Clamp(position.z, -extent, extent));
            splashStrength = Mathf.Clamp(-marble.linearVelocity.y / 7f, 0.35f, 1.2f);
        }

        if (!entered)
            return false;

        entryTime += Time.fixedDeltaTime;
        if (depth > 0f)
        {
            float fraction = Mathf.Clamp01(depth / (2f * radius));
            float volume = fraction * fraction * (3f - 2f * fraction);
            // A dense marble still sinks once fully submerged.
            Vector3 buoyancy = -Physics.gravity * (0.70f * volume);
            Vector3 drag = -marble.linearVelocity * (5.9f * volume);
            marble.AddForce(buoyancy + drag, ForceMode.Acceleration);
            marble.AddTorque(-marble.angularVelocity * (3.3f * volume), ForceMode.Acceleration);
        }

        return entryTime >= SplashDuration;
    }

    private void LateUpdate()
    {
        if (surfaceMesh == null)
            return;

        float age = Time.time - splashTime;
        float impact = entered ? splashStrength * Mathf.Clamp01(1f - age / SplashDuration) : 0f;
        UpdateSurface(Time.time, impact);
        UpdateRipples(age, impact);
        UpdateDroplets(entered ? age : DropletDuration);
    }

    private void CreateVisuals()
    {
        foam = new MaterialPropertyBlock();
        surfaceMaterial = MakeMaterial("Garden water", WaterColor, WaterColor * 0.16f, 0.79f);
        foamMaterial = MakeMaterial("Garden whitewater", FoamColor, FoamColor * 0.37f, 0.50f);
        surfaceMesh = new Mesh { name = "Garden water grid" };
        surfaceMesh.MarkDynamic();
        surfaceMesh.vertices = vertices;
        surfaceMesh.triangles = GridTriangles();
        surface = CreateVisual("Water surface", surfaceMesh, surfaceMaterial).transform;
        surface.GetComponent<MeshRenderer>().receiveShadows = true;

        ringMesh = MakeRingMesh();
        for (int index = 0; index < rings.Length; index++)
            rings[index] = CreateVisual("Splash ripple", ringMesh, foamMaterial);

        dropletMesh = MakeDropletMesh();
        for (int index = 0; index < droplets.Length; index++)
            droplets[index] = CreateVisual("Splash droplet", dropletMesh, foamMaterial).transform;
    }

    private MeshRenderer CreateVisual(string objectName, Mesh mesh, Material material)
    {
        var visual = new GameObject(objectName);
        visual.transform.SetParent(transform, false);
        visual.AddComponent<MeshFilter>().sharedMesh = mesh;
        MeshRenderer renderer = visual.AddComponent<MeshRenderer>();
        renderer.sharedMaterial = material;
        renderer.shadowCastingMode = ShadowCastingMode.Off;
        renderer.receiveShadows = false;
        return renderer;
    }

    private static Material MakeMaterial(string materialName, Color color, Color emission, float smoothness)
    {
        var material = new Material(Shader.Find("Standard")) { name = materialName, color = color };
        material.SetFloat("_Glossiness", smoothness);
        material.SetColor(EmissionProperty, emission);
        material.EnableKeyword("_EMISSION");
        return material;
    }

    private void UpdateSurface(float time, float impact)
    {
        float stride = 2f * extent / (GridSize - 1);
        for (int row = 0; row < GridSize; row++)
        {
            float z = row * stride - extent;
            for (int column = 0; column < GridSize; column++)
            {
                float x = column * stride - extent;
                float y = 0.025f * Mathf.Sin(2.3f * x + 1.5f * z + time * 1.3f)
                    + 0.017f * Mathf.Sin(1.7f * x - 3.1f * z + time * 1.8f);
                if (impact > 0f)
                {
                    float distance = Vector2.Distance(new Vector2(x, z), new Vector2(splashCenter.x, splashCenter.z));
                    float front = distance - (time - splashTime) * 2.3f;
                    if (Mathf.Abs(front) < 1.15f)
                        y += 0.14f * impact * (1f - Mathf.Abs(front) / 1.15f) * Mathf.Cos(front * 7f);
                }
                vertices[row * GridSize + column] = new Vector3(x, y, z);
            }
        }
        surfaceMesh.vertices = vertices;
        surfaceMesh.RecalculateNormals();
    }

    private void UpdateRipples(float age, float impact)
    {
        float reach = extent - Mathf.Max(Mathf.Abs(splashCenter.x), Mathf.Abs(splashCenter.z));
        for (int index = 0; index < rings.Length; index++)
        {
            MeshRenderer ring = rings[index];
            float elapsed = age - index * 0.145f;
            float radius = 0.15f + elapsed * 2.1f;
            ring.enabled = impact > 0f && elapsed >= 0f && radius < reach;
            if (!ring.enabled)
                continue;

            ring.transform.position = splashCenter + Vector3.up * 0.065f;
            ring.transform.localScale = new Vector3(radius, 1f, radius);
            float visible = Mathf.Clamp01(impact * (1f - index * 0.19f));
            Color tone = Color.Lerp(WaterColor, FoamColor, visible);
            foam.SetColor(ColorProperty, tone);
            foam.SetColor(EmissionProperty, tone * (0.16f + visible * 0.21f));
            ring.SetPropertyBlock(foam);
        }
    }

    private void UpdateDroplets(float age)
    {
        for (int index = 0; index < droplets.Length; index++)
        {
            Transform droplet = droplets[index];
            float angle = index * 2.4f;
            float launch = (2.2f + 0.28f * (index % 5)) * Mathf.Sqrt(splashStrength);
            float lift = age * launch - 4.9f * age * age;
            bool active = age < DropletDuration && lift > 0f;
            droplet.gameObject.SetActive(active);
            if (!active)
                continue;

            float reach = (0.42f + (index % 4) * 0.18f) * age * splashStrength;
            droplet.position = splashCenter + new Vector3(Mathf.Cos(angle) * reach, lift, Mathf.Sin(angle) * reach);
            droplet.localScale = Vector3.one * (0.030f + 0.019f * (index % 3));
        }
    }

    private static int[] GridTriangles()
    {
        int[] triangles = new int[(GridSize - 1) * (GridSize - 1) * 6];
        int index = 0;
        for (int row = 0; row < GridSize - 1; row++)
        {
            for (int column = 0; column < GridSize - 1; column++)
            {
                int a = row * GridSize + column;
                int b = a + GridSize;
                triangles[index++] = a;
                triangles[index++] = b;
                triangles[index++] = a + 1;
                triangles[index++] = a + 1;
                triangles[index++] = b;
                triangles[index++] = b + 1;
            }
        }
        return triangles;
    }

    private static Mesh MakeRingMesh()
    {
        Vector3[] points = new Vector3[RingSegments * 2];
        int[] triangles = new int[RingSegments * 6];
        for (int index = 0; index < RingSegments; index++)
        {
            float angle = index * (2f * Mathf.PI / RingSegments);
            Vector3 direction = new(Mathf.Cos(angle), 0f, Mathf.Sin(angle));
            int current = index * 2;
            int next = (current + 2) % points.Length;
            points[current] = direction * 0.989f;
            points[current + 1] = direction;
            int triangle = index * 6;
            triangles[triangle] = current;
            triangles[triangle + 1] = next;
            triangles[triangle + 2] = current + 1;
            triangles[triangle + 3] = current + 1;
            triangles[triangle + 4] = next;
            triangles[triangle + 5] = next + 1;
        }
        var mesh = new Mesh { name = "Garden ripple", vertices = points, triangles = triangles };
        mesh.RecalculateNormals();
        return mesh;
    }

    private static Mesh MakeDropletMesh()
    {
        var mesh = new Mesh
        {
            name = "Garden droplet",
            vertices = new[] { Vector3.up, Vector3.down, Vector3.right, Vector3.forward, Vector3.left, Vector3.back },
            triangles = new[] { 0, 3, 2, 0, 4, 3, 0, 5, 4, 0, 2, 5, 1, 2, 3, 1, 3, 4, 1, 4, 5, 1, 5, 2 }
        };
        mesh.RecalculateNormals();
        return mesh;
    }

    private void OnDestroy()
    {
        Destroy(surfaceMesh);
        Destroy(ringMesh);
        Destroy(dropletMesh);
        Destroy(surfaceMaterial);
        Destroy(foamMaterial);
    }
}
