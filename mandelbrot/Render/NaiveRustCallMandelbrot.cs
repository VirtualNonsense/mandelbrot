using System.Numerics;
using mandelbrot.Model;
using RustFractals;

namespace mandelbrot.Render;

public class NaiveRustCallMandelbrotIFractalRenderer: IFractalRenderer
{
    public string Name => "Naive unsafe Rust";


    public void Render(CameraState camera, RenderSettings settings, int widthPx, int heightPx, uint[] dst)
    {
        if (widthPx <= 0 || heightPx <= 0) return;
        if (camera.Zoom <= 0) return;
        if (dst.Length < widthPx * heightPx) return;

        Vector2 center = camera.CenterWorld.Value;
        var len = (UIntPtr)dst.Length;
        unsafe
        {
            fixed (uint* pDst = dst)
            {
                NativeMethods.mandelbrot_baseline_render_u32(center.X, center.Y, camera.Zoom, widthPx, heightPx,
                    settings.MaxIterations, pDst, len);
            }
        }


    }
}