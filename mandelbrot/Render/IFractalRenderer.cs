using mandelbrot.Model;
using SkiaSharp;

namespace mandelbrot.Render;

public interface IFractalRenderer
{
    string Name { get; }

    void Render(
        CameraState camera,
        RenderSettings settings,
        int widthPx,
        int heightPx,
        uint[] dst);
}