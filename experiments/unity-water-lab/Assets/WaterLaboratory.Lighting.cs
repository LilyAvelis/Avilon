using UnityEngine;
using UnityEngine.Rendering;
using UnityEngine.SceneManagement;

public sealed partial class WaterLaboratory
{
    public const float MinSunElevation = -18f;
    public const float MaxSunElevation = 85f;

    private static readonly int LightDirectionProperty = Shader.PropertyToID("_Sun");
    private static readonly int HorizonProperty = Shader.PropertyToID("_Horizon");
    private static readonly int ZenithProperty = Shader.PropertyToID("_Zenith");
    private static readonly int SunColorProperty = Shader.PropertyToID("_SunColor");
    private static readonly int SunPowerProperty = Shader.PropertyToID("_SunPower");
    private static readonly Vector3 DefaultSunDirection = new Vector3(-0.3f, 0.55f, 0.38f).normalized;

    private static readonly Color DuskHorizon = new Color32(185, 115, 102, 255);
    private static readonly Color DuskZenith = new Color32(45, 56, 93, 255);
    private static readonly Color DuskSun = new Color32(255, 141, 83, 255);
    private static readonly Color NightHorizon = new Color32(26, 39, 62, 255);
    private static readonly Color NightZenith = new Color32(7, 14, 31, 255);
    private static readonly Color NightSun = new Color32(134, 159, 210, 255);
    private static readonly Color DuskAmbientSky = new Color32(76, 75, 103, 255);
    private static readonly Color DuskAmbientEquator = new Color32(91, 66, 69, 255);
    private static readonly Color DuskAmbientGround = new Color32(49, 45, 61, 255);
    private static readonly Color NightAmbientSky = new Color32(33, 47, 80, 255);
    private static readonly Color NightAmbientEquator = new Color32(24, 35, 59, 255);
    private static readonly Color NightAmbientGround = new Color32(16, 24, 42, 255);

    private Vector3 sunDirection = DefaultSunDirection;
    private Light sceneSun;
    private Quaternion daySunRotation;
    private Color dayHorizon, dayZenith, daySunColor;
    private Color dayAmbientSky, dayAmbientEquator, dayAmbientGround, dayFog;
    private float daySunIntensity, dayAmbientIntensity;
    private bool dayFogEnabled;
    private FogMode dayFogMode;
    private float dayFogDensity;
    private Material lightingSkybox;
    private CameraClearFlags initialClearFlags;
    private MeshRenderer skyBackdrop;
    private Mesh skyMesh;
    private bool lightingInitialized;

    public float SunElevation => Mathf.Asin(Mathf.Clamp(sunDirection.y, -1f, 1f)) * Mathf.Rad2Deg;
    private float SunAzimuth => Mathf.Repeat(Mathf.Atan2(sunDirection.x, sunDirection.z) * Mathf.Rad2Deg, 360f);

    private void InitializeLighting()
    {
        if (lightingInitialized)
            return;

        sceneSun = RenderSettings.sun;
        initialClearFlags = view.clearFlags;
        Material sourceSkybox = RenderSettings.skybox;
        if (sourceSkybox != null)
        {
            lightingSkybox = new Material(sourceSkybox) { name = "Water Lab Sky (runtime)" };
            CreateSkyBackdrop();
            SetBuiltInSkyVisible(true);
        }

        dayHorizon = material.GetColor(HorizonProperty);
        dayZenith = material.GetColor(ZenithProperty);
        dayAmbientSky = RenderSettings.ambientSkyColor;
        dayAmbientEquator = RenderSettings.ambientEquatorColor;
        dayAmbientGround = RenderSettings.ambientGroundColor;
        dayAmbientIntensity = RenderSettings.ambientIntensity;
        dayFog = RenderSettings.fogColor;
        dayFogEnabled = RenderSettings.fog;
        dayFogMode = RenderSettings.fogMode;
        dayFogDensity = RenderSettings.fogDensity;
        RenderSettings.fog = true;
        RenderSettings.fogMode = FogMode.ExponentialSquared;
        RenderSettings.fogDensity = 0.018f;
        daySunColor = new Color32(255, 233, 207, 255);
        daySunIntensity = 1f;

        if (sceneSun != null)
        {
            daySunRotation = sceneSun.transform.rotation;
            daySunColor = sceneSun.color;
            daySunIntensity = sceneSun.intensity;
            sunDirection = -sceneSun.transform.forward;
        }
        else
        {
            Vector3 savedDirection = material.GetVector(LightDirectionProperty);
            if (savedDirection.sqrMagnitude > 0.0001f)
                sunDirection = savedDirection.normalized;
        }

        lightingInitialized = true;
        ApplyLighting();
    }

    private void CreateSkyBackdrop()
    {
        var backdrop = new GameObject("Procedural sky");
        backdrop.transform.SetParent(view.transform, false);
        skyMesh = new Mesh { name = "Sky cube" };
        skyMesh.vertices = new[]
        {
            new Vector3(-1, -1, -1), new Vector3(1, -1, -1), new Vector3(1, 1, -1), new Vector3(-1, 1, -1),
            new Vector3(-1, -1,  1), new Vector3(1, -1,  1), new Vector3(1, 1,  1), new Vector3(-1, 1,  1)
        };
        skyMesh.triangles = new[]
        {
            0, 1, 2, 0, 2, 3, 4, 6, 5, 4, 7, 6, 0, 4, 5, 0, 5, 1,
            1, 5, 6, 1, 6, 2, 2, 6, 7, 2, 7, 3, 3, 7, 4, 3, 4, 0
        };
        skyMesh.bounds = new Bounds(Vector3.zero, Vector3.one * 32f);
        backdrop.AddComponent<MeshFilter>().sharedMesh = skyMesh;
        skyBackdrop = backdrop.AddComponent<MeshRenderer>();
        skyBackdrop.sharedMaterial = lightingSkybox;
        skyBackdrop.shadowCastingMode = ShadowCastingMode.Off;
        skyBackdrop.receiveShadows = false;
        skyBackdrop.lightProbeUsage = LightProbeUsage.Off;
        skyBackdrop.reflectionProbeUsage = ReflectionProbeUsage.Off;
    }

    private void SetBuiltInSkyVisible(bool visible)
    {
        if (skyBackdrop == null)
            return;
        skyBackdrop.enabled = visible;
        view.clearFlags = visible ? CameraClearFlags.SolidColor : initialClearFlags;
    }

    public void AimSunFromCamera()
    {
        if (view == null)
            return;

        // The orbit camera faces its pivot, so its backward axis points from that pivot to the eye.
        Vector3 direction = -view.transform.forward;
        Vector2 horizontal = new(direction.x, direction.z);
        float heading = horizontal.sqrMagnitude > 0.0001f
            ? Mathf.Atan2(horizontal.x, horizontal.y) * Mathf.Rad2Deg
            : SunAzimuth;
        SetSunPosition(heading, Mathf.Asin(Mathf.Clamp(direction.y, -1f, 1f)) * Mathf.Rad2Deg);
    }

    public void SetSunElevation(float degrees)
    {
        SetSunPosition(SunAzimuth, degrees);
    }

    private void SetSunPosition(float azimuth, float degrees)
    {
        float radians = Mathf.Clamp(degrees, MinSunElevation, MaxSunElevation) * Mathf.Deg2Rad;
        float heading = azimuth * Mathf.Deg2Rad;
        float horizontal = Mathf.Cos(radians);
        ChangeSunDirection(new Vector3(Mathf.Sin(heading) * horizontal, Mathf.Sin(radians), Mathf.Cos(heading) * horizontal));
    }

    private void ChangeSunDirection(Vector3 direction)
    {
        if (!lightingInitialized || direction.sqrMagnitude < 0.0001f ||
            (direction - sunDirection).sqrMagnitude < 0.00000001f)
            return;

        sunDirection = direction.normalized;
        if (sceneSun != null)
            sceneSun.transform.rotation = Quaternion.LookRotation(-sunDirection, Vector3.up);
        ApplyLighting();
    }

    private void ApplyLighting()
    {
        float elevation = SunElevation;
        float day = Mathf.SmoothStep(0f, 1f, Mathf.InverseLerp(1f, 29f, elevation));
        float twilight = Mathf.SmoothStep(0f, 1f, Mathf.InverseLerp(-14f, -1f, elevation));
        float sunPower = Mathf.SmoothStep(0f, 1f, Mathf.InverseLerp(-2f, 26f, elevation));
        Color horizon = DaylightColor(NightHorizon, DuskHorizon, dayHorizon, twilight, day);
        Color zenith = DaylightColor(NightZenith, DuskZenith, dayZenith, twilight, day);
        Color sunlight = DaylightColor(NightSun, DuskSun, daySunColor, twilight, day);
        ApplyMaterialLighting(material, horizon, zenith, sunlight, sunPower);
        if (seabedMaterial != null)
            ApplyMaterialLighting(seabedMaterial, horizon, zenith, sunlight, sunPower);
        if (lightingSkybox != null)
            ApplyMaterialLighting(lightingSkybox, horizon, zenith, sunlight, sunPower);
        if (sceneSun != null)
        {
            sceneSun.color = sunlight;
            sceneSun.intensity = daySunIntensity * sunPower;
        }

        RenderSettings.ambientSkyColor = DaylightColor(NightAmbientSky, DuskAmbientSky, dayAmbientSky, twilight, day);
        RenderSettings.ambientEquatorColor = DaylightColor(NightAmbientEquator, DuskAmbientEquator, dayAmbientEquator, twilight, day);
        RenderSettings.ambientGroundColor = DaylightColor(NightAmbientGround, DuskAmbientGround, dayAmbientGround, twilight, day);
        RenderSettings.ambientIntensity = dayAmbientIntensity * Mathf.Lerp(0.45f, 1f, Mathf.Max(day, 0.7f * twilight));
        RenderSettings.fogColor = horizon;
    }

    private static Color DaylightColor(Color night, Color dusk, Color day, float twilight, float daylight)
        => Color.Lerp(Color.Lerp(night, dusk, twilight), day, daylight);

    private void ApplyMaterialLighting(Material target, Color horizon, Color zenith, Color sunlight, float power)
    {
        target.SetVector(LightDirectionProperty, sunDirection);
        target.SetColor(HorizonProperty, horizon);
        target.SetColor(ZenithProperty, zenith);
        if (target.HasProperty(SunColorProperty))
            target.SetColor(SunColorProperty, sunlight);
        if (target.HasProperty(SunPowerProperty))
            target.SetFloat(SunPowerProperty, power);
    }

    private void DisposeLighting()
    {
        if (!lightingInitialized)
            return;

        if (sceneSun != null)
        {
            sceneSun.transform.rotation = daySunRotation;
            sceneSun.color = daySunColor;
            sceneSun.intensity = daySunIntensity;
        }
        if (view != null)
            view.clearFlags = initialClearFlags;
        if (gameObject.scene == SceneManager.GetActiveScene())
        {
            RenderSettings.ambientSkyColor = dayAmbientSky;
            RenderSettings.ambientEquatorColor = dayAmbientEquator;
            RenderSettings.ambientGroundColor = dayAmbientGround;
            RenderSettings.ambientIntensity = dayAmbientIntensity;
            RenderSettings.fogColor = dayFog;
            RenderSettings.fog = dayFogEnabled;
            RenderSettings.fogMode = dayFogMode;
            RenderSettings.fogDensity = dayFogDensity;
        }
        if (lightingSkybox != null)
            Destroy(lightingSkybox);
        if (skyBackdrop != null)
            Destroy(skyBackdrop.gameObject);
        if (skyMesh != null)
            Destroy(skyMesh);
        lightingInitialized = false;
    }
}
