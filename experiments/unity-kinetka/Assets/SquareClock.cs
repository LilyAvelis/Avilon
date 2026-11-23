using UnityEngine;
using UnityEngine.Rendering;

public sealed class SquareClock : MonoBehaviour
{
    public Light sun;
    public Light moon;
    public Light[] lanterns;
    public Renderer[] windows;
    public Material sky;
    public Transform clockHand;
    public Camera view;

    private static readonly int Emission = Shader.PropertyToID("_EmissionColor");
    private static readonly int Top = Shader.PropertyToID("_Top");
    private static readonly int Bottom = Shader.PropertyToID("_Bottom");
    private MaterialPropertyBlock glass;
    private KinetkaHud hud;
    private Texture2D dial;
    private bool turning;
    private float pointerAngle;
    private float hour = 11.5f;
    private float referenceSize;

    private static readonly Color Paper = new(0.99f, 0.94f, 0.80f);
    private static readonly Color Brass = new(0.94f, 0.72f, 0.42f);
    private static readonly Color DialFace = new(0.09f, 0.12f, 0.19f);
    private static readonly Color BrightGlass = new(1f, 0.73f, 0.31f);
    private static readonly Color DarkGlass = new(0.12f, 0.20f, 0.24f);

    private void Start()
    {
        Application.targetFrameRate = 60;
        referenceSize = view.orthographicSize;
        SetHour(hour);
    }

    private void Update()
    {
        KinetkaHud.FitCamera(view, referenceSize);
        float direction = (Input.GetKey(KeyCode.RightArrow) ? 1 : 0)
            - (Input.GetKey(KeyCode.LeftArrow) ? 1 : 0);
        if (direction != 0)
            SetHour(hour + direction * 7f * Time.unscaledDeltaTime);
    }

    public void SetHour(float value)
    {
        glass ??= new MaterialPropertyBlock();
        hour = Mathf.Repeat(value, 24f);
        float phase = (hour - 6f) / 24f * 2f * Mathf.PI;
        float elevation = Mathf.Sin(phase);
        float day = Mathf.SmoothStep(0f, 1f, Mathf.InverseLerp(-0.19f, 0.38f, elevation));
        float dusk = Mathf.Exp(-Mathf.Abs(elevation) * 9f);
        float windowLight = 1f - Mathf.SmoothStep(0f, 1f, Mathf.InverseLerp(-0.34f, 0.15f, elevation));

        sun.transform.rotation = Quaternion.Euler(elevation * 61f, 16f + 79f * Mathf.Cos(phase), 0f);
        sun.color = Color.Lerp(new Color(1f, 0.49f, 0.23f), new Color(1f, 0.93f, 0.78f), day);
        sun.intensity = Mathf.Lerp(0f, 0.89f, Mathf.SmoothStep(0f, 1f, Mathf.InverseLerp(-0.04f, 0.49f, elevation)));
        moon.intensity = Mathf.Lerp(0.43f, 0.10f, day);

        Color upper = Color.Lerp(new Color(0.065f, 0.075f, 0.18f), new Color(0.26f, 0.39f, 0.53f), day);
        Color lower = Color.Lerp(new Color(0.11f, 0.12f, 0.25f), new Color(0.40f, 0.53f, 0.62f), day);
        sky.SetColor(Top, Color.Lerp(upper, new Color(0.43f, 0.22f, 0.34f), dusk * 0.35f));
        sky.SetColor(Bottom, Color.Lerp(lower, new Color(0.74f, 0.36f, 0.32f), dusk * 0.63f));
        RenderSettings.ambientMode = AmbientMode.Trilight;
        RenderSettings.ambientSkyColor = Color.Lerp(new Color(0.185f, 0.20f, 0.36f), new Color(0.58f, 0.61f, 0.69f), day);
        RenderSettings.ambientEquatorColor = RenderSettings.ambientSkyColor * 0.82f;
        RenderSettings.ambientGroundColor = RenderSettings.ambientSkyColor * 0.53f;

        glass.SetColor(Emission, Color.Lerp(DarkGlass * 0.12f, BrightGlass * 1.22f, windowLight));
        foreach (Renderer window in windows)
            window.SetPropertyBlock(glass);
        foreach (Light lantern in lanterns)
            lantern.intensity = windowLight * 2.6f;
        clockHand.localRotation = Quaternion.Euler(0f, hour * 15f, 0f);
    }

    private void OnGUI()
    {
        if (hud == null)
        {
            hud = new KinetkaHud(Paper, "Georgia");
            dial = DrawDial();
        }
        Matrix4x4 before = GUI.matrix;
        GUI.matrix = KinetkaHud.Matrix;
        float width = Screen.width / KinetkaHud.Scale;
        float height = Screen.height / KinetkaHud.Scale;
        Vector2 center = new(width - 141, height - 152);
        ReadDial(center);
        hud.Header("Площадь от рассвета до ночи", "Поверни диск и посмотри, как меняется свет.");
        GUI.DrawTexture(new Rect(center.x - 82, center.y - 82, 164, 164), dial);
        Matrix4x4 dialMatrix = GUI.matrix;
        GUIUtility.RotateAroundPivot(hour * 15f, center);
        KinetkaHud.Box(new Rect(center.x - 1.5f, center.y - 69, 3, 19), Brass);
        GUI.matrix = dialMatrix;
        int minutes = Mathf.FloorToInt(hour * 60f);
        hud.Label(new Rect(center.x - 27, center.y - 18, 57, 36), $"{minutes / 60:00}:{minutes % 60:00}");
        hud.Small(new Rect(center.x - 56, center.y + 89, 145, 29), "Мышь  ·  ← →");
        GUI.matrix = before;
    }

    private void ReadDial(Vector2 center)
    {
        Event input = Event.current;
        Vector2 offset = input.mousePosition - center;
        float angle = Mathf.Atan2(offset.x, -offset.y) * Mathf.Rad2Deg;
        if (input.type == EventType.MouseDown && input.button == 0 && offset.magnitude is >= 21f and <= 91f)
        {
            turning = true;
            pointerAngle = angle;
            SetHour(angle / 15f);
            input.Use();
        }
        else if (input.type == EventType.MouseDrag && turning)
        {
            if (offset.sqrMagnitude >= 100f)
            {
                SetHour(hour + Mathf.DeltaAngle(pointerAngle, angle) / 15f);
                pointerAngle = angle;
            }
            input.Use();
        }
        else if (input.rawType == EventType.MouseUp)
        {
            turning = false;
        }
    }

    private static Texture2D DrawDial()
    {
        const int size = 328;
        var pixels = new Color32[size * size];
        for (int y = 0; y < size; y++)
        for (int x = 0; x < size; x++)
        {
            Vector2 point = (new Vector2(x + 0.5f, y + 0.5f) / size - Vector2.one * 0.5f) * 2f;
            float radius = point.magnitude;
            float units = Mathf.Atan2(point.x, point.y) * (24f / (2f * Mathf.PI));
            int tick = Mathf.RoundToInt(units);
            bool major = tick % 6 == 0;
            float ring = 1f - Mathf.Clamp01(Mathf.Abs(radius - 0.95f) * 95f);
            float marks = radius > (major ? 0.70f : 0.78f) && radius < 0.88f
                ? 1f - Mathf.SmoothStep(0f, 1f, Mathf.Abs(units - tick) * (major ? 13f : 20f))
                : 0f;
            Color pixel = Color.Lerp(DialFace, Brass, Mathf.Max(ring, marks * (major ? 0.9f : 0.55f)));
            pixel.a = Mathf.Clamp01((0.99f - radius) * 170f);
            pixels[y * size + x] = pixel;
        }
        var result = new Texture2D(size, size, TextureFormat.RGBA32, false);
        result.SetPixels32(pixels);
        result.Apply(false, true);
        return result;
    }

    private void OnDestroy()
    {
        if (dial != null)
            Destroy(dial);
    }
}
