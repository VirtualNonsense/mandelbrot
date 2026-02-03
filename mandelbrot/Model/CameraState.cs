using System.Numerics;

namespace mandelbrot.Model;

public readonly record struct CameraState(
    WorldPoint CenterWorld,
    ulong InitialZoom,
    ulong Zoom,
    PixelSize ViewportPx
)
{
    public double Magnification()
    {
        return (double)Zoom / InitialZoom;
    } 
};
