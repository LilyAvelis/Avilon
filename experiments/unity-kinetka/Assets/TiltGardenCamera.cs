using UnityEngine;

public sealed class TiltGardenCamera : MonoBehaviour
{
    private const float FieldOfView = 43f;
    private const float DefaultYaw = -35f;
    private const float DefaultPitch = 43f;
    private float yaw;
    private float pitch;
    private float distance;
    private float span;
    private float aspect;
    private float framingDistance;
    private Vector2 boardHalfSize;
    private Camera view;
    private Vector3 center;
    private Vector3 offset;

    public void Frame(Vector3 target, Vector3 boardSize)
    {
        center = target + Vector3.up * 0.15f;
        span = Mathf.Max(boardSize.x, boardSize.z);
        boardHalfSize = new Vector2(boardSize.x * 0.5f + 0.16f, boardSize.z * 0.5f + 0.16f);
        offset = Vector3.zero;
        yaw = DefaultYaw;
        pitch = DefaultPitch;
        view = GetComponent<Camera>();
        view.orthographic = false;
        view.fieldOfView = FieldOfView;
        aspect = view.aspect;
        framingDistance = FramingDistance(aspect);
        distance = framingDistance;
        Place();
    }

    private float MaxDistance => Mathf.Max(span * 3.1f, framingDistance);

    private float FramingDistance(float widthToHeight)
    {
        Quaternion inverse = Quaternion.Inverse(Quaternion.Euler(DefaultPitch, DefaultYaw, 0f));
        float halfHeight = Mathf.Tan(FieldOfView * 0.5f * Mathf.Deg2Rad) * 0.82f;
        float halfWidth = halfHeight * Mathf.Max(0.28f, widthToHeight);
        float fit = 0f;
        for (int x = -1; x <= 1; x += 2)
        for (int z = -1; z <= 1; z += 2)
        {
            Vector3 corner = inverse * new Vector3(boardHalfSize.x * x, -0.15f, boardHalfSize.y * z);
            float minimumDepth = Mathf.Max(Mathf.Abs(corner.x) / halfWidth, Mathf.Abs(corner.y) / halfHeight);
            fit = Mathf.Max(fit, minimumDepth - corner.z);
        }
        return fit;
    }

    private void Update()
    {
        if (span <= 0f)
            return;

        // Holding RMB leaves the pointer available for the level and appearance controls.
        if (Input.GetMouseButton(1))
        {
            yaw = Mathf.Repeat(yaw + Input.GetAxisRaw("Mouse X") * 4f, 360f);
            pitch = Mathf.Clamp(pitch - Input.GetAxisRaw("Mouse Y") * 3f, 23f, 79f);
        }

        if (Mathf.Abs(aspect - view.aspect) > 0.001f)
        {
            aspect = view.aspect;
            float nextFraming = FramingDistance(aspect);
            distance *= nextFraming / framingDistance;
            framingDistance = nextFraming;
        }

        float zoom = (Input.GetKey(KeyCode.S) ? 1f : 0f) - (Input.GetKey(KeyCode.V) ? 1f : 0f);
        distance = Mathf.Clamp(distance + zoom * span * Time.deltaTime, span * 0.53f, MaxDistance);

        float strafe = (Input.GetKey(KeyCode.D) ? 1f : 0f) - (Input.GetKey(KeyCode.A) ? 1f : 0f);
        Vector3 right = Quaternion.Euler(0f, yaw, 0f) * Vector3.right;
        offset = Vector3.ClampMagnitude(offset + right * (strafe * span * 0.6f * Time.deltaTime), span * 0.42f);
        Place();
    }

    private void Place()
    {
        Quaternion rotation = Quaternion.Euler(pitch, yaw, 0f);
        Vector3 position = center + offset - rotation * Vector3.forward * distance;
        transform.SetPositionAndRotation(position, rotation);
    }
}
