using System.Numerics;

namespace mandelbrot.Model;

public readonly record struct WorldPoint(Vector2 Value)
{
    public float X => Value.X;
    public float Y => Value.Y;
    public static WorldPoint Of(float x, float y) => new(new Vector2(x, y));
}