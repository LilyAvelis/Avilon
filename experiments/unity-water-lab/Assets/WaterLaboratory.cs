using System.Collections.Generic;
using UnityEngine;

public sealed partial class WaterLaboratory : MonoBehaviour
{
    public const float InteractionRadius = 118f;
    private const int RenderResolution = 513;
    private static readonly int[] WaveQualitySizes = { 1024, 1536, 2048, 3072, 4096 };
    private const float DefaultRadius = 0.32f;
    private const float MinRadius = 0.16f;
    private const float MaxRadius = 0.70f;
    private const float MaxDepth = -2.1f;

    public Camera view;
    public Rigidbody sphere;
    public Material waterMaterial;

    private sealed class Liquid
    {
        public readonly string Name;
        public readonly Color Tint;
        public readonly float Speed, Damping, Drag, Buoyancy, Extinction;

        public Liquid(string name, Color tint, float speed, float damping, float drag, float buoyancy, float extinction)
        {
            Name = name;
            Tint = tint;
            Speed = speed;
            Damping = damping;
            Drag = drag;
            Buoyancy = buoyancy;
            Extinction = extinction;
        }
    }

    private static readonly Liquid[] Liquids =
    {
        new("Water", new Color32(26, 116, 138, 255), 4.1f, 0.65f, 3.2f, 1.02f, 0.34f),
        new("Oil", new Color32(166, 131, 54, 255), 3.2f, 1.65f, 6.1f, 0.84f, 0.64f),
        new("Syrup", new Color32(133, 56, 31, 255), 2.4f, 4.2f, 11.5f, 1.18f, 1.04f)
    };
    private const int NoiseMode = 3;
#if MIRA_WATER_HDRP
    private const int HdrpMode = 4;
    private MeshRenderer laboratoryWaterRenderer;
#endif
    private static readonly int NoiseAmount = Shader.PropertyToID("_NoiseAmount");
    private static readonly int NoiseVariant = Shader.PropertyToID("_NoiseMode");
    private static readonly int WaterReach = Shader.PropertyToID("_WaterReach");
    private WaterLabGpuRain waveField;
    private Vector4 waveGrid;
    private int requestedWaveQuality = 2;
    private int noiseVariant = 1;
    private float noiseAmount = 0.65f;
    private Mesh mesh;
    private Material material;
    private int mode = 2;
    private int liquidIndex;
    private Liquid Current => Liquids[liquidIndex];
    private float radius = DefaultRadius;
    private bool floating;
    private bool falling;
    private bool touched;
    private float releaseTime;
    private float yaw = -23f;
    private float pitch = 19f;
    private float distance = 14.2f;

    private void Start()
    {
        Application.targetFrameRate = 90;
        view.depthTextureMode |= DepthTextureMode.Depth;
        material = new Material(waterMaterial);
        material.SetVector(WaterReach, new Vector4(0f, InteractionRadius, 0f, 0f));
        SetWaveResolution(WaveQualitySizes[requestedWaveQuality]);
        InitializeWeather();
        InitializeSeabed();
        InitializeLighting();
        BuildSurface();
#if MIRA_WATER_HDRP
        InitializeHdrp();
#endif
        ApplyMaterial();
        ConfigureBall(radius, floating);
        ResetLab();
        PositionCamera();
    }

    private void Update()
    {
        int targetResolution = WaveQualitySizes[requestedWaveQuality];
        if (targetResolution != waveField.GridResolution && !Input.GetMouseButton(0))
        {
            try
            {
                SetWaveResolution(targetResolution);
            }
            catch (System.Exception error)
            {
                requestedWaveQuality = System.Array.IndexOf(WaveQualitySizes, waveField.GridResolution);
                Debug.LogWarning($"Water quality stayed at {waveField.GridResolution}: {error.Message}");
            }
        }

        if (Input.GetKeyDown(KeyCode.Alpha1)) SelectMode(0);
        if (Input.GetKeyDown(KeyCode.Alpha2)) SelectMode(1);
        if (Input.GetKeyDown(KeyCode.Alpha3)) SelectMode(2);
        if (Input.GetKeyDown(KeyCode.Alpha4)) SelectMode(NoiseMode);
#if MIRA_WATER_HDRP
        if (Input.GetKeyDown(KeyCode.Alpha5)) SelectMode(HdrpMode);
        UpdateRenderingBackend();
#endif
        if (Input.GetKeyDown(KeyCode.Space)) Drop();
        if (Input.GetKeyDown(KeyCode.R)) ResetLab();

        if (!poincareOverview && Input.GetMouseButton(1))
        {
            yaw = Mathf.Repeat(yaw + Input.GetAxisRaw("Mouse X") * 4f, 360f);
            pitch = Mathf.Clamp(pitch - Input.GetAxisRaw("Mouse Y") * 3f, 5.5f, 66f);
        }
        if (!poincareOverview && !PointerOnPanel())
            distance = Mathf.Clamp(distance - Input.mouseScrollDelta.y * 1.1f, 5.2f, 27f);
        PositionCamera();
        UpdateWeather();
        UpdateSeabed();

        if (Input.GetMouseButtonDown(0) && !PointerOnPanel())
        {
            Ray ray = view.ScreenPointToRay(Input.mousePosition);
            var plane = new Plane(Vector3.up, Vector3.zero);
            if (plane.Raycast(ray, out float length))
            {
                Vector3 point = ray.GetPoint(length);
                Vector2 xz = new(point.x, point.z);
                if (xz.sqrMagnitude < InteractionRadius * InteractionRadius && (poincareOverview || length < InteractionRadius))
                    Strike(xz, 1.15f);
            }
        }
    }

    private void FixedUpdate()
    {
        StepRain(Time.fixedTimeAsDouble);
        waveField.Step(Time.fixedDeltaTime, Current.Speed, Current.Damping);
        ApplyWaveTexture();
        if (!falling)
            return;

        Vector3 position = sphere.position;
        if (HideSphereInFog(position))
            return;
        Vector2 xz = new(position.x, position.z);
        float waterHeight = waveField.SampleHeight(xz);
#if MIRA_WATER_HDRP
        if (hdrpActive)
            waterHeight = SampleHdrpHeight(position, waterHeight);
#endif
        float submerged = WaterLabBallPhysics.SubmergedFraction(position.y, waterHeight, radius);
        if (submerged > 0f)
        {
            if (!touched)
            {
                touched = true;
                float strength = WaterLabBallPhysics.ImpactStrength(-sphere.linearVelocity.y, radius);
                float width = 0.55f * radius / DefaultRadius;
                StrikeWave(xz, strength, Mathf.Max(0.28f, width));
            }

            float densityRatio = EffectiveDensityRatio();
            float drag = floating ? Mathf.Max(6f, Current.Drag) : Current.Drag;
            sphere.AddForce(WaterLabBallPhysics.Acceleration(
                sphere.linearVelocity - SurfaceCurrent(), Physics.gravity, submerged, densityRatio, drag), ForceMode.Acceleration);
        }
        bool reachedBottom = floating
            ? position.y < -3.5f
            : position.y < MaxDepth + radius || Time.time - releaseTime > 8f;
        if (reachedBottom)
            ParkSphere();
    }

    private Vector4 ActiveWaveGrid => waveField.IsActive ? waveGrid : Vector4.zero;

    private void SetWaveResolution(int resolution)
    {
        WaterLabGpuRain previous = waveField;
        var replacement = new WaterLabGpuRain(InteractionRadius, resolution);
        try
        {
            if (previous != null && previous.IsActive)
                replacement.RestoreFrom(previous);
        }
        catch
        {
            replacement.Dispose();
            throw;
        }

        waveField = replacement;
        float texel = 1f / resolution;
        float inverseSpacing = (resolution - 1f) / (2f * InteractionRadius);
        waveGrid = new Vector4(inverseSpacing * texel, texel, inverseSpacing, InteractionRadius);
        ApplyWaveTexture();
        previous?.Dispose();
    }

    private void ApplyWaveTexture()
    {
        material.SetTexture("_GpuWaveMap", waveField.HeightTexture);
        material.SetVector("_GpuWaveGrid", ActiveWaveGrid);
#if MIRA_WATER_HDRP
        hdrpWaveTextureDirty = true;
#endif
    }

    private void Strike(Vector2 point, float strength) => StrikeWave(point, strength, 0.65f);

    private void StrikeWave(Vector2 point, float strength, float width)
    {
        waveField.Add(point, width * 1.7f, strength * 3.2f);
    }

    private void SelectMode(int next)
    {
        if (next == mode)
            return;
#if MIRA_WATER_HDRP
        if (next == HdrpMode)
            lastBuiltinMode = mode;
#endif
        mode = next;
        ApplyMaterial();
    }

    private void ApplyMaterial()
    {
        material.SetColor("_Tint", Current.Tint);
        material.SetFloat("_Optical", mode >= 2 ? 1f : 0f);
        material.SetFloat("_WaveContrast", mode == 1 ? 1.25f : 0.80f);
        material.SetFloat("_Extinction", Current.Extinction);
        material.SetFloat(NoiseVariant, mode == NoiseMode ? noiseVariant : 0f);
        material.SetFloat(NoiseAmount, noiseAmount);
    }

    private void SelectLiquid(int index)
    {
        if (liquidIndex != index)
            noiseVariant = index == 0 ? 1 : 2;
        liquidIndex = index;
        ApplyMaterial();
    }

    private float EffectiveDensityRatio() => Current.Buoyancy * (floating ? 1.7f : 0.68f);

    private void ConfigureBall(float nextRadius, bool nextFloating)
    {
        radius = Mathf.Clamp(nextRadius, MinRadius, MaxRadius);
        floating = nextFloating;
        ParkSphere();
        sphere.transform.localScale = Vector3.one * (2f * radius);
        sphere.mass = (4f / 3f) * Mathf.PI * radius * radius * radius * (floating ? 0.6f : 1.5f);
    }

    private void Drop()
    {
        ParkSphere();
        falling = true;
        releaseTime = Time.time;
        sphere.isKinematic = false;
        sphere.WakeUp();
    }

    private void ParkSphere()
    {
        if (!sphere.isKinematic)
        {
            sphere.linearVelocity = Vector3.zero;
            sphere.angularVelocity = Vector3.zero;
        }
        sphere.isKinematic = true;
        sphereRenderer.enabled = true;
        sphere.position = new Vector3(0.35f, 2.18f + radius, 0.35f);
        sphere.rotation = Quaternion.identity;
        falling = false;
        touched = false;
    }

    private void ResetLab()
    {
        ParkSphere();
        waveField.Clear();
        ApplyWaveTexture();
        ClearRain();
    }

    private void PositionCamera()
    {
        if (poincareOverview)
        {
            float halfViewTangent = Mathf.Tan(0.5f * view.fieldOfView * Mathf.Deg2Rad) * Mathf.Min(1f, view.aspect);
            float overviewDistance = WaterLabSeabed.PoincareRadius * 1.14f / halfViewTangent;
            Quaternion overhead = Quaternion.Euler(90f, yaw, 0f);
            view.farClipPlane = overviewDistance + 100f;
            view.transform.SetPositionAndRotation(Vector3.up * overviewDistance, overhead);
            return;
        }
        Quaternion rotation = Quaternion.Euler(pitch, yaw, 0f);
        Vector3 target = new(0f, 0.12f, 0.75f);
        view.transform.SetPositionAndRotation(target - rotation * Vector3.forward * distance, rotation);
    }

    private void BuildSurface()
    {
        var points = new List<Vector3>(RenderResolution * RenderResolution + 16);
        var indices = new List<int>((RenderResolution - 1) * (RenderResolution - 1) * 6 + 24);
        float spacing = 2f * InteractionRadius / (RenderResolution - 1);
        for (int z = 0; z < RenderResolution; z++)
        for (int x = 0; x < RenderResolution; x++)
        {
            points.Add(new Vector3(x * spacing - InteractionRadius, 0f, z * spacing - InteractionRadius));
            if (x < RenderResolution - 1 && z < RenderResolution - 1)
            {
                int i = z * RenderResolution + x;
                AddQuadIndices(indices, i, i + RenderResolution, i + 1, i + RenderResolution + 1);
            }
        }

        // Flat geometry beyond the simulation fills the view under the sky.
        const float far = 260f;
        AddApron(points, indices, -far, -InteractionRadius, -far, far);
        AddApron(points, indices, InteractionRadius, far, -far, far);
        AddApron(points, indices, -InteractionRadius, InteractionRadius, -far, -InteractionRadius);
        AddApron(points, indices, -InteractionRadius, InteractionRadius, InteractionRadius, far);

        mesh = new Mesh
        {
            name = "Laboratory water to the horizon",
            indexFormat = UnityEngine.Rendering.IndexFormat.UInt32
        };
        mesh.SetVertices(points);
        mesh.SetTriangles(indices, 0);
        mesh.RecalculateNormals();
        mesh.bounds = new Bounds(Vector3.zero, new Vector3(2f * far, 12f, 2f * far));
        var water = new GameObject("Water surface");
        water.transform.SetParent(transform, false);
        water.AddComponent<MeshFilter>().sharedMesh = mesh;
        var renderer = water.AddComponent<MeshRenderer>();
        renderer.sharedMaterial = material;
#if MIRA_WATER_HDRP
        laboratoryWaterRenderer = renderer;
#endif
    }

    private static void AddApron(List<Vector3> points, List<int> indices, float left, float right, float near, float far)
    {
        int i = points.Count;
        points.Add(new Vector3(left, 0, near));
        points.Add(new Vector3(left, 0, far));
        points.Add(new Vector3(right, 0, near));
        points.Add(new Vector3(right, 0, far));
        AddQuadIndices(indices, i, i + 1, i + 2, i + 3);
    }

    private static void AddQuadIndices(List<int> indices, int a, int b, int c, int d)
    {
        indices.Add(a);
        indices.Add(b);
        indices.Add(c);
        indices.Add(c);
        indices.Add(b);
        indices.Add(d);
    }

    private void OnDestroy()
    {
#if MIRA_WATER_HDRP
        DisposeHdrp();
#endif
        DisposeLighting();
        DisposeSeabed();
        rainSurface?.Dispose();
        rainStreaks?.Dispose();
        waveField?.Dispose();
        if (mesh != null) Destroy(mesh);
        if (material != null) Destroy(material);
    }
}
