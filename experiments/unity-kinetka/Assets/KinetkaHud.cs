using UnityEngine;

internal sealed class KinetkaHud
{
    private readonly GUIStyle heading;
    private readonly GUIStyle label;
    private readonly GUIStyle small;
    private readonly GUIStyle button;
    private readonly Color ink;

    public KinetkaHud(Color ink, string displayFont)
    {
        this.ink = ink;
        heading = Style(50, Font.CreateDynamicFontFromOSFont(displayFont, 50));
        label = Style(20);
        small = Style(15);
        button = new GUIStyle(Style(17)) { alignment = TextAnchor.MiddleCenter };
        button.hover.textColor = ink;
        button.active.textColor = ink;
    }

    public static float Scale => Mathf.Min(Screen.width / 1280f, Screen.height / 820f);
    public static Matrix4x4 Matrix => Matrix4x4.Scale(new Vector3(Scale, Scale, 1f));

    public static void FitCamera(Camera camera, float referenceSize)
    {
        camera.orthographicSize = referenceSize * Mathf.Max(1f, (1280f / 820f) / camera.aspect);
    }

    private GUIStyle Style(int size, Font font = null)
    {
        var style = new GUIStyle(GUI.skin.label) { fontSize = size, font = font };
        style.normal.textColor = ink;
        return style;
    }

    public void Header(string title, string instruction)
    {
        GUI.Label(new Rect(44, 27, 1050, 76), title, heading);
        GUI.Label(new Rect(49, 101, 900, 40), instruction, label);
    }

    public void Label(Rect rect, string text) => GUI.Label(rect, text, label);

    public void Small(Rect rect, string text) => GUI.Label(rect, text, small);

    public bool Button(Rect rect, string text, Color fill)
    {
        Rect edge = new(rect.x - 1, rect.y - 1, rect.width + 2, rect.height + 2);
        Box(edge, new Color(ink.r, ink.g, ink.b, 0.48f));
        Box(rect, rect.Contains(Event.current.mousePosition) ? Color.Lerp(fill, ink, 0.20f) : fill);
        return GUI.Button(rect, text, button);
    }

    public static void Box(Rect rect, Color color)
    {
        Color previous = GUI.color;
        GUI.color = color;
        GUI.DrawTexture(rect, Texture2D.whiteTexture);
        GUI.color = previous;
    }
}
