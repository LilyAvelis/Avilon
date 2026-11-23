using UnityEngine;

public sealed partial class WaterLaboratory
{
    private const float FogExitDistance = 66f;
    private static readonly int Wind = Shader.PropertyToID("_Wind");
    private static readonly int RainEnabled = Shader.PropertyToID("_RainEnabled");
    private static readonly int RainIntensity = Shader.PropertyToID("_RainIntensity");

    public Material rainMaterial;

    private readonly WaterLabRain rain = new(InteractionRadius);
    private WaterLabRainSurface rainSurface;
    private WaterLabRainStreaks rainStreaks;
    private Renderer sphereRenderer;
    private Vector2 windDirection = new(0.80f, 0.60f);
    private float windSpeed = 3.0f;
    private float windTravel;
    private float rainRate;
    private bool physicalRain;
    private bool appliedPhysicalRain;
    private float meanDropDiameter = 0.032f;
    private float dropDiameterDeviation = 0.006f;
    private bool renderingRain;

    private float WindHeading => Mathf.Repeat(Mathf.Atan2(windDirection.x, windDirection.y) * Mathf.Rad2Deg, 360f);

    private void SetWindHeading(float degrees)
    {
        float radians = degrees * Mathf.Deg2Rad;
        windDirection = new Vector2(Mathf.Sin(radians), Mathf.Cos(radians));
    }

    private void AimWindFromCamera()
    {
        Vector3 forward = view.transform.forward;
        windDirection = new Vector2(forward.x, forward.z).normalized;
    }

    private void InitializeWeather()
    {
        sphereRenderer = sphere.GetComponent<Renderer>();
        rainSurface = new WaterLabRainSurface(material, InteractionRadius);
#if MIRA_WATER_HDRP
        const bool attachRain = false;
#else
        const bool attachRain = true;
#endif
        rainStreaks = new WaterLabRainStreaks(view, rainMaterial != null ? rainMaterial.color : Color.white, attachRain);
        ClearRain();
    }

    private void UpdateWeather()
    {
        windTravel += windSpeed * 0.13f * Time.deltaTime;
        material.SetVector(Wind, new Vector4(windDirection.x, windDirection.y, windSpeed, windTravel));
        SynchronizeRainStyle();
        bool enabled = rainRate > 0f;
        if (renderingRain != enabled)
        {
            renderingRain = enabled;
            if (!enabled)
                ClearRain();
        }
        material.SetFloat(RainEnabled, enabled && !physicalRain ? 1f : 0f);
        if (!enabled)
            return;

        material.SetFloat(RainIntensity, rain.Intensity);
        if (!physicalRain)
            rainSurface.Publish();
#if MIRA_WATER_HDRP
        if (hdrpActive)
            hdrpRainStreaks.Draw(rain.Falling, Time.timeAsDouble, waveField.HeightTexture, ActiveWaveGrid);
        else
#endif
            rainStreaks?.Draw(rain.Falling, Time.timeAsDouble, waveField.HeightTexture, ActiveWaveGrid);
    }

    private void StepRain(double now)
    {
        SynchronizeRainStyle();
        rain.Step(now, rainRate, physicalRain ? meanDropDiameter : WaterLabRain.DefaultDiameter,
            physicalRain ? dropDiameterDeviation : 0f);
        while (rain.TryLand(now, out WaterLabRain.Drop drop))
        {
            if (physicalRain)
                waveField.Add(drop.Point, drop.Diameter, WaterLabRain.ImpactImpulse(drop.Diameter));
            else
                rainSurface.Add(drop);
        }
    }

    private void SynchronizeRainStyle()
    {
        if (appliedPhysicalRain == physicalRain)
            return;
        appliedPhysicalRain = physicalRain;
        ClearRain();
    }

    private void ClearRain()
    {
        rainSurface.Clear();
        rainStreaks?.Clear();
#if MIRA_WATER_HDRP
        hdrpRainStreaks?.Clear();
#endif
        rain.Reset(Time.timeAsDouble, rainRate, physicalRain ? meanDropDiameter : WaterLabRain.DefaultDiameter,
            physicalRain ? dropDiameterDeviation : 0f);
    }

    private Vector3 SurfaceCurrent()
    {
        float response = 0.14f * windSpeed * Mathf.Min(1f, 4f / Current.Drag);
        return new Vector3(windDirection.x, 0f, windDirection.y) * response;
    }

    private bool HideSphereInFog(Vector3 position)
    {
        if (new Vector2(position.x, position.z).sqrMagnitude < FogExitDistance * FogExitDistance)
            return false;
        sphereRenderer.enabled = false;
        sphere.linearVelocity = Vector3.zero;
        sphere.isKinematic = true;
        falling = false;
        return true;
    }
}
