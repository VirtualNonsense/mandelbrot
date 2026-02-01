using System.Numerics;
using mandelbrot.Model;

namespace mandelbrot.ViewModel;

public sealed class CameraViewModel
{
    private CameraState _state;

    // Tunables (keep explicit)
    private readonly ulong _minZoom;
    private readonly ulong _maxZoom;

    public CameraViewModel(
        CameraState initial,
        ulong minZoom = 1, // pixels per world unit
        ulong maxZoom = ulong.MaxValue) // cap to avoid numeric blowups
    {
        if (minZoom <= 0) throw new ArgumentOutOfRangeException(nameof(minZoom));
        if (maxZoom <= minZoom) throw new ArgumentOutOfRangeException(nameof(maxZoom));

        _minZoom = minZoom;
        _maxZoom = maxZoom;

        _state = initial with { Zoom = Clamp(initial.Zoom, _minZoom, _maxZoom) };
    }

    public CameraState Snapshot() => _state;

    public void SetViewport(PixelSize sizePx)
    {
        // Avoid divide-by-zero and invalid transforms.
        if (sizePx.Width <= 0 || sizePx.Height <= 0)
            return;

        _state = _state with { ViewportPx = sizePx };
    }

    /// <summary>
    /// Pan by a pixel delta (e.g. from drag gesture).
    /// Positive dx = drag right; positive dy = drag down.
    /// This uses "grab & drag content" behavior:
    /// - dragging right moves the fractal right (camera center moves left in world)
    /// - dragging down moves the fractal down (camera center moves up in world, since world Y is up)
    /// </summary>
    public void PanByPixels(PixelDelta deltaPx)
    {
        var s = _state;

        if (!IsViewportValid(s.ViewportPx))
            return;

        double invZoom = 1.0 / s.Zoom;

        // Screen Y down, world Y up -> dy maps with + sign to increase world Y
        float dxWorld = (float)(-deltaPx.X * invZoom);
        float dyWorld = (float)(+deltaPx.Y * invZoom);

        var newCenter = new WorldPoint(s.CenterWorld.Value + new Vector2(dxWorld, dyWorld));
        _state = s with { CenterWorld = newCenter };
    }

 

    /// <summary>
    /// Zoom while keeping the world point under 'anchorPx' fixed on screen.
    /// delta use to increment / decrement the zoom.
    /// </summary>
    public void ZoomAtPixel(PixelPoint anchorPx, int delta)
    {
        var s = _state;
        var zoom = Delta(s.Zoom, delta);

        if (!IsViewportValid(s.ViewportPx))
            return;

        // World position under cursor BEFORE zoom
        WorldPoint worldBefore = ScreenToWorld(anchorPx, s);

        // Update zoom (clamped)
        var newZoom = Clamp(zoom, _minZoom, _maxZoom);
        var s2 = s with { Zoom = newZoom };

        // World position under cursor AFTER zoom (with same center)
        WorldPoint worldAfter = ScreenToWorld(anchorPx, s2);

        // Adjust center so that 'worldBefore' remains under the anchor pixel
        Vector2 deltaWorld = worldBefore.Value - worldAfter.Value;

        var newCenter = new WorldPoint(s2.CenterWorld.Value + deltaWorld);
        _state = s2 with { CenterWorld = newCenter };
    }

    // ---- Coordinate conversions (useful for renderers) ----------------------

    public WorldPoint ScreenToWorld(PixelPoint px) => ScreenToWorld(px, _state);

    public PixelPoint WorldToScreen(WorldPoint world) => WorldToScreen(world, _state);

    private static WorldPoint ScreenToWorld(PixelPoint px, CameraState s)
    {
        // Translate pixels relative to viewport center
        float halfW = (float)(s.ViewportPx.Width * 0.5);
        float halfH = (float)(s.ViewportPx.Height * 0.5);

        // Convert pixels to world units; note Y inversion (screen down -> world up)
        float dxWorld = (px.X - halfW) / s.Zoom;
        float dyWorld = -(px.Y - halfH) / s.Zoom;

        var world = s.CenterWorld.Value + new Vector2(dxWorld, dyWorld);
        return new WorldPoint(world);
    }

    private static PixelPoint WorldToScreen(WorldPoint world, CameraState s)
    {
        float halfW = (float)(s.ViewportPx.Width * 0.5);
        float halfH = (float)(s.ViewportPx.Height * 0.5);

        Vector2 rel = world.Value - s.CenterWorld.Value;

        float xPx = halfW + rel.X * s.Zoom;
        float yPx = halfH + -rel.Y * s.Zoom; // invert back (world up -> screen down)

        return new PixelPoint(new Vector2(xPx, yPx));
    }

    private static bool IsViewportValid(PixelSize s) => s is { Width: > 0, Height: > 0 };

    private static ulong Clamp(ulong v, ulong min, ulong max)
        => v < min ? min : (v > max ? max : v);

    private static ulong Delta(ulong current, int delta)
    {
        ulong abs;
        if (delta > 0)
        {
            abs = (ulong)delta;
            var max = ulong.MaxValue;
            if (max - current <= abs)
            {
                return max;
            }
            
            return current + abs;
        }

        abs = (ulong)Math.Abs(delta);
        if (abs > current)
        {
            return 0;
        }
        return current - abs;
    }
}