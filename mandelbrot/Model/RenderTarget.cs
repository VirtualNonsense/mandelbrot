using System.Collections.Concurrent;
using System.Runtime.InteropServices;

namespace mandelbrot.Model;

public sealed class RenderTarget : IDisposable
{
    private readonly PinnedU32Buffer _buf0;
    private readonly PinnedU32Buffer _buf1;

    // Packed (frameId << 1) | frontIndex
    private long _published;

    public int Width { get; }
    public int Height { get; }
    public int RowBytes => _buf0.RowBytes;

    public RenderTarget(int width, int height)
    {
        if (width <= 0) throw new ArgumentOutOfRangeException(nameof(width));
        if (height <= 0) throw new ArgumentOutOfRangeException(nameof(height));

        Width = width;
        Height = height;

        _buf0 = new PinnedU32Buffer(width, height);
        _buf1 = new PinnedU32Buffer(width, height);

        _published = 0; // frameId=0, frontIndex=0
    }

    public (IntPtr frontPtr, int width, int height, int rowBytes, long frameId) GetFrontSnapshot()
    {
        long pub = Volatile.Read(ref _published);
        int frontIndex = (int)(pub & 1L);
        long frameId = pub >> 1;

        var front = frontIndex == 0 ? _buf0 : _buf1;
        return (front.Ptr, Width, Height, front.RowBytes, frameId);
    }

    public (uint[] backArray, long publishedAtStart) GetBackForRender()
    {
        long pub = Volatile.Read(ref _published);
        int frontIndex = (int)(pub & 1L);
        int backIndex = 1 - frontIndex;

        uint[] arr = backIndex == 0 ? _buf0.Array : _buf1.Array;
        return (arr, pub);
    }

    // Try to publish: only succeeds if nobody published since we started (i.e., same pub value)
    public bool TryPublish(long publishedAtStart)
    {
        int oldFront = (int)(publishedAtStart & 1L);
        int newFront = 1 - oldFront;

        long oldFrameId = publishedAtStart >> 1;
        long newFrameId = oldFrameId + 1;

        long newPub = (newFrameId << 1) | (uint)newFront;

        // Ensures UI never sees mismatched frameId/frontIndex.
        return Interlocked.CompareExchange(ref _published, newPub, publishedAtStart) == publishedAtStart;
    }

    public void Dispose()
    {
        _buf0.Dispose();
        _buf1.Dispose();
    }
}

// Your pinned buffer from earlier (unchanged)
public sealed class PinnedU32Buffer : IDisposable
{
    private readonly uint[] _data;
    private GCHandle _handle;

    public int Width { get; }
    public int Height { get; }
    public int RowBytes => checked(Width * 4);
    public uint[] Array => _data;
    public IntPtr Ptr => _handle.AddrOfPinnedObject();

    public PinnedU32Buffer(int width, int height)
    {
        Width = width;
        Height = height;
        _data = GC.AllocateUninitializedArray<uint>(checked(width * height));
        _handle = GCHandle.Alloc(_data, GCHandleType.Pinned);
    }

    public void Dispose()
    {
        if (_handle.IsAllocated) _handle.Free();
    }
}

public sealed class RetiredTargets
{
    private readonly ConcurrentQueue<RenderTarget> _q = new();

    public void Retire(RenderTarget t) => _q.Enqueue(t);

    public void DisposeAll()
    {
        while (_q.TryDequeue(out var t))
            t.Dispose();
    }
}
