using System.Collections.Generic;
using UnityEngine;

public sealed partial class WaterLaboratory
{
    private static readonly int[] SeabedDepths = { 5, 10, 20 };
    private static readonly int ProceduralBed = Shader.PropertyToID("_ProceduralBed");
    private static readonly int SeabedProfileProperty = Shader.PropertyToID("_SeabedProfile");
    private static readonly int PoincareOverview = Shader.PropertyToID("_PoincareOverview");

    public GameObject[] referenceBedObjects;
    private readonly List<GameObject> referenceBottom = new();
    private WaterLabSeabed seabed;
    private Material seabedMaterial;
    private bool proceduralSeabed;
    private int seabedDepthIndex;
    private int seabedProfileIndex = 1;
    private int spectreScaleIndex;
    private float seabedNoise = 0.65f;
    private bool poincareOverview;
    private float savedFarClip;
    private bool appliedSeabed;
    private bool appliedPoincareOverview;
    private int appliedSeabedDepth = -1;
    private int appliedSeabedProfile = -1;
    private int appliedSpectreScale = -1;
    private float appliedSeabedNoise = -1f;

    private void InitializeSeabed()
    {
        foreach (GameObject piece in referenceBedObjects ?? System.Array.Empty<GameObject>())
        {
            if (piece != null && piece.activeSelf)
                referenceBottom.Add(piece);
        }
        seabedMaterial = new Material(Resources.Load<Shader>("Mira/WaterLabSeabed"));
    }

    private void UpdateSeabed()
    {
        if (poincareOverview && (!proceduralSeabed || seabedProfileIndex != (int)SeabedProfile.Poincare))
            TogglePoincareOverview();
        if (appliedSeabed == proceduralSeabed && appliedSeabedDepth == seabedDepthIndex &&
            appliedSeabedProfile == seabedProfileIndex && appliedSeabedNoise == seabedNoise &&
            appliedSpectreScale == spectreScaleIndex && appliedPoincareOverview == poincareOverview)
            return;

        if (proceduralSeabed && seabed == null)
            seabed = new WaterLabSeabed(transform, seabedMaterial);

        if (appliedSeabed != proceduralSeabed)
        {
            foreach (GameObject reference in referenceBottom)
                if (reference != null)
                    reference.SetActive(!proceduralSeabed);
        }

        seabedMaterial.SetFloat(SeabedProfileProperty, seabedProfileIndex);
        seabed?.SetView(proceduralSeabed, SeabedDepths[seabedDepthIndex], (SeabedProfile)seabedProfileIndex,
            seabedNoise, spectreScaleIndex == 1, poincareOverview);
        material.SetFloat(ProceduralBed, proceduralSeabed ? 1f : 0f);
        appliedSeabed = proceduralSeabed;
        appliedSeabedDepth = seabedDepthIndex;
        appliedSeabedProfile = seabedProfileIndex;
        appliedSpectreScale = spectreScaleIndex;
        appliedPoincareOverview = poincareOverview;
        appliedSeabedNoise = seabedNoise;
    }

    private void TogglePoincareOverview()
    {
        poincareOverview = !poincareOverview;
        if (poincareOverview)
            savedFarClip = view.farClipPlane;
        else
            view.farClipPlane = savedFarClip;
        material.SetFloat(PoincareOverview, poincareOverview ? 1f : 0f);
        if (lightingSkybox != null)
            lightingSkybox.SetFloat(PoincareOverview, poincareOverview ? 1f : 0f);
        PositionCamera();
    }

    private void DisposeSeabed()
    {
        if (appliedSeabed)
            foreach (GameObject reference in referenceBottom)
                if (reference != null)
                    reference.SetActive(true);

        seabed?.Dispose();
        if (seabedMaterial != null)
            Destroy(seabedMaterial);
    }
}
