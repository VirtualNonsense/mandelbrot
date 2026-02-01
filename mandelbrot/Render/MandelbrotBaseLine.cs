using System.Numerics;
using mandelbrot.Model;

namespace mandelbrot.Render;

public sealed class MandelbrotBaselineRenderer(IColorProvider? colorProvider) : IFractalRenderer
{
    public string Name => "C# Baseline";

    private readonly IColorProvider _colorProvider = colorProvider!;

    public void Render(CameraState camera, RenderSettings settings, int widthPx, int heightPx, uint[] dst)
    {
        if (widthPx <= 0 || heightPx <= 0) return;
        if (camera.Zoom <= 0) return;
        if (dst.Length < widthPx * heightPx) return;

        double invZoom = 1.0 / camera.Zoom; // world units per pixel

        double halfW = widthPx * 0.5;
        double halfH = heightPx * 0.5;


        Vector2 center = camera.CenterWorld.Value;
        int maxIter = settings.MaxIterations;

        Parallel.For(0, heightPx, py =>
        {
            // screen Y down, world Y up -> invert
            double yWorld = center.Y + (-(py - halfH) * invZoom);
            int row = py * widthPx;

            for (int px = 0; px < widthPx; px++)
            {
                double xWorld = center.X + ((px - halfW) * invZoom);

                int iter = Iterate(xWorld, yWorld, maxIter);

                dst[row + px] = _colorProvider.GetColor(iter, maxIter);
            }
        });
    }

    private static int Iterate(double x0, double y0, int maxIter)
    {
        double x = 0.0, y = 0.0;
        int i = 0;

        while (i < maxIter)
        {
            double xx = x * x - y * y + x0;
            double yy = 2.0 * x * y + y0;
            x = xx;
            y = yy;

            if ((x * x + y * y) > 4.0) break;
            i++;
        }

        return i;
    }
}