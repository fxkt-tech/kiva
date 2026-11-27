using UnityEngine;

public enum SurfaceKind
{
    [InspectorName("岩石面")] Rock,
    [InspectorName("白面")] Plaster,
    [InspectorName("金属面")] Metal,
    [InspectorName("水晶面")] Crystal
}

// Each side has one bit in each of two six-bit planes. The low plane by itself
// has the exact representation of the white-face mask used by existing levels.
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
        return (SurfaceKind)(low + 2 * high);
    }

    public static BlockFaces With(this BlockFaces faces, int side, SurfaceKind kind)
    {
        int low = 1 << side;
        int high = low << 6;
        int otherFaces = (int)faces & ~(low | high);
        int value = (int)kind;
        if ((value & 1) != 0) otherFaces |= low;
        if ((value & 2) != 0) otherFaces |= high;
        return (BlockFaces)otherFaces;
    }

    public static string Label(this SurfaceKind kind) => kind switch
    {
        SurfaceKind.Rock => "岩石面",
        SurfaceKind.Plaster => "白面",
        SurfaceKind.Metal => "金属面",
        SurfaceKind.Crystal => "水晶面",
        _ => throw new System.ArgumentOutOfRangeException(nameof(kind))
    };

    public static bool AllowsInstallation(this SurfaceKind kind) => kind == SurfaceKind.Plaster;
}
