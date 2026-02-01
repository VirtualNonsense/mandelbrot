using System.Numerics;

namespace mandelbrot.Model;

public readonly record struct PixelDelta(Vector2 Value)
{
    public float X => Value.X;
    public float Y => Value.Y;
    public static PixelDelta Of(float dx, float dy) => new(new Vector2(dx, dy));
}