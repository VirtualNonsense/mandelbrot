namespace mandelbrot.Services;

public class FrameTimeAverager
{
    private readonly double[] _values;
    private int _index;
    private int _count;

    public FrameTimeAverager(int windowSize)
    {
        if (windowSize <= 0) throw new ArgumentOutOfRangeException(nameof(windowSize));
        _values = new double[windowSize];
    }

    public double AverageMs
    {
        get
        {
            if (_count == 0) return 0.0;
            double sum = 0.0;
            for (int i = 0; i < _count; i++) sum += _values[i];
            return sum / _count;
        }
    }

    public void Push(double ms)
    {
        _values[_index] = ms;
        _index = (_index + 1) % _values.Length;
        if (_count < _values.Length) _count++;
    }
}