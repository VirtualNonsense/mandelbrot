using System.Numerics;

namespace mandelbrot.Model;

public readonly record struct CameraState(
    WorldPoint CenterWorld,
    ulong Zoom,
    PixelSize ViewportPx
);
