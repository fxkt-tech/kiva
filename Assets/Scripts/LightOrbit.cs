using UnityEngine;

// Place a light off the pivot's local Y axis to give it a horizontal orbit.
[DisallowMultipleComponent]
public sealed class LightOrbit : MonoBehaviour
{
    [SerializeField, Min(.01f)] float periodSeconds = 60f;
    Quaternion initialRotation;
    double startedAt;

    void OnEnable()
    {
        initialRotation = transform.localRotation;
        startedAt = Time.timeAsDouble;
    }

    void Update()
    {
        double period = Mathf.Max(.01f, periodSeconds);
        float angle = (float)(((Time.timeAsDouble - startedAt) % period) * 360.0 / period);
        transform.localRotation = initialRotation * Quaternion.AngleAxis(angle, Vector3.up);
    }

    void OnDisable() => transform.localRotation = initialRotation;
}
