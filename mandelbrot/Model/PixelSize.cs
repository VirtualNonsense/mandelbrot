namespace mandelbrot.Model;

public readonly record struct PixelSize(int Width, int Height)
{
    public static PixelSize Of(int width, int height) => new(width, height);
}