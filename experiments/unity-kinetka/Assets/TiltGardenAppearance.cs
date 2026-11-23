using UnityEngine;

public sealed class TiltGardenAppearance : MonoBehaviour
{
    public Light sun;
    public Light fill;
    public Camera view;

    private static readonly int BaseColor = Shader.PropertyToID("_BaseColor");
    private static readonly int ColorProperty = Shader.PropertyToID("_Color");
    private static readonly int SkyTop = Shader.PropertyToID("_Top");
    private static readonly int SkyBottom = Shader.PropertyToID("_Bottom");

    private static readonly Color Ink = new Color32(238, 239, 222, 255);
    private static readonly Color Muted = new Color32(181, 197, 185, 255);
    private static readonly Color Brass = new Color32(226, 189, 128, 255);
    private static readonly Color Border = new Color32(103, 123, 112, 255);
    private static readonly Color Panel = new Color32(18, 40, 37, 242);
    private static readonly Color Chip = new Color32(31, 55, 50, 255);
    private static readonly Color SelectedChip = new Color32(63, 76, 59, 255);
    private static readonly Color NightSun = new Color32(164, 191, 255, 255);
    private static readonly Color NightFill = new Color32(166, 198, 242, 255);
    private static readonly Color NightTop = new Color32(26, 39, 63, 255);
    private static readonly Color NightBottom = new Color32(12, 20, 35, 255);
    private static readonly string[] SurfaceNames = { "Песок", "Мох", "Сланец" };
    private static readonly Color[] SurfaceColors =
    {
        new Color32(199, 167, 119, 255),
        new Color32(124, 151, 103, 255),
        new Color32(120, 147, 161, 255)
    };

    private Renderer currentDeck;
    private MaterialPropertyBlock deckProperties;
    private bool night;
    private int surfaceIndex;
    private bool dayCaptured;
    private Color daySunColor;
    private Color dayFillColor;
    private float daySunIntensity;
    private float dayFillIntensity;
    private Color dayAmbientSky;
    private Color dayAmbientEquator;
    private Color dayAmbientGround;
    private Color dayCameraBackground;
    private Material sceneSky;
    private Material runtimeSky;
    private GUIStyle label;
    private GUIStyle button;
    private GUIStyle selectedButton;

    public void BindDeck(Renderer deck)
    {
        currentDeck = deck;
        ApplySurface();
    }

    public void SetNight(bool enabled)
    {
        if (night == enabled)
            return;
        CaptureDay();
        night = enabled;

        if (sun != null)
        {
            sun.color = night ? NightSun : daySunColor;
            sun.intensity = night ? daySunIntensity * 0.38f : daySunIntensity;
        }
        if (fill != null)
        {
            fill.color = night ? NightFill : dayFillColor;
            fill.intensity = night ? dayFillIntensity * 0.69f : dayFillIntensity;
        }
        RenderSettings.ambientSkyColor = night ? new Color(0.25f, 0.32f, 0.47f) : dayAmbientSky;
        RenderSettings.ambientEquatorColor = night ? new Color(0.18f, 0.24f, 0.36f) : dayAmbientEquator;
        RenderSettings.ambientGroundColor = night ? new Color(0.12f, 0.15f, 0.23f) : dayAmbientGround;
        if (view != null)
            view.backgroundColor = night ? NightBottom : dayCameraBackground;
        if (runtimeSky != null)
            RenderSettings.skybox = night ? runtimeSky : sceneSky;
    }

    private void CaptureDay()
    {
        if (dayCaptured)
            return;
        dayCaptured = true;
        if (sun != null)
        {
            daySunColor = sun.color;
            daySunIntensity = sun.intensity;
        }
        if (fill != null)
        {
            dayFillColor = fill.color;
            dayFillIntensity = fill.intensity;
        }
        dayAmbientSky = RenderSettings.ambientSkyColor;
        dayAmbientEquator = RenderSettings.ambientEquatorColor;
        dayAmbientGround = RenderSettings.ambientGroundColor;
        if (view != null)
            dayCameraBackground = view.backgroundColor;

        sceneSky = RenderSettings.skybox;
        if (sceneSky == null || !sceneSky.HasProperty(SkyTop) || !sceneSky.HasProperty(SkyBottom))
            return;
        runtimeSky = new Material(sceneSky) { name = "Garden sky · Night" };
        runtimeSky.SetColor(SkyTop, NightTop);
        runtimeSky.SetColor(SkyBottom, NightBottom);
    }

    public void SetSurface(int index)
    {
        if (index < 0 || index >= SurfaceColors.Length || index == surfaceIndex)
            return;
        surfaceIndex = index;
        ApplySurface();
    }

    private void ApplySurface()
    {
        if (currentDeck == null || currentDeck.sharedMaterial == null)
            return;
        Material material = currentDeck.sharedMaterial;
        int property = material.HasProperty(BaseColor) ? BaseColor : ColorProperty;
        if (!material.HasProperty(property))
            return;

        deckProperties ??= new MaterialPropertyBlock();
        currentDeck.GetPropertyBlock(deckProperties);
        deckProperties.SetColor(property, SurfaceColors[surfaceIndex]);
        currentDeck.SetPropertyBlock(deckProperties);
    }

    public void Draw(Rect area)
    {
        if (label == null)
            MakeStyles();

        Box(area, Panel);
        Box(new Rect(area.x, area.yMax - 1f, area.width, 1f), Border);
        float unit = area.width / 440f;
        float top = area.center.y - 15f;
        Rect Item(float x, float width) => new(area.x + x * unit, top, width * unit, 30f);

        GUI.Label(Item(9f, 38f), "Свет", label);
        if (Control(Item(48f, 47f), "День", !night))
            SetNight(false);
        if (Control(Item(99f, 49f), "Ночь", night))
            SetNight(true);
        Box(new Rect(area.x + 155f * unit, top + 6f, 1f, 18f), Border);
        GUI.Label(Item(164f, 70f), "Покрытие", label);
        for (int i = 0; i < SurfaceNames.Length; i++)
        {
            if (Control(Item(233f + i * 67f, 63f), SurfaceNames[i], surfaceIndex == i, i))
                SetSurface(i);
        }
    }

    private bool Control(Rect rect, string text, bool selected, int swatch = -1)
    {
        Box(rect, selected ? Brass : Border);
        var inner = new Rect(rect.x + 1f, rect.y + 1f, rect.width - 2f, rect.height - 2f);
        Color background = selected ? SelectedChip : Chip;
        if (rect.Contains(Event.current.mousePosition))
            background = Color.Lerp(background, Ink, 0.16f);
        Box(inner, background);
        if (swatch >= 0)
            Box(new Rect(rect.x + 5f, rect.center.y - 3f, 6f, 6f), SurfaceColors[swatch]);
        bool pressed = GUI.Button(rect, GUIContent.none, GUIStyle.none);
        if (swatch >= 0)
        {
            rect.xMin += 10f;
        }
        GUI.Label(rect, text, selected ? selectedButton : button);
        return pressed;
    }

    private void MakeStyles()
    {
        label = new GUIStyle(GUI.skin.label)
        {
            alignment = TextAnchor.MiddleLeft,
            fontSize = 13,
            padding = new RectOffset()
        };
        label.normal.textColor = Muted;
        button = new GUIStyle(label) { alignment = TextAnchor.MiddleCenter, fontSize = 12 };
        button.normal.textColor = Ink;
        selectedButton = new GUIStyle(button) { fontStyle = FontStyle.Bold };
        selectedButton.normal.textColor = Brass;
    }

    private static void Box(Rect rect, Color color)
    {
        Color previous = GUI.color;
        GUI.color = color;
        GUI.DrawTexture(rect, Texture2D.whiteTexture);
        GUI.color = previous;
    }

    private void OnDestroy()
    {
        if (runtimeSky == null)
            return;
        if (RenderSettings.skybox == runtimeSky)
            RenderSettings.skybox = sceneSky;
        Destroy(runtimeSky);
    }
}
