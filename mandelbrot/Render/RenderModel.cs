namespace mandelbrot.Render;

public readonly record struct RenderSettings(int MaxIterations)
{
    public static RenderSettings Default => new(MaxIterations: 256);
}

public readonly record struct RenderStats(
    double AvgComputeMs,
    double ComputeFps,
    int Width,
    int Height,
    long FramesComputedTotal
);