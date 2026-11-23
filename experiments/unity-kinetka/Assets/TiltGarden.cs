using System;
using UnityEngine;

public sealed class TiltGarden : MonoBehaviour
{
    [Serializable]
    public sealed class Level
    {
        public Rigidbody tray;
        public Transform start;
        public Transform goal;
        public Renderer goalRing;
        public Renderer deck;
    }

    public Level[] levels;
    public Rigidbody marble;
    public TiltGardenCamera orbit;
    public TiltGardenAppearance appearance;
    public TiltGardenWater water;
    public AudioSource sound;

    private const float MarbleRadius = 0.246f;
    private int levelIndex;
    private Level Current => levels[levelIndex];

    private const float MaxTilt = 12.5f;
    private const float MaxTiltSpeed = 28f;
    private static readonly int Emission = Shader.PropertyToID("_EmissionColor");
    private MaterialPropertyBlock glow;
    private Vector2 steering;
    private GUIStyle title;
    private GUIStyle body;
    private GUIStyle hint;
    private AudioClip chime;
    private float goalTime;
    private float successTime;
    private bool completed;
    private bool falling;

    private void Start()
    {
        glow = new MaterialPropertyBlock();
        Application.targetFrameRate = 90;
        sound.volume = 0.42f;
        chime = MakeChime();
        water.Initialize(-2.65f, 9.5f);
        SelectLevel(0);
    }

    private void SelectLevel(int index)
    {
        levelIndex = index;
        for (int i = 0; i < levels.Length; i++)
            levels[i].tray.gameObject.SetActive(i == index);

        appearance.BindDeck(Current.deck);
        Restart();
        orbit.Frame(Current.tray.position, Current.deck.transform.localScale);
    }

    private void Update()
    {
        if (Input.GetKeyDown(KeyCode.R))
            Restart();

        steering = Vector2.ClampMagnitude(new Vector2(
            (Input.GetKey(KeyCode.RightArrow) ? 1 : 0) - (Input.GetKey(KeyCode.LeftArrow) ? 1 : 0),
            (Input.GetKey(KeyCode.UpArrow) ? 1 : 0) - (Input.GetKey(KeyCode.DownArrow) ? 1 : 0)), 1f);
    }

    private void FixedUpdate()
    {
        Level level = Current;
        Rigidbody tray = level.tray;
        Vector3 cameraRight = Vector3.ProjectOnPlane(orbit.transform.right, Vector3.up).normalized;
        Vector3 cameraUp = Vector3.Cross(cameraRight, Vector3.up);
        Vector3 down = cameraRight * steering.x + cameraUp * steering.y;
        float slope = Mathf.Tan(MaxTilt * Mathf.Deg2Rad);
        Quaternion wanted = completed
            ? Quaternion.identity
            : Quaternion.FromToRotation(Vector3.up, Vector3.up + down * slope);
        float angle = Quaternion.Angle(tray.rotation, wanted);
        float easedStep = angle * (1f - Mathf.Exp(-9f * Time.fixedDeltaTime));
        float limitedStep = Mathf.Min(easedStep, MaxTiltSpeed * Time.fixedDeltaTime);
        Quaternion turn = Quaternion.RotateTowards(tray.rotation, wanted, limitedStep);
        tray.MoveRotation(turn);

        if (completed)
        {
            Vector3 target = tray.position + turn * level.goal.localPosition;
            marble.MovePosition(Vector3.Lerp(marble.position, target, 1f - Mathf.Exp(-12f * Time.fixedDeltaTime)));
            successTime += Time.fixedDeltaTime;
            SetGlow(1.6f + 2.8f * Mathf.Exp(-successTime * 2.8f));
            return;
        }

        Vector3 local = Quaternion.Inverse(tray.rotation) * (marble.position - tray.position);
        falling = local.y < -0.45f;
        if (water.Step(marble, MarbleRadius) || marble.position.y < -7f)
        {
            Restart();
            return;
        }

        Vector3 goalPosition = level.goal.localPosition;
        Vector2 offset = new(local.x - goalPosition.x, local.z - goalPosition.z);
        bool inside = offset.sqrMagnitude < 0.52f * 0.52f && Mathf.Abs(local.y - goalPosition.y) < 0.28f;
        goalTime = inside ? goalTime + Time.fixedDeltaTime : 0f;
        SetGlow(0.20f + (inside ? 1.3f : 0.10f * Mathf.Sin(Time.time * 2.3f)));
        if (goalTime < 0.12f)
            return;

        completed = true;
        marble.linearVelocity = Vector3.zero;
        marble.angularVelocity = Vector3.zero;
        marble.isKinematic = true;
        sound.PlayOneShot(chime);
    }

    private void Restart()
    {
        completed = false;
        falling = false;
        water.ResetWater();
        goalTime = 0f;
        successTime = 0f;
        Current.tray.rotation = Quaternion.identity;
        marble.isKinematic = false;
        marble.position = Current.start.position;
        marble.rotation = Quaternion.identity;
        marble.linearVelocity = Vector3.zero;
        marble.angularVelocity = Vector3.zero;
        marble.WakeUp();
        SetGlow(0.2f);
    }

    private void SetGlow(float value)
    {
        glow.SetColor(Emission, new Color(0.21f, 0.91f, 0.73f) * value);
        Current.goalRing.SetPropertyBlock(glow);
    }

    private void OnGUI()
    {
        if (title == null)
        {
            Font serif = Font.CreateDynamicFontFromOSFont("Georgia", 54);
            title = new GUIStyle(GUI.skin.label) { font = serif, fontSize = 54 };
            title.normal.textColor = new Color(0.91f, 0.95f, 0.86f);
            body = new GUIStyle(GUI.skin.label) { fontSize = 19, wordWrap = true, richText = false };
            body.normal.textColor = new Color(0.86f, 0.92f, 0.87f);
            hint = new GUIStyle(GUI.skin.label) { fontSize = 15 };
            hint.normal.textColor = new Color(0.83f, 0.90f, 0.85f);
        }

        float scale = Mathf.Min(Screen.width / 1280f, Screen.height / 820f);
        Matrix4x4 previous = GUI.matrix;
        GUI.matrix = Matrix4x4.Scale(new Vector3(scale, scale, 1f));
        float width = Screen.width / scale;
        GUI.Label(new Rect(46, 32, 620, 82), "Наклонный сад", title);

        float tabsWidth = 111f * levels.Length;
        float tabX = width - tabsWidth - 46f;
        for (int i = 0; i < levels.Length; i++)
        {
            bool selected = i == levelIndex;
            bool pressed = GUI.Toggle(new Rect(tabX + i * 111f, 54, 103, 34),
                selected, $"Карта {i + 1}", GUI.skin.button);
            if (pressed && !selected)
                SelectLevel(i);
        }

        appearance.Draw(new Rect(width - 504f, 105f, 458f, 46f));
        string text = completed
            ? "Шарик в лунке. Сад успокоился."
            : falling
                ? "Шарик упал. Сейчас он вернётся на старт."
                : "Наклони поднос и докати шарик до светящейся лунки.";
        GUI.Label(new Rect(53, 116, 640, 62), text, body);
        if (GUI.Button(new Rect(width - 154f, Screen.height / scale - 62f, 108, 34), "Заново"))
            Restart();
        GUI.Label(new Rect(53, Screen.height / scale - 53f, 1000, 35),
            "Стрелки — наклон   ·   ПКМ + мышь — поворот   ·   A/D — вбок   ·   V/S — ближе / дальше   ·   R — заново", hint);
        GUI.matrix = previous;
    }

    private static AudioClip MakeChime()
    {
        const int sampleRate = 44100;
        const float duration = 0.85f;
        float[] samples = new float[(int)(sampleRate * duration)];
        float[] frequencies = { 523.25f, 659.25f, 783.99f };
        for (int i = 0; i < samples.Length; i++)
        {
            for (int note = 0; note < frequencies.Length; note++)
            {
                float t = i / (float)sampleRate - note * 0.065f;
                if (t < 0f)
                    continue;
                float envelope = (1f - Mathf.Exp(-110f * t)) * Mathf.Exp(-8f * t);
                samples[i] += 0.16f * envelope * Mathf.Sin(t * frequencies[note] * Mathf.PI * 2f);
            }
        }
        AudioClip clip = AudioClip.Create("Garden chime", samples.Length, 1, sampleRate, false);
        clip.SetData(samples, 0);
        return clip;
    }
}
