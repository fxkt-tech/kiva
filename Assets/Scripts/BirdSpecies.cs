using UnityEngine;

[CreateAssetMenu(menuName="百鸟卷/鸟种")]
public sealed class BirdSpecies : ScriptableObject
{
    public string id, birdName;
    [TextArea] public string description;
    [TextArea] public string naturalHistory, albumStory;
    public Color[] pigments;
    public Mesh body, leftWing, rightWing, tail;
    public AudioClip call;
    public bool forkedTail, bobsTail;
    public static readonly Vector3 WingRoot = new Vector3(.065f,.23f,-.015f);
    public static readonly Vector3 TailRoot = new Vector3(0,.16f,-.11f);

    public Mesh Part(int part)
    {
        switch (part) { case 1: return leftWing; case 2: return rightWing; case 3: return tail; default: return body; }
    }

    public Vector3 Pivot(int part)
    {
        if (part == 1) return Vector3.Scale(WingRoot,new Vector3(-1,1,1));
        if (part == 2) return WingRoot;
        return part == 3 ? TailRoot : Vector3.zero;
    }

    public Quaternion Pose(int part,float phase,float flight)
    {
        if (part == 3) return Quaternion.Euler((bobsTail ? 13f : 4f)*Mathf.Sin(phase*2),0,0);
        if (part == 0) return Quaternion.identity;
        float open = 2f*Mathf.Sin(phase) + flight*(40f+32f*Mathf.Sin(phase*6));
        return Quaternion.Euler(0,0,part==1 ? -open : open);
    }
}
