using System.ComponentModel;
using System.Diagnostics;
using mandelbrot.Model;
using mandelbrot.Render;
using mandelbrot.Services;

namespace mandelbrot.ViewModel;

public sealed class RenderViewModel : INotifyPropertyChanged, IDisposable
{
    public event PropertyChangedEventHandler? PropertyChanged;

    // View subscribes; View must marshal to UI thread.
    public event Action? FrameReady;

    private readonly AppState _appState;
    private readonly RendererRegistry _registry;

    private readonly CameraViewModel _camera;
    private readonly Lock _cameraGate = new();

    private readonly FrameTimeAverager _avgCompute = new(windowSize: 60);


    private RenderTarget? _target;
    private readonly RetiredTargets _retired = new();


    private RenderSettings _settings = RenderSettings.Default;

    private CancellationTokenSource? _cts;
    private Task? _task;


    private readonly Stopwatch _fpsClock = Stopwatch.StartNew();
    private long _framesComputedWindow;
    private long _windowStartTicks;
    private double _computeFps;

    private readonly ManualResetEventSlim _renderSignal = new(initialState: false);
    private int _pendingRequests; // coalescing counter

    public RenderViewModel(AppState appState, RendererRegistry registry, CameraViewModel initialCamera)
    {
        _camera = initialCamera;
        _appState = appState;
        _registry = registry;


        _appState.PropertyChanged += (_, e) =>
        {
            if (e.PropertyName == nameof(AppState.SelectedApproach))
            {
                RequestFrame();
                OnPropertyChanged(nameof(HudApproach));
            }
        };
    }

    public string HudApproach => $"Approach: {_appState.SelectedApproach}";
    public string HudStats => $"Compute: {_computeFps:0.0} fps | Avg: {_avgCompute.AverageMs:0.00} ms | {_target?.Width}×{_target?.Height}px";

    public void SetMaxIterations(int maxIter)
    {
        if (maxIter <= 0) return;
        _settings = new RenderSettings(maxIter);
        RequestFrame();
    }


    public void SetViewport(int widthPx, int heightPx)
    {
        if (widthPx <= 0 || heightPx <= 0) return;

        var current = Volatile.Read(ref _target);
        if (current is not null && current.Width == widthPx && current.Height == heightPx)
            return;

        var next = new RenderTarget(widthPx, heightPx);

        // Atomic swap of the whole target.
        var old = Interlocked.Exchange(ref _target, next);
        if (old is not null)
            _retired.Retire(old);

        lock (_cameraGate)
        {
            _camera.SetViewport(PixelSize.Of(widthPx, heightPx));
        }

        RequestFrame();
        OnPropertyChanged(nameof(HudStats));
    }

    public void DisposeRetiredTargetsOnUiThread() => _retired.DisposeAll();


    public void PanByPixels(float dxPx, float dyPx)
    {
        lock (_cameraGate)
        {
            _camera.PanByPixels(PixelDelta.Of(dxPx, dyPx));
        }

        RequestFrame();
    }

    public void ZoomAtPixel(float anchorXPx, float anchorYPx, int delta)
    {
        lock (_cameraGate)
        {
            _camera.ZoomAtPixel(PixelPoint.Of(anchorXPx, anchorYPx), delta);
        }

        RequestFrame();
    }

    public void RequestFrame()
    {
        Interlocked.Increment(ref _pendingRequests);
        _renderSignal.Set();
    }


    public (IntPtr frontPtr, int widthPx, int heightPx, int rowBytes, long frameId)? TryGetLatestFrame()
    {
        var t = Volatile.Read(ref _target);
        if (t is null) return null;
        return t.GetFrontSnapshot();
    }


    public void Start()
    {
        if (_cts is not null) return;

        _cts = new CancellationTokenSource();
        _task = Task.Run(() => ComputeLoop(_cts.Token));
    }

    public void Stop()
    {
        var cts = _cts;
        var task = _task;

        _cts = null;
        _task = null;

        if (cts is null) return;

        cts.Cancel();
        _renderSignal.Set(); // wake wait
        cts.Dispose();
        _ = task;
    }

    private CameraState SnapshotCamera()
    {
        lock (_cameraGate)
        {
            return _camera.Snapshot();
        }
    }

    private void ComputeLoop(CancellationToken ct)
    {
        var fpsClock = Stopwatch.StartNew();
        long windowStartTicks = fpsClock.ElapsedTicks;

        while (!ct.IsCancellationRequested)
        {
            // Wait for work unless you truly want to render nonstop regardless of input.
            _renderSignal.Wait(ct);
            if (ct.IsCancellationRequested) break;

            // coalesce requests
            Interlocked.Exchange(ref _pendingRequests, 0);
            _renderSignal.Reset();

            var t = Volatile.Read(ref _target);
            if (t is null) continue;

            var camera = SnapshotCamera();
            var renderer = _registry.Get(_appState.SelectedApproach);

            // Determine which back buffer to write, based on the published state at start.
            var (backArray, publishedAtStart) = t.GetBackForRender();

            var sw = Stopwatch.StartNew();
            renderer.Render(camera, _settings, t.Width, t.Height, backArray);
            sw.Stop();
            _avgCompute.Push(sw.Elapsed.TotalMilliseconds);

            // If target changed mid-render (resize), discard.
            if (!ReferenceEquals(t, Volatile.Read(ref _target)))
                continue;

            // Publish atomically (frameId + frontIndex). If someone else published, that's fine: discard.
            if (!t.TryPublish(publishedAtStart))
                continue;

            // Throughput
            _framesComputedWindow++;
            var nowTicks = fpsClock.ElapsedTicks;
            double elapsedSec = (nowTicks - windowStartTicks) / (double)Stopwatch.Frequency;
            if (elapsedSec >= 1.0)
            {
                _computeFps = _framesComputedWindow / elapsedSec;
                _framesComputedWindow = 0;
                windowStartTicks = nowTicks;
                OnPropertyChanged(nameof(HudStats));
            }

            FrameReady?.Invoke();
        }
    }


    private void UpdateThroughput()
    {
        _framesComputedWindow++;

        long now = _fpsClock.ElapsedTicks;
        double elapsedSec = (now - _windowStartTicks) / (double)Stopwatch.Frequency;

        if (elapsedSec >= 1.0)
        {
            _computeFps = _framesComputedWindow / elapsedSec;
            _framesComputedWindow = 0;
            _windowStartTicks = now;
            OnPropertyChanged(nameof(HudStats));
        }
    }


    private void OnPropertyChanged(string name)
        => PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(name));

    public void Dispose()
    {
        Stop();
    }
}