namespace mandelbrot.Render;

public sealed class RendererRegistry
{
    private readonly Dictionary<Approach, IFractalRenderer> _map;

    public RendererRegistry(IEnumerable<(Approach key, IFractalRenderer renderer)> renderers)
    {
        _map = renderers.ToDictionary(x => x.key, x => x.renderer);
    }

    public IFractalRenderer Get(Approach approach)
    {
        if (_map.TryGetValue(approach, out var r)) return r;
        throw new KeyNotFoundException($"No renderer registered for {approach}.");
    }
}