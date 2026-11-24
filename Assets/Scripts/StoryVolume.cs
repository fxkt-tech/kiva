using UnityEngine;

[CreateAssetMenu(menuName="百鸟卷/剧情画卷")]
public sealed class StoryVolume : ScriptableObject
{
    [System.Serializable]
    public struct BirdSpot
    {
        public string name;
        public string habitat;
        [Tooltip("以画幅左上为原点的位置与点击尺寸，单位为画幅宽高的比例。")]
        public Vector2 position, hitSize;
        public WorkshopLevel chapter;
    }

    public string number, title;
    [TextArea] public string preface;
    public Texture2D painting;
    public WorkshopLevel[] chapters = System.Array.Empty<WorkshopLevel>();
    public Texture2D storyPanorama;
    public Rect storyCrop = new Rect(0,0,1,1);
    public BirdSpot[] birdSpots = System.Array.Empty<BirdSpot>();
    [Tooltip("整卷关卡定稿并实装后勾选，全部寻回即获得完整画作。")]
    public bool allChaptersPublished;

    public bool StageCleared(WorkshopLevel stage,BirdCollection book)
    {
        return stage.perches.Length>0 && book.Count(stage)==stage.perches.Length;
    }
    public int Cleared(BirdCollection book)
    {
        int completed=0;
        foreach(var stage in chapters)if(StageCleared(stage,book))completed++;
        return completed;
    }
    public bool Collected(BirdCollection book)=>allChaptersPublished && chapters.Length>0 && Cleared(book)==chapters.Length;
}
