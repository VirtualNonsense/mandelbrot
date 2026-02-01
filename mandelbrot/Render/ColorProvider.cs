namespace mandelbrot.Render;

public interface IColorProvider
{
    /// <summary>
    /// Returns a packed color (ARGB32: 0xAARRGGBB).
    /// iteration == maxIteration should typically map to black (inside set).
    /// </summary>
    uint GetColor(int iteration, int maxIteration);
}

public sealed class ClassicColormapProvider : IColorProvider
{
    // Matches your "prered/pregreen/preblue"
    private static readonly byte[] Red   = [0, 0,   0,   0,   128, 255, 255, 255];
    private static readonly byte[] Green = [0, 0,   128, 255, 128, 128, 255, 255];
    private static readonly byte[] Blue  = [0, 255, 255, 128, 0,   0,   128, 255];

    private readonly int _cwidth;

    public ClassicColormapProvider(int cwidth = 50)
    {
        if (cwidth <= 0) throw new ArgumentOutOfRangeException(nameof(cwidth));
        _cwidth = cwidth;
    }

    public uint GetColor(int iteration, int maxIteration)
    {
        if (maxIteration <= 0) return PackArgb(0xFF, 0, 0, 0);
        if (iteration < 0) iteration = 0;

        // Common Mandelbrot convention: points that never escaped -> black.
        if (iteration >= maxIteration) return PackArgb(0xFF, 0, 0, 0);

        // This provider implements the "non-FLAT_CMAP" branch from your C code.
        // In your C, 'val' is a continuous value. With only (iteration, maxIteration),
        // we use val = iteration.
        double val = iteration;

        int mapLen = Red.Length; // 8

        // First ramp segment: val < cwidth
        if (val < _cwidth)
        {
            double t = val / _cwidth; // 0..1
            byte r = LerpByte(Red[0],   Red[1],   t);
            byte g = LerpByte(Green[0], Green[1], t);
            byte b = LerpByte(Blue[0],  Blue[1],  t);
            return PackArgb(0xFF, r, g, b);
        }

        // Cyclic segments
        val -= _cwidth;

        // base = ((int)val / cwidth % (map_len - 1)) + 1;
        int segmentIndex = ((int)(val / _cwidth) % (mapLen - 1)) + 1;

        // perc = (val - (cwidth * (int)(val / cwidth))) / cwidth;
        double frac = (val - (_cwidth * (int)(val / _cwidth))) / _cwidth;

        int top = segmentIndex + 1;
        if (top >= mapLen) top = 1;

        byte rr = LerpByte(Red[segmentIndex],   Red[top],   frac);
        byte gg = LerpByte(Green[segmentIndex], Green[top], frac);
        byte bb = LerpByte(Blue[segmentIndex],  Blue[top],  frac);

        return PackArgb(0xFF, rr, gg, bb);
    }

    private static byte LerpByte(byte a, byte b, double t)
    {
        if (t <= 0) return a;
        if (t >= 1) return b;
        int v = (int)Math.Round(a + (b - a) * t);
        return (byte)Math.Clamp(v, 0, 255);
    }

    /// <summary>ARGB32: 0xAARRGGBB</summary>
    private static uint PackArgb(byte a, byte r, byte g, byte b)
        => (uint)(a << 24 | r << 16 | g << 8 | b);
}
