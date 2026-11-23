using UnityEngine;

public sealed class DominoRun : MonoBehaviour
{
    public Rigidbody[] tiles;
    public Rigidbody bell;
    public Renderer bellSignal;
    public Camera view;
    public AudioSource sound;

    private static readonly int Emission = Shader.PropertyToID("_EmissionColor");
    private static readonly Color Cream = new(1f, 0.93f, 0.79f);
    private static readonly Color Ink = new(0.19f, 0.10f, 0.24f);
    private MaterialPropertyBlock glow;
    private Vector3[] starts;
    private Quaternion[] rotations;
    private Quaternion bellRest;
    private bool running;
    private int fallen;
    private float ringAge = -1;
    private AudioClip chime;
    private KinetkaHud hud;
    private float referenceSize;

    public int Fallen => fallen;
    public bool Finished => ringAge >= 0f;

    private void Start()
    {
        Application.targetFrameRate = 60;
        referenceSize = view.orthographicSize;
        glow = new MaterialPropertyBlock();
        RecordStartingPoses();
        chime = MakeChime();
    }

    private void RecordStartingPoses()
    {
        starts = new Vector3[tiles.Length];
        rotations = new Quaternion[tiles.Length];
        for (int i = 0; i < tiles.Length; i++)
        {
            starts[i] = tiles[i].position;
            rotations[i] = tiles[i].rotation;
        }
        bellRest = bell.rotation;
    }

    private void Update()
    {
        KinetkaHud.FitCamera(view, referenceSize);
        if (Input.GetKeyDown(KeyCode.R))
            ResetRun();
        if (Input.GetKeyDown(KeyCode.Space))
            StartRun();
        if (!running && Input.GetMouseButtonDown(0)
            && Physics.Raycast(view.ScreenPointToRay(Input.mousePosition), out RaycastHit hit, 100f)
            && hit.rigidbody == tiles[0])
            StartRun();
    }

    private void FixedUpdate()
    {
        if (!running)
            return;
        int tipped = 0;
        foreach (Rigidbody tile in tiles)
            if (Vector3.Dot(tile.rotation * Vector3.up, Vector3.up) < 0.81f)
                tipped++;
        fallen = Mathf.Max(fallen, tipped);

        if (!Finished)
            return;
        ringAge += Time.fixedDeltaTime;
        float swing = 13f * Mathf.Sin(ringAge * 15f) * Mathf.Exp(-ringAge * 2.15f);
        bell.MoveRotation(bellRest * Quaternion.Euler(-swing, 0f, 0f));
        glow.SetColor(Emission, new Color(1f, 0.44f, 0.09f) * (0.55f + 1.3f * Mathf.Exp(-ringAge * 3f)));
        bellSignal.SetPropertyBlock(glow);
    }

    private void OnCollisionEnter(Collision collision)
    {
        if (!running || Finished || collision.rigidbody != tiles[^1])
            return;
        ringAge = 0;
        sound.PlayOneShot(chime);
    }

    public void StartRun()
    {
        if (running)
            return;
        running = true;
        var first = tiles[0];
        first.angularVelocity = first.rotation * Vector3.right * 3.9f;
        first.linearVelocity = first.rotation * Vector3.forward * 0.43f;
        first.WakeUp();
    }

    public void ResetRun()
    {
        running = false;
        fallen = 0;
        ringAge = -1;
        for (int i = 0; i < tiles.Length; i++)
        {
            Rigidbody tile = tiles[i];
            tile.linearVelocity = Vector3.zero;
            tile.angularVelocity = Vector3.zero;
            tile.position = starts[i];
            tile.rotation = rotations[i];
            tile.transform.SetPositionAndRotation(starts[i], rotations[i]);
        }
        bell.rotation = bellRest;
        bell.transform.rotation = bellRest;
        Physics.SyncTransforms();
        foreach (Rigidbody tile in tiles)
            tile.Sleep();
        bellSignal.SetPropertyBlock(null);
        sound.Stop();
    }

    private void OnGUI()
    {
        hud ??= new KinetkaHud(Cream, "Georgia");
        Matrix4x4 before = GUI.matrix;
        GUI.matrix = KinetkaHud.Matrix;
        float width = Screen.width / KinetkaHud.Scale;
        float height = Screen.height / KinetkaHud.Scale;
        hud.Header("Дорожка домино", "Толкни первую плашку, чтобы зазвенел колокольчик.");
        hud.Label(new Rect(48, height - 91, 480, 35),
            Finished ? "Колокольчик прозвенел" : $"Плашки  {fallen} / {tiles.Length}");
        hud.Small(new Rect(48, height - 51, 480, 28), "ЛКМ по первой плашке  ·  Space — толчок  ·  R — заново");
        if (hud.Button(new Rect(width - 183, height - 88, 134, 42), "Заново", Ink))
            ResetRun();
        GUI.matrix = before;
    }

    private static AudioClip MakeChime()
    {
        const int rate = 44100;
        float[] samples = new float[rate * 3 / 2];
        for (int i = 0; i < samples.Length; i++)
        {
            float time = i / (float)rate;
            float attack = 1f - Mathf.Exp(-900f * time);
            samples[i] = attack * (0.34f * Mathf.Sin(time * 2f * Mathf.PI * 920f) * Mathf.Exp(-3.3f * time)
                + 0.16f * Mathf.Sin(time * 2f * Mathf.PI * 1856f) * Mathf.Exp(-7f * time)
                + 0.06f * Mathf.Sin(time * 2f * Mathf.PI * 2537f) * Mathf.Exp(-14f * time));
        }
        AudioClip clip = AudioClip.Create("Brass bell", samples.Length, 1, rate, false);
        clip.SetData(samples, 0);
        return clip;
    }

    private void OnDestroy()
    {
        if (chime != null)
            Destroy(chime);
    }
}
