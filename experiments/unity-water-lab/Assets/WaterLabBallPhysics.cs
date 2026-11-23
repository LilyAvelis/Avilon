using UnityEngine;

public static class WaterLabBallPhysics
{
    private const float ReferenceRadius = 0.32f;

    public static float SubmergedFraction(float centerY, float surfaceY, float radius)
    {
        if (radius <= 0f)
            return 0f;

        // The cap height runs from zero at first contact to the sphere's diameter.
        // Dividing its volume by the full sphere volume gives f²(3 − 2f).
        float depth = Mathf.Clamp01((surfaceY - centerY + radius) / (2f * radius));
        return depth * depth * (3f - 2f * depth);
    }

    public static float ImpactStrength(float downwardSpeed, float radius)
    {
        // Scale the impulse from the old reference ball without letting the
        // largest ball's full volume ratio overwhelm the height field.
        float referenceStrength = Mathf.Clamp(downwardSpeed * 0.62f, 0.9f, 5f);
        return referenceStrength * Mathf.Pow(Mathf.Max(0f, radius / ReferenceRadius), 1.5f);
    }

    public static Vector3 Acceleration(
        Vector3 velocity, Vector3 gravity, float submergedFraction, float densityRatio, float drag)
    {
        return (-gravity * densityRatio - velocity * drag) * submergedFraction;
    }
}
