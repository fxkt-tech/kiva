using UnityEngine;

public enum SurfaceKind
{
    [InspectorName("岩石面")] Rock,
    [InspectorName("白面")] Plaster,
    [InspectorName("金属面")] Metal,
    [InspectorName("水晶面")] Crystal,
    [InspectorName("清水面")] Water,
    [InspectorName("玻璃面")] Glass
}

// Bit planes are appended above the old data so four-finish levels and the
// original white-face masks retain their serialized values.
public enum BlockFaces
{
    Rock = 0, WhiteLeft = 1, WhiteRight = 2, WhiteBottom = 4, WhiteTop = 8,
    WhiteBack = 16, WhiteFront = 32, AllWhite = 63
}

public static class SurfaceKinds
{
    public static SurfaceKind At(this BlockFaces faces, int side)
    {
        int bits = (int)faces;
        int low = (bits >> side) & 1;
        int high = (bits >> (side + 6)) & 1;
        int transparent = (bits >> (side + 12)) & 1;
        return (SurfaceKind)(low + 2 * high + 4 * transparent);
    }

    public static BlockFaces With(this BlockFaces faces, int side, SurfaceKind kind)
    {
        int low = 1 << side;
        int high = low << 6;
        int third = high << 6;
        int otherFaces = (int)faces & ~(low | high | third);
        int value = (int)kind;
        if ((value & 1) != 0) otherFaces |= low;
        if ((value & 2) != 0) otherFaces |= high;
        if ((value & 4) != 0) otherFaces |= third;
        return (BlockFaces)otherFaces;
    }

    public static string Label(this SurfaceKind kind) => kind switch
    {
        SurfaceKind.Rock => "岩石面",
        SurfaceKind.Plaster => "白面",
        SurfaceKind.Metal => "金属面",
        SurfaceKind.Crystal => "水晶面",
        SurfaceKind.Water => "清水面",
        SurfaceKind.Glass => "玻璃面",
        _ => throw new System.ArgumentOutOfRangeException(nameof(kind))
    };

    public static bool AllowsInstallation(this SurfaceKind kind) => kind == SurfaceKind.Plaster;
    public static bool IsTransparent(this SurfaceKind kind) => kind == SurfaceKind.Water || kind == SurfaceKind.Glass;
}
