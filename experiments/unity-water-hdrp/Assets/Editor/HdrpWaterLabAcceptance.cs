using System;
using System.IO;
using System.Reflection;
using UnityEditor;
using UnityEditor.SceneManagement;
using UnityEngine;
using UnityEngine.Rendering;
using UnityEngine.Rendering.HighDefinition;

[InitializeOnLoad]
public static class HdrpWaterLabAcceptance
{
    private const BindingFlags Private = BindingFlags.Instance | BindingFlags.NonPublic;
    private static readonly string Output = Environment.GetEnvironmentVariable("MIRA_WATER_HDRP_CAPTURE");
    private static readonly Vector2 Impact = new(4.25f, -3.7f);
    private static int phase;
    private static float captureAfter;
    private static Vector3 startingOrbit;
    private static AmbientMode startAmbientMode;
    private static Material startSkybox;

    static HdrpWaterLabAcceptance()
    {
        if (Output != null)
            EditorApplication.update += Tick;
    }

    public static void Start()
    {
        if (Output == null)
            throw new InvalidOperationException("Set MIRA_WATER_HDRP_CAPTURE before running acceptance.");
        Directory.CreateDirectory(Output);
        EditorSceneManager.OpenScene(HdrpWaterLabBuild.Scene);
        EditorApplication.EnterPlaymode();
    }

    private static void Tick()
    {
        if (!EditorApplication.isPlaying || Time.time < Math.Max(0.75f, captureAfter))
            return;

        try
        {
            var lab = UnityEngine.Object.FindFirstObjectByType<WaterLaboratory>();
            if (phase == 0)
            {
                Require(GraphicsSettings.currentRenderPipeline == null, "The lab did not start in the original renderer.");
                VerifyPanelLayout(lab);
                Set(lab, "proceduralSeabed", true);
                Set(lab, "seabedProfileIndex", (int)SeabedProfile.Pool);
                Set(lab, "windSpeed", 0f);
                Call(lab, "UpdateWeather");
                Call(lab, "UpdateSeabed");
                startingOrbit = lab.view.transform.position;
                VerifyBuiltInSky(lab);
                VerifyPoincareHorizon(lab);
                Capture(lab.view, "water-experience-03");
                startAmbientMode = RenderSettings.ambientMode;
                startSkybox = RenderSettings.skybox;
                Call(lab, "StrikeWave", Impact, 2.9f, 1.1f);
                Call(lab, "SelectMode", 4);
                captureAfter = Time.time + 0.15f;
                phase++;
                return;
            }
            if (phase == 1)
            {
                Capture(lab.view, "water-experience-05");
                Require(RenderPipelineManager.currentPipeline is HDRenderPipeline, "The fifth mode did not select HDRP.");
                Require(!((MeshRenderer)Read(lab, "skyBackdrop")).enabled, "The Built-in sky is still drawn in HDRP.");
                Require(Vector3.Distance(startingOrbit, lab.view.transform.position) < .001f, "Changing renderer moved the camera.");
                Require(lab.hdrpSurface.GetDeformationBuffer() != null, "HDRP has no deformation buffer.");
                VerifyImpact(lab.hdrpSurface);
                Call(lab, "ResetLab");
                Set(lab, "windSpeed", 5f);
                Set(lab, "seabedProfileIndex", (int)SeabedProfile.Poincare);
                Set(lab, "pitch", 31f);
                captureAfter = Time.time + 0.25f;
                phase++;
                return;
            }
            if (phase == 2)
            {
                Capture(lab.view, "water-hdrp-poincare");
                Set(lab, "seabedProfileIndex", (int)SeabedProfile.Spectre);
                captureAfter = Time.time + 0.2f;
                phase++;
                return;
            }
            if (phase == 3)
            {
                Require(RenderSettings.sun.intensity < 10f, "The HDRP light was reset to a daylight intensity.");
                Capture(lab.view, "water-hdrp-spectre");
                Set(lab, "rainRate", 15f);
                Set(lab, "seabedProfileIndex", (int)SeabedProfile.Pool);
                captureAfter = Time.time + 2f;
                phase++;
                return;
            }
            if (phase == 4)
            {
                Capture(lab.view, "water-hdrp-rain");
                Require(lab.hdrpImpacts.gameObject.activeSelf && !lab.hdrpSurface.scriptInteractions,
                    "Rain should use the decal without requesting readback for a parked ball.");
                VerifyRain(lab.hdrpSurface, "rain", 0.0004f);
                Set(lab, "rainRate", WaterLabRain.MaximumIntensity);
                Set(lab, "meanDropDiameter", 0.24f);
                Set(lab, "dropDiameterDeviation", 0.055f);
                Call(lab, "ResetLab");
                captureAfter = Time.time + 2f;
                phase++;
                return;
            }
            if (phase == 5)
            {
                Capture(lab.view, "water-hdrp-hail");
                VerifyRain(lab.hdrpSurface, "hail", 0.003f);
                TimeRender(lab.view, "HDRP active deformation");
                Set(lab, "rainRate", 0f);
                Call(lab, "ResetLab");
                captureAfter = Time.time + 0.2f;
                phase++;
                return;
            }
            if (phase == 6)
            {
                Require(!lab.hdrpImpacts.gameObject.activeSelf, "A calm surface still runs the deformation passes.");
                TimeRender(lab.view, "HDRP dry");
                lab.hdrpImpacts.gameObject.SetActive(true);
                lab.hdrpImpacts.RequestUpdate();
                lab.hdrpSurface.scriptInteractions = true;
                try
                {
                    TimeRender(lab.view, "HDRP dry with old decal and readback settings");
                }
                finally
                {
                    lab.hdrpImpacts.gameObject.SetActive(false);
                    lab.hdrpSurface.scriptInteractions = false;
                }
                Call(lab, "SelectMode", 3);
                captureAfter = Time.time + 0.12f;
                phase++;
                return;
            }
            if (phase == 7)
            {
                Require(GraphicsSettings.currentRenderPipeline == null && !lab.hdrpSurface.gameObject.activeSelf,
                    "Leaving the fifth mode did not restore the original renderer.");
                Require(((MeshRenderer)Read(lab, "skyBackdrop")).enabled, "The Built-in sky did not return.");
                Capture(lab.view, "water-experience-04-restored");
                Require(RenderSettings.ambientMode == startAmbientMode && RenderSettings.skybox == startSkybox,
                    $"HDRP changed the Built-in lighting from {startAmbientMode}/{startSkybox} " +
                    $"to {RenderSettings.ambientMode}/{RenderSettings.skybox}.");
                TimeRender(lab.view, "Built-in dry");
                Call(lab, "SelectMode", 4);
                captureAfter = Time.time + 0.15f;
                phase++;
                return;
            }
            if (phase == 8)
            {
                Capture(lab.view, "water-experience-05-reselected");
                Require(RenderPipelineManager.currentPipeline is HDRenderPipeline, "HDRP did not restart on the second selection.");
                lab.enabled = false;
                Require(!lab.hdrpSurface.gameObject.activeSelf && RenderSettings.skybox == startSkybox,
                    "Disabling the laboratory left HDRP objects or lighting in the scene.");
                lab.enabled = true;
                captureAfter = Time.time + 0.2f;
                phase++;
                return;
            }
            Require(RenderPipelineManager.currentPipeline is HDRenderPipeline, "HDRP did not restart after reenabling.");
            Debug.Log("MIRA WATER RENDERER ROUND TRIP OK");
            EditorApplication.Exit(0);
        }
        catch (Exception error)
        {
            Debug.LogException(error);
            EditorApplication.Exit(1);
        }
    }

    private static void VerifyPanelLayout(WaterLaboratory lab)
    {
        var arrange = typeof(WaterLaboratory).GetMethod("LayoutPanels", Private);
        object Layout(float width, float height) => arrange.Invoke(lab, new object[] { width, height });
        Rect Area(object layout, string panel) => (Rect)layout.GetType().GetField(panel).GetValue(layout);
        var desktop = Layout(1280f, 820f);
        Rect experiments = Area(desktop, "Experiments"), hdrp = Area(desktop, "Hdrp");
        Require(hdrp.xMin > experiments.xMax && hdrp.yMin == experiments.yMin &&
            hdrp.xMax < Area(desktop, "Weather").xMin && hdrp.height == 41f,
            "The collapsed HDRP panel is not beside the four experiments.");

        object originalPanel = Read(lab, "compactPanel");
        Set(lab, "hdrpPanelOpen", true);
        try
        {
            var expanded = Layout(3840f, 2160f);
            Require(Area(expanded, "Hdrp").height == 128f &&
                Area(expanded, "Experiments").height == experiments.height,
                "Expanding HDRP should not resize the original panel.");
            var narrow = Layout(720f, 880f);
            Require(Area(narrow, "Experiments").width > 0f && Area(narrow, "Hdrp").width == 0f,
                "The compact experiments tab should show only the original panel.");
            Set(lab, "compactPanel", Enum.Parse(originalPanel.GetType(), "Hdrp"));
            narrow = Layout(720f, 880f);
            Require(Area(narrow, "Experiments").width == 0f && Area(narrow, "Hdrp").height == 128f &&
                Area(narrow, "Hdrp").yMin > Area(narrow, "Tabs").yMax &&
                Area(narrow, "Hdrp").yMax < Area(narrow, "Footer").yMin,
                "The compact HDRP tab does not fit independently.");
        }
        finally
        {
            Set(lab, "hdrpPanelOpen", false);
            Set(lab, "compactPanel", originalPanel);
        }
        Debug.Log("MIRA WATER PANEL LAYOUT OK");
    }

    private static void VerifyBuiltInSky(WaterLaboratory lab)
    {
        float pitch = (float)Read(lab, "pitch");
        float yaw = (float)Read(lab, "yaw");
        Vector3 sun = (Vector3)Read(lab, "sunDirection");
        Set(lab, "pitch", 6f);
        Set(lab, "yaw", 12f);
        Call(lab, "PositionCamera");
        Call(lab, "SetSunPosition", 18f, 63f);
        Capture(lab.view, "water-sky-day", image =>
        {
            float low = 1f, high = 0f;
            for (int y = 0; y < 5; y++)
            for (int x = 0; x < 16; x++)
            {
                float brightness = image.GetPixelBilinear((x + .5f) / 16f, .69f + y * .06f).grayscale;
                low = Mathf.Min(low, brightness);
                high = Mathf.Max(high, brightness);
            }
            Require(high - low > .15f, "Built-in sky fell back to a flat background.");
        });
        Call(lab, "SetSunPosition", 18f, 6f);
        Vector3 cameraSun = lab.view.transform.InverseTransformDirection((Vector3)Read(lab, "sunDirection"));
        float halfFov = Mathf.Tan(.5f * lab.view.fieldOfView * Mathf.Deg2Rad);
        Capture(lab.view, "water-sky-sunset", image =>
        {
            float u = .5f + cameraSun.x / (2f * cameraSun.z * halfFov * image.width / image.height);
            float v = .5f + cameraSun.y / (2f * cameraSun.z * halfFov);
            Color disk = image.GetPixelBilinear(u, v);
            Color nearby = image.GetPixelBilinear(u + .018f, v);
            Require(disk.r > nearby.r + .12f, "The sunset disk is missing at the projected sun direction.");
        });
        Call(lab, "SetSunPosition", 18f, -16f);
        Capture(lab.view, "water-sky-night");

        Set(lab, "pitch", pitch);
        Set(lab, "yaw", yaw);
        Call(lab, "ChangeSunDirection", sun);
        Call(lab, "PositionCamera");
    }

    private static void VerifyPoincareHorizon(WaterLaboratory lab)
    {
        Vector3 sun = (Vector3)Read(lab, "sunDirection");
        Set(lab, "seabedProfileIndex", (int)SeabedProfile.Poincare);
        Call(lab, "TogglePoincareOverview");
        Call(lab, "UpdateSeabed");
        Call(lab, "SetSunPosition", 18f, 4f);
        Capture(lab.view, "water-poincare-sunset", image =>
        {
            Color nearEdge = image.GetPixelBilinear(.08f, .5f);
            foreach (float x in new[] { .01f, .92f, .99f })
            {
                Color other = image.GetPixelBilinear(x, .5f);
                Require(Mathf.Abs(nearEdge.r - other.r) < .015f && Mathf.Abs(nearEdge.g - other.g) < .015f,
                    "The edge of the water plane is visible in the Poincare sunset overview.");
            }
        });
        Call(lab, "TogglePoincareOverview");
        Set(lab, "seabedProfileIndex", (int)SeabedProfile.Pool);
        Call(lab, "UpdateSeabed");
        Call(lab, "ChangeSunDirection", sun);
    }

    private static void TimeRender(Camera camera, string label)
    {
        var target = new RenderTexture(3840, 2160, 24, RenderTextureFormat.ARGB32);
        target.Create();
        var request = new RenderPipeline.StandardRequest { destination = target };
        try
        {
            const int warmup = 6;
            var samples = new double[20];
            for (int i = -warmup; i < samples.Length; i++)
            {
                var timer = System.Diagnostics.Stopwatch.StartNew();
                if (GraphicsSettings.currentRenderPipeline == null)
                {
                    camera.targetTexture = target;
                    try { camera.Render(); }
                    finally { camera.targetTexture = null; }
                }
                else
                {
                    RenderPipeline.SubmitRenderRequest(camera, request);
                }

                var fence = AsyncGPUReadback.Request(target, 0, 0, 1, 0, 1, 0, 1);
                fence.WaitForCompletion();
                Require(!fence.hasError, "Could not synchronize the 4K render.");
                if (i >= 0)
                    samples[i] = timer.Elapsed.TotalMilliseconds;
            }
            Array.Sort(samples);
            Debug.Log($"4K offscreen render + readback fence | {label} | median: {(samples[9] + samples[10]) / 2:F2} ms");
        }
        finally
        {
            target.Release();
            UnityEngine.Object.DestroyImmediate(target);
        }
    }

    private static void VerifyRain(WaterSurface surface, string label, float minimumRms)
    {
        foreach (Vector2 point in new[] { Vector2.zero, new Vector2(63f, -51f) })
        {
            Texture texture = surface.GetDeformationBuffer();
            surface.GetDecalRegion(out var center, out var area);
            int x = Mathf.FloorToInt(((point.x - center.x) / area.x + .5f) * texture.width);
            int z = Mathf.FloorToInt(((point.y - center.y) / area.y + .5f) * texture.height);
            const int width = 128;
            var request = AsyncGPUReadback.Request(texture, 0, x - width / 2, width,
                z - width / 2, width, 0, 1, TextureFormat.RFloat);
            request.WaitForCompletion();
            Require(!request.hasError, "Unable to read rain from the HDRP deformation buffer.");
            double sum = 0.0;
            float peak = 0f;
            foreach (float height in request.GetData<float>())
            {
                sum += height * height;
                peak = Mathf.Max(peak, Mathf.Abs(height));
            }
            double rms = Math.Sqrt(sum / (width * width));
            Debug.Log($"HDRP {label} at {point}: {rms * 1000:F3} mm RMS, peak {peak * 1000:F2} mm");
            Require(rms > minimumRms, $"HDRP lost the {label} deformation at {point}.");
        }
    }

    private static void VerifyImpact(WaterSurface surface)
    {
        Texture field = surface.GetDeformationBuffer();
        surface.GetDecalRegion(out var center, out var area);
        float expected = ReadPeak(field, Impact);
        float flipX = ReadPeak(field, new Vector2(-Impact.x, Impact.y));
        float flipZ = ReadPeak(field, new Vector2(Impact.x, -Impact.y));
        float ReadPeak(Texture texture, Vector2 point)
        {
            int x = Mathf.FloorToInt(((point.x - center.x) / area.x + .5f) * texture.width);
            int z = Mathf.FloorToInt(((point.y - center.y) / area.y + .5f) * texture.height);
            const int width = 33;
            var request = AsyncGPUReadback.Request(texture, 0, x - width / 2, width,
                z - width / 2, width, 0, 1, TextureFormat.RFloat);
            request.WaitForCompletion();
            Require(!request.hasError, "Unable to read the HDRP deformation buffer.");
            float peak = 0f;
            foreach (float height in request.GetData<float>())
                peak = Mathf.Max(peak, Mathf.Abs(height));
            return peak;
        }
        Debug.Log($"HDRP impact at {Impact}: intended {expected:F5} m, mirrored X {flipX:F5}, mirrored Z {flipZ:F5}");
        Require(expected > .01f && expected > 4 * Mathf.Max(flipX, flipZ), "HDRP missed or mirrored the impact.");
    }

    private static void Capture(Camera camera, string name, Action<Texture2D> inspect = null)
    {
        const int width = 1600, height = 900;
        var destination = new RenderTexture(width, height, 24, RenderTextureFormat.ARGB32)
        {
            hideFlags = HideFlags.HideAndDontSave
        };
        destination.Create();
        Texture2D image = null;
        RenderTexture old = RenderTexture.active;
        try
        {
            if (GraphicsSettings.currentRenderPipeline == null)
            {
                camera.targetTexture = destination;
                try { camera.Render(); }
                finally { camera.targetTexture = null; }
            }
            else
            {
                var request = new RenderPipeline.StandardRequest { destination = destination };
                Require(RenderPipeline.SupportsRenderRequest(camera, request), "HDRP refused the camera capture.");
                RenderPipeline.SubmitRenderRequest(camera, request);
            }
            image = new Texture2D(width, height, TextureFormat.RGB24, false);
            RenderTexture.active = destination;
            image.ReadPixels(new Rect(0, 0, width, height), 0, 0);
            image.Apply();
            inspect?.Invoke(image);
            File.WriteAllBytes(Path.Combine(Output, name + ".png"), image.EncodeToPNG());
            if (name == "water-hdrp-poincare")
                File.WriteAllBytes(Path.Combine(Output, name + ".jpg"), image.EncodeToJPG(90));
        }
        finally
        {
            RenderTexture.active = old;
            UnityEngine.Object.DestroyImmediate(destination);
            if (image != null)
                UnityEngine.Object.DestroyImmediate(image);
        }
    }

    private static void Set(WaterLaboratory lab, string member, object value)
        => typeof(WaterLaboratory).GetField(member, Private).SetValue(lab, value);

    private static object Read(WaterLaboratory lab, string member)
        => typeof(WaterLaboratory).GetField(member, Private).GetValue(lab);

    private static void Call(WaterLaboratory lab, string member, params object[] args)
        => typeof(WaterLaboratory).GetMethod(member, Private).Invoke(lab, args);

    private static void Require(bool condition, string message)
    {
        if (!condition)
            throw new InvalidOperationException(message);
    }
}
