using UnityEngine;

public sealed partial class WaterLaboratory
{
    private const float WeatherPanelHeight = 453f;
    private const float RainSizeControlsHeight = 55f;
    private const float WaterQualityControlsHeight = 56f;
    private const float SeabedPanelHeight = 87f;
    private const float SeabedDetailHeight = 171f;
    private const float ExperimentalBedControlsHeight = 36f;
    private const float ExperimentsPanelWidth = 318f;
    private const float HdrpPanelWidth = 258f;
    private const float PanelGap = 10f;
    private const int PoincareSeabedProfile = (int)SeabedProfile.Poincare;
    private const int SpectreSeabedProfile = (int)SeabedProfile.Spectre;
    private static readonly Color UiCobalt = new Color32(14, 28, 86, 247);
    private static readonly Color UiDeep = new Color32(8, 18, 58, 255);
    private static readonly Color UiGrid = new Color32(50, 69, 131, 255);
    private static readonly Color UiPaper = new Color32(231, 236, 255, 255);
    private static readonly Color UiSoft = new Color32(160, 185, 255, 255);
    private static readonly Color UiSignal = new Color32(226, 250, 128, 255);
    private static readonly string[] UiSeabedProfiles = { "Tile", "Sand", "Rock", "Mud" };
    private static readonly string[] UiModes =
    {
        "Surface", "Waves", "Optics", "Procedural Ripples",
    };
    private static readonly string[] UiModeNotes =
    {
        "Watch the wave shapes and highlights.",
        "Click two spots to see how waves overlap.",
        "Waves, depth, refraction and sky reflections.",
        "Optics, waves and irregular surface ripples.",
    };
    private float MainExperimentsHeight => (mode == NoiseMode ? 525f : 411f) + WaterQualityControlsHeight;
    private float HdrpPanelHeight
    {
        get
        {
#if MIRA_WATER_HDRP
            return hdrpPanelOpen ? 128f : 41f;
#else
            return 0f;
#endif
        }
    }

    private enum WaterPanel
    {
        None,
        Experiments,
        Weather,
        Seabed,
        Hdrp
    }

    private readonly struct WaterInterfaceLayout
    {
        public readonly float Scale;
        public readonly Rect Experiments, Hdrp, Weather, Seabed, Tabs, Footer;
        public readonly bool Compact;

        public WaterInterfaceLayout(float scale, Rect experiments, Rect hdrp, Rect weather, Rect seabed, Rect tabs, Rect footer, bool compact)
        {
            Scale = scale;
            Experiments = experiments;
            Hdrp = hdrp;
            Weather = weather;
            Seabed = seabed;
            Tabs = tabs;
            Footer = footer;
            Compact = compact;
        }
    }

    private WaterPanel compactPanel = WaterPanel.Experiments;
    private GUIStyle uiTitle, uiSection, uiText, uiNote, uiValue;
    private GUIStyle uiButton, uiActiveButton, uiModeButton, uiActiveModeButton;
    private GUIStyle uiEmptyTrack, uiEmptyThumb;

    private WaterInterfaceLayout InterfaceLayout() => LayoutPanels(Screen.width, Screen.height);

    private WaterInterfaceLayout LayoutPanels(float screenWidth, float screenHeight)
    {
        bool compact = screenWidth < 820f || screenWidth < screenHeight * 1.15f;
        float minWidth = compact ? 560f : 1040f;
        float minHeight = compact ? 690f : 886f;
        float scale = Mathf.Min(1.35f, Mathf.Min(screenWidth / minWidth, screenHeight / minHeight));
        scale = Mathf.Max(scale, 0.001f);
        float width = screenWidth / scale;
        float height = screenHeight / scale;
        float margin = compact ? 14f : 22f;
        float top = compact ? 59f : 22f;
        Rect experiments = !compact || compactPanel == WaterPanel.Experiments
            ? new Rect(margin, top, ExperimentsPanelWidth, MainExperimentsHeight)
            : default;
        Rect hdrp = HdrpPanelHeight > 0f && (!compact || compactPanel == WaterPanel.Hdrp)
            ? new Rect(compact ? margin : experiments.xMax + PanelGap, top, HdrpPanelWidth, HdrpPanelHeight)
            : default;
        float right = width - margin - 278f;
        float weatherHeight = WeatherPanelHeight + (physicalRain ? RainSizeControlsHeight : 0f);
        Rect weather = !compact || compactPanel == WaterPanel.Weather
            ? new Rect(right, top, 278f, weatherHeight)
            : default;
        float seabedHeight = SeabedPanelHeight;
        if (proceduralSeabed)
        {
            seabedHeight += SeabedDetailHeight;
            if (seabedProfileIndex is PoincareSeabedProfile or SpectreSeabedProfile)
                seabedHeight += ExperimentalBedControlsHeight;
        }
        Rect seabed = !compact || compactPanel == WaterPanel.Seabed
            ? new Rect(right, compact ? top : top + weatherHeight + PanelGap, 278f, seabedHeight)
            : default;
        Rect tabs = compact ? new Rect(margin, 14f, width - 2f * margin, 35f) : default;
        Rect footer = new(margin, height - 36f, Mathf.Min(width - 2f * margin, 647f), 24f);
        return new WaterInterfaceLayout(scale, experiments, hdrp, weather, seabed, tabs, footer, compact);
    }

    private bool PointerOnPanel()
    {
        WaterInterfaceLayout layout = InterfaceLayout();
        Vector2 mouse = Input.mousePosition;
        Vector2 point = new Vector2(mouse.x, Screen.height - mouse.y) / layout.Scale;
        return layout.Experiments.Contains(point)
            || layout.Hdrp.Contains(point)
            || layout.Weather.Contains(point)
            || layout.Seabed.Contains(point)
            || layout.Tabs.Contains(point)
            || layout.Footer.Contains(point);
    }

    private void OnGUI()
    {
        if (uiTitle == null)
            CreateInterfaceStyles();

        WaterInterfaceLayout layout = InterfaceLayout();
        Matrix4x4 previousMatrix = GUI.matrix;
        Color previousColor = GUI.color;
        GUI.matrix = Matrix4x4.Scale(new Vector3(layout.Scale, layout.Scale, 1f));
        GUI.color = Color.white;
        try
        {
            if (layout.Compact)
                DrawCompactTabs(layout.Tabs);
            if (layout.Experiments.width > 0f)
                DrawExperimentsPanel(layout.Experiments);
#if MIRA_WATER_HDRP
            if (layout.Hdrp.width > 0f)
                DrawHdrpPanel(layout.Hdrp);
#endif
            if (layout.Weather.width > 0f)
                DrawWeatherPanel(layout.Weather);
            if (layout.Seabed.width > 0f)
                DrawSeabedPanel(layout.Seabed);

            Fill(layout.Footer, UiDeep);
            GUI.Label(new Rect(layout.Footer.x + 9f, layout.Footer.y, layout.Footer.width - 18f, 24f),
                layout.Compact
                    ? "LMB — waves   ·   Space — drop   ·   RMB — orbit   ·   R — reset"
                    : "LMB — waves   ·   Space — drop   ·   RMB — orbit   ·   Wheel — zoom   ·   R — reset",
                uiNote);
        }
        finally
        {
            GUI.color = previousColor;
            GUI.matrix = previousMatrix;
        }
    }

    private void DrawCompactTabs(Rect rect)
    {
        int count = HdrpPanelHeight > 0f ? 4 : 3;
        DrawCompactTab(Segment(rect, 0f, rect.height, 0, count), "Experiments", WaterPanel.Experiments);
        DrawCompactTab(Segment(rect, 0f, rect.height, 1, count), "Weather", WaterPanel.Weather);
        DrawCompactTab(Segment(rect, 0f, rect.height, 2, count), "Seabed", WaterPanel.Seabed);
        if (count == 4)
            DrawCompactTab(Segment(rect, 0f, rect.height, 3, count), "HDRP", WaterPanel.Hdrp);
    }

    private void DrawCompactTab(Rect rect, string label, WaterPanel panel)
    {
        if (PanelButton(rect, label, compactPanel == panel))
            compactPanel = compactPanel == panel ? WaterPanel.None : panel;
    }

    private void DrawExperimentsPanel(Rect panel)
    {
        FillPanel(panel);
        Rect inner = PanelContent(panel);
        PanelTitle(inner, "WATER LABORATORY");
        GUI.Label(Row(inner, 46f, 17f), "EXPERIMENT", uiSection);

        for (int index = 0; index < UiModes.Length; index++)
        {
            Rect entry = Row(inner, 66f + index * 28f, 25f);
            if (PanelButton(entry, $"{index + 1:00}   {UiModes[index]}", index == mode, leftAligned: true))
                SelectMode(index);
        }
        string note = mode < UiModeNotes.Length ? UiModeNotes[mode] : "The HDRP water renderer is active.";
        GUI.Label(Row(inner, 178f, 36f), note, uiNote);
        DrawSeparator(inner, 221f);
        GUI.Label(Row(inner, 227f, 19f), "LIQUID", uiSection);
        for (int index = 0; index < Liquids.Length; index++)
        {
            Rect entry = Segment(inner, 251f, 28f, index, Liquids.Length);
            if (PanelButton(entry, Liquids[index].Name, index == liquidIndex))
                SelectLiquid(index);
        }

        DrawSeparator(inner, 289f);
        GUI.Label(Row(inner, 295f, 19f), "BALL", uiSection);
        Rect drop = new(inner.xMax - 110f, inner.y + 293f, 110f, 26f);
        if (PanelButton(drop, "Drop ball", false))
            Drop();

        float nextRadius = PanelSlider(inner, 322f, "Diameter", System.FormattableString.Invariant($"{radius * 2f:0.00} m"), radius, MinRadius, MaxRadius);
        bool nextFloating = floating;
        if (PanelButton(Segment(inner, 367f, 28f, 0, 2), "Sinks", !floating))
            nextFloating = false;
        if (PanelButton(Segment(inner, 367f, 28f, 1, 2), "Floats", floating))
            nextFloating = true;
        if (!Mathf.Approximately(radius, nextRadius) || floating != nextFloating)
            ConfigureBall(nextRadius, nextFloating);

        DrawSeparator(inner, 405f);
        int quality = Mathf.Clamp(requestedWaveQuality, 0, WaveQualitySizes.Length - 1);
        int gridSize = WaveQualitySizes[quality];
#if MIRA_WATER_HDRP
        string qualityLabel = hdrpActive ? "Impact grid" : "Wave resolution";
#else
        const string qualityLabel = "Wave resolution";
#endif
        requestedWaveQuality = Mathf.RoundToInt(PanelSlider(inner, 412f, qualityLabel,
            $"{gridSize} × {gridSize}", quality, 0f, WaveQualitySizes.Length - 1, stepped: true));

        if (mode == NoiseMode)
        {
            inner.y += WaterQualityControlsHeight;
            DrawNoisePanel(inner);
        }
    }

#if MIRA_WATER_HDRP
    private void DrawHdrpPanel(Rect panel)
    {
        FillPanel(panel);
        Rect inner = PanelContent(panel);
        string disclosure = hdrpPanelOpen ? "▾" : "▸";
        if (PanelButton(Row(inner, 8f, 25f), $"{disclosure}   HDRP Water", mode == HdrpMode, leftAligned: true))
            hdrpPanelOpen = !hdrpPanelOpen;
        if (!hdrpPanelOpen)
            return;

        bool active = mode == HdrpMode;
        if (PanelButton(Row(inner, 39f, 28f), active ? "Back to Built-in" : "Switch to HDRP", false))
            SelectMode(active ? lastBuiltinMode : HdrpMode);
        if (PanelButton(Segment(inner, 77f, 28f, 0, 2), "Caustics", hdrpCausticsEnabled))
            hdrpCausticsEnabled = !hdrpCausticsEnabled;
        if (PanelButton(Segment(inner, 77f, 28f, 1, 2), "SSR reflections", hdrpReflectionsEnabled))
            hdrpReflectionsEnabled = !hdrpReflectionsEnabled;
    }
#endif

    private void DrawNoisePanel(Rect inner)
    {
        DrawSeparator(inner, 405f);
        GUI.Label(Row(inner, 411f, 19f), "RIPPLES", uiSection);
        int nextVariant = noiseVariant;
        if (PanelButton(Segment(inner, 434f, 26f, 0, 2), "Layered", noiseVariant == 1))
            nextVariant = 1;
        if (PanelButton(Segment(inner, 434f, 26f, 1, 2), "Swirling", noiseVariant == 2))
            nextVariant = 2;

        float nextAmount = PanelSlider(inner, 465f, "Ripple strength", System.FormattableString.Invariant($"{noiseAmount:P0}"), noiseAmount, 0f, 1f);
        if (nextVariant != noiseVariant || !Mathf.Approximately(noiseAmount, nextAmount))
        {
            noiseVariant = nextVariant;
            noiseAmount = nextAmount;
            ApplyMaterial();
        }
    }

    private void DrawWeatherPanel(Rect panel)
    {
        FillPanel(panel);
        Rect inner = PanelContent(panel);
        PanelTitle(inner, "WEATHER");

        windSpeed = PanelSlider(inner, 46f, "Wind", windSpeed <= 0f ? "Calm" : System.FormattableString.Invariant($"{windSpeed:0.0} m/s"), windSpeed, 0f, 12f);
        float heading = WindHeading;
        float nextHeading = PanelSlider(inner, 97f, "Direction",
            $"{Mathf.FloorToInt(heading):000}°", heading, 0f, 359.99f);
        if (!Mathf.Approximately(heading, nextHeading))
            SetWindHeading(nextHeading);
        if (PanelButton(Row(inner, 145f, 28f), "Align wind with view", false))
            AimWindFromCamera();

        bool hail;
#if MIRA_WATER_HDRP
        if (hdrpActive)
        {
            if (PanelButton(Segment(inner, 182f, 28f, 0, 2), "Rain", meanDropDiameter < 0.1f))
            {
                meanDropDiameter = WaterLabRain.DefaultDiameter;
                dropDiameterDeviation = 0.006f;
                if (rainRate <= 0f)
                    rainRate = 8f;
            }
            hail = PanelButton(Segment(inner, 182f, 28f, 1, 2), "Hail", meanDropDiameter >= 0.1f);
        }
        else
#endif
        {
            if (PanelButton(Segment(inner, 182f, 28f, 0, 3), "Rings", !physicalRain))
                physicalRain = false;
            if (PanelButton(Segment(inner, 182f, 28f, 1, 3), "Physical", physicalRain))
                physicalRain = true;
            hail = PanelButton(Segment(inner, 182f, 28f, 2, 3), "Hail", false);
        }
        if (hail)
        {
            physicalRain = true;
            meanDropDiameter = 0.24f;
            dropDiameterDeviation = 0.055f;
            rainRate = Mathf.Min(8f, WaterLabRain.MaximumIntensity);
        }

        string rainValue = rainRate <= 0f ? "Off" : System.FormattableString.Invariant($"{rainRate:0.0} hits / {WaterLabRain.ReferenceArea:0} m²/s");
        string rainLabel = rainRate >= 40f ? "Downpour" : "Rain";
        float rainControl = Mathf.Sqrt(rainRate / WaterLabRain.MaximumIntensity);
        float nextRainControl = PanelSlider(inner, 219f, rainLabel, rainValue, rainControl, 0f, 1f);
        if (nextRainControl != rainControl)
            rainRate = nextRainControl * nextRainControl * WaterLabRain.MaximumIntensity;

        if (physicalRain)
            DrawRainSizeControls(inner);

        float sunTop = physicalRain ? RainSizeControlsHeight : 0f;
        DrawSeparator(inner, 269f + sunTop);
        GUI.Label(Row(inner, 281f + sunTop, 22f), "SUN", uiSection);
        float elevation = SunElevation;
        float nextElevation = PanelSlider(inner, 314f + sunTop, "Elevation",
            $"{elevation:0}°", elevation, MinSunElevation, MaxSunElevation);
        if (!Mathf.Approximately(elevation, nextElevation))
            SetSunElevation(nextElevation);
        float azimuth = SunAzimuth;
        float nextAzimuth = PanelSlider(inner, 363f + sunTop, "Azimuth", $"{Mathf.FloorToInt(azimuth):000}°",
            azimuth, 0f, 359.99f);
        if (!Mathf.Approximately(azimuth, nextAzimuth))
            SetSunPosition(nextAzimuth, SunElevation);
        if (PanelButton(Row(inner, 411f + sunTop, 28f), "Light from camera", false))
            AimSunFromCamera();
    }

    private void DrawRainSizeControls(Rect inner)
    {
        Rect diameter = Segment(inner, 270f, 40f, 0, 2);
        Rect deviation = Segment(inner, 270f, 40f, 1, 2);
        meanDropDiameter = PanelSlider(diameter, 0f, "Diameter", System.FormattableString.Invariant($"{meanDropDiameter * 100f:0.0} cm"),
            meanDropDiameter, 0.012f, 0.40f);
        dropDiameterDeviation = PanelSlider(deviation, 0f, "Spread", System.FormattableString.Invariant($"{dropDiameterDeviation * 100f:0.0} cm"),
            dropDiameterDeviation, 0f, 0.18f);
    }

    private void DrawSeabedPanel(Rect panel)
    {
        FillPanel(panel);
        Rect inner = PanelContent(panel);
        PanelTitle(inner, "SEABED");
        if (PanelButton(Segment(inner, 49f, 26f, 0, 2), "Reference bed", !proceduralSeabed))
            proceduralSeabed = false;
        if (PanelButton(Segment(inner, 49f, 26f, 1, 2), "Procedural bed", proceduralSeabed))
            proceduralSeabed = true;

        if (!proceduralSeabed)
            return;

        for (int index = 0; index < SeabedDepths.Length; index++)
        {
            Rect entry = Segment(inner, 84f, 26f, index, SeabedDepths.Length);
            if (PanelButton(entry, $"{SeabedDepths[index]} m", index == seabedDepthIndex))
                seabedDepthIndex = index;
        }

        DrawSeparator(inner, 121f);
        for (int index = 0; index < UiSeabedProfiles.Length; index++)
        {
            Rect entry = Segment(inner, 133f, 28f, index, UiSeabedProfiles.Length);
            if (PanelButton(entry, UiSeabedProfiles[index], index == seabedProfileIndex))
                seabedProfileIndex = index;
        }
        if (PanelButton(Segment(inner, 166f, 28f, 0, 2), "Poincaré", seabedProfileIndex == PoincareSeabedProfile))
            seabedProfileIndex = PoincareSeabedProfile;
        if (PanelButton(Segment(inner, 166f, 28f, 1, 2), "Spectre", seabedProfileIndex == SpectreSeabedProfile))
            seabedProfileIndex = SpectreSeabedProfile;

        float noiseTop = 206f;
        if (seabedProfileIndex == SpectreSeabedProfile)
        {
            GUI.Label(Row(inner, 202f, 26f), "Scale", uiText);
            Rect choices = new(inner.x + 84f, inner.y, inner.width - 84f, inner.height);
            if (PanelButton(Segment(choices, 202f, 26f, 0, 2), "Fine", spectreScaleIndex == 0))
                spectreScaleIndex = 0;
            if (PanelButton(Segment(choices, 202f, 26f, 1, 2), "Coarse", spectreScaleIndex == 1))
                spectreScaleIndex = 1;
            noiseTop += ExperimentalBedControlsHeight;
        }
        else if (seabedProfileIndex == PoincareSeabedProfile)
        {
            if (PanelButton(Row(inner, 202f, 26f), poincareOverview ? "Back to water" : "Disk overview", poincareOverview))
                TogglePoincareOverview();
            noiseTop += ExperimentalBedControlsHeight;
        }
        seabedNoise = PanelSlider(inner, noiseTop, "Seabed noise", System.FormattableString.Invariant($"{seabedNoise:P0}"), seabedNoise, 0f, 1f);
    }

    private static Rect PanelContent(Rect panel) => new(panel.x + 14f, panel.y, panel.width - 28f, panel.height);
    private static Rect Row(Rect inner, float y, float height) => new(inner.x, inner.y + y, inner.width, height);

    private static Rect Segment(Rect inner, float y, float height, int index, int count)
    {
        const float gap = 5f;
        float width = (inner.width - gap * (count - 1)) / count;
        return new Rect(inner.x + index * (width + gap), inner.y + y, width, height);
    }

    private void PanelTitle(Rect inner, string title)
    {
        GUI.Label(Row(inner, 7f, 28f), title, uiTitle);
        DrawSeparator(inner, 39f);
    }

    private static void FillPanel(Rect panel)
    {
        Fill(panel, UiCobalt);
        Fill(new Rect(panel.x, panel.y, panel.width, 1f), UiGrid);
        Fill(new Rect(panel.x, panel.yMax - 1f, panel.width, 1f), UiGrid);
        Fill(new Rect(panel.x, panel.y, 1f, panel.height), UiGrid);
        Fill(new Rect(panel.xMax - 1f, panel.y, 1f, panel.height), UiGrid);
        Fill(new Rect(panel.x, panel.y, 11f, 2f), UiSignal);
        Fill(new Rect(panel.x, panel.y, 2f, 11f), UiSignal);
    }

    private static void DrawSeparator(Rect rect, float y) => Fill(Row(rect, y, 1f), UiGrid);

    private bool PanelButton(Rect rect, string text, bool active, bool leftAligned = false)
    {
        bool hover = rect.Contains(Event.current.mousePosition);
        Fill(rect, active ? UiSignal : UiGrid);
        if (!active)
        {
            Rect inner = new(rect.x + 1f, rect.y + 1f, rect.width - 2f, rect.height - 2f);
            Fill(inner, hover ? UiGrid : UiDeep);
        }
        GUIStyle style = leftAligned
            ? (active ? uiActiveModeButton : uiModeButton)
            : (active ? uiActiveButton : uiButton);
        return GUI.Button(rect, text, style);
    }

    private float PanelSlider(Rect inner, float top, string label, string valueText, float value,
        float min, float max, bool stepped = false)
    {
        Rect labelRow = Row(inner, top, 22f);
        GUI.Label(labelRow, label, uiText);
        GUI.Label(labelRow, valueText, uiValue);
        Rect input = Row(inner, top + 24f, 16f);

        float clamped = Mathf.Clamp(value, min, max);
        float next = GUI.HorizontalSlider(input, clamped, min, max, uiEmptyTrack, uiEmptyThumb);
        if (stepped)
            next = Mathf.Round(next);
        float knob = Mathf.Lerp(input.x + 6f, input.xMax - 6f, Mathf.InverseLerp(min, max, next));
        float middle = input.center.y;
        Fill(new Rect(input.x, middle - 1f, input.width, 2f), UiGrid);
        Fill(new Rect(input.x, middle - 1f, knob - input.x, 2f), UiSoft);
        if (stepped)
        {
            for (float step = min; step <= max; step++)
            {
                float x = Mathf.Lerp(input.x + 6f, input.xMax - 6f, Mathf.InverseLerp(min, max, step));
                Fill(new Rect(x - 1f, middle - 3f, 2f, 6f), UiSoft);
            }
        }
        Fill(new Rect(knob - 5f, middle - 5f, 10f, 10f), UiSignal);
        return next == clamped ? value : next;
    }

    private void CreateInterfaceStyles()
    {
        uiTitle = TextStyle(20, UiSignal, TextAnchor.MiddleLeft, FontStyle.Bold);
        uiSection = TextStyle(12, UiSignal, TextAnchor.MiddleLeft, FontStyle.Bold);
        uiText = TextStyle(14, UiPaper, TextAnchor.MiddleLeft);
        uiNote = TextStyle(12, UiSoft, TextAnchor.MiddleLeft);
        uiNote.wordWrap = true;
        uiValue = TextStyle(13, UiSignal, TextAnchor.MiddleRight);
        uiButton = TextStyle(13, UiPaper, TextAnchor.MiddleCenter);
        uiActiveButton = TextStyle(13, UiDeep, TextAnchor.MiddleCenter, FontStyle.Bold);
        uiModeButton = TextStyle(13, UiPaper, TextAnchor.MiddleLeft);
        uiActiveModeButton = TextStyle(13, UiDeep, TextAnchor.MiddleLeft, FontStyle.Bold);
        uiModeButton.padding = new RectOffset(9, 3, 0, 0);
        uiActiveModeButton.padding = uiModeButton.padding;
        uiEmptyTrack = new GUIStyle { fixedHeight = 16f };
        uiEmptyThumb = new GUIStyle { fixedWidth = 12f, fixedHeight = 16f };
    }

    private static GUIStyle TextStyle(int size, Color ink, TextAnchor anchor, FontStyle weight = FontStyle.Normal)
    {
        var style = new GUIStyle
        {
            font = GUI.skin.font,
            fontSize = size,
            fontStyle = weight,
            alignment = anchor,
            clipping = TextClipping.Clip
        };
        style.normal.textColor = ink;
        style.hover.textColor = ink;
        style.active.textColor = ink;
        style.focused.textColor = ink;
        return style;
    }

    private static void Fill(Rect rect, Color color)
    {
        Color previous = GUI.color;
        GUI.color = color;
        GUI.DrawTexture(rect, Texture2D.whiteTexture);
        GUI.color = previous;
    }
}
