using System;
using UnityEditor;
using UnityEditor.SceneManagement;
using UnityEngine;

// Run with -batchmode -executeMethod DominoPlayCheck.Run, without -quit.
internal static class DominoPlayCheck
{
    private const string Pending = "Kinetka.DominoPlayCheck";
    private static DominoRun run;
    private static Vector3[] startPositions;
    private static Quaternion[] startRotations;
    private static Quaternion bellStart;
    private static int stage;
    private static float began;
    private static double deadline;

    public static void Run()
    {
        EditorSceneManager.OpenScene("Assets/Domino Run.unity");
        SessionState.SetBool(Pending, true);
        Register();
        EditorApplication.EnterPlaymode();
    }

    [InitializeOnLoadMethod]
    private static void Register()
    {
        if (!SessionState.GetBool(Pending, false))
            return;
        deadline = EditorApplication.timeSinceStartup + 55;
        EditorApplication.update -= Tick;
        EditorApplication.update += Tick;
    }

    private static void Tick()
    {
        try
        {
            if (EditorApplication.timeSinceStartup > deadline)
                throw new InvalidOperationException("Timed out waiting for the domino scene.");
            if (EditorApplication.isPlaying && Time.time > 0.20f)
                CheckStage();
        }
        catch (Exception error)
        {
            Debug.LogException(error);
            Exit(1);
        }
    }

    private static void CheckStage()
    {
        if (run == null)
        {
            run = UnityEngine.Object.FindFirstObjectByType<DominoRun>();
            startPositions = Array.ConvertAll(run.tiles, tile => tile.transform.position);
            startRotations = Array.ConvertAll(run.tiles, tile => tile.transform.rotation);
            bellStart = run.bell.rotation;
            run.StartRun();
            began = Time.time;
            return;
        }

        switch (stage)
        {
            case 0 when run.Fallen >= 5:
                Debug.Log($"Domino Play Mode: resetting mid-run at {run.Fallen} tiles.");
                ResetAndCheck();
                stage++;
                break;
            case 1 when Time.time - began > 0.34f:
            case 3 when Time.time - began > 0.34f:
                CheckRest();
                run.StartRun();
                began = Time.time;
                stage++;
                break;
            case 2 when run.Finished && Time.time - began > 8.8f:
                CheckBell();
                ResetAndCheck();
                stage++;
                break;
            case 4 when run.Finished && Time.time - began > 8.8f:
                CheckBell();
                ResetAndCheck();
                Debug.Log("Domino Play Mode: two solid bell hits and three immediate visual resets passed.");
                Exit(0);
                break;
        }
    }

    private static void CheckBell()
    {
        if (run.Fallen < run.tiles.Length - 1)
            throw new InvalidOperationException($"The bell rang before the chain completed: {run.Fallen}.");
        Debug.Log($"Domino Play Mode: collision with bell after {Time.time - began:F2}s ({run.Fallen}/{run.tiles.Length} tilted).");
    }

    private static void ResetAndCheck()
    {
        run.ResetRun();
        CheckRest();
        began = Time.time;
    }

    private static void CheckRest()
    {
        if (run.Fallen != 0 || run.Finished || Quaternion.Angle(run.bell.transform.rotation, bellStart) > 0.25f)
            throw new InvalidOperationException("The bell or run status survived a reset.");
        for (int i = 0; i < run.tiles.Length; i++)
        {
            Rigidbody tile = run.tiles[i];
            if (Vector3.Distance(tile.transform.position, startPositions[i]) > 0.015f
                || Quaternion.Angle(tile.transform.rotation, startRotations[i]) > 0.25f
                || Vector3.Distance(tile.position, tile.transform.position) > 0.015f
                || Quaternion.Angle(tile.rotation, tile.transform.rotation) > 0.25f)
                throw new InvalidOperationException($"Visible tile {i + 1} did not return to its initial pose.");
        }
    }

    private static void Exit(int code)
    {
        SessionState.EraseBool(Pending);
        EditorApplication.update -= Tick;
        EditorApplication.Exit(code);
    }
}
