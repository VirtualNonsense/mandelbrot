using mandelbrot.ViewModel;
using SkiaSharp;
using SkiaSharp.Views.Maui;

namespace mandelbrot.Views;

public partial class RenderPage : ContentPage
{
    private readonly RenderViewModel _vm;

    private SKBitmap? _bitmap;
    private IntPtr _installedPtr = IntPtr.Zero;
    private int _installedW;
    private int _installedH;
    private int _installedRowBytes;


    private (float x, float y)? _dragStart;
    private bool _isPinching;
    private const double PinchZoomSensitivity = 20.0;

    public RenderPage(RenderViewModel viewModel)
    {
        InitializeComponent();

        _vm = viewModel;

        _vm.FrameReady += () =>
        {
            MainThread.BeginInvokeOnMainThread(() =>
            {
                UpdateHud();
                Canvas.InvalidateSurface();
            });
        };

        PinchGestureRecognizer pinchGestureRecognizer = new PinchGestureRecognizer();
        pinchGestureRecognizer.PinchUpdated += OnPinchUpdated;
        Canvas.GestureRecognizers.Add(pinchGestureRecognizer);

        UpdateHud();
    }

    private void OnPinchUpdated(object? sender, PinchGestureUpdatedEventArgs e)
    {
        switch (e.Status)
        {
            case GestureStatus.Started:
                _isPinching = true;
                return;

            case GestureStatus.Running:
                if (!_isPinching)
                    return;

                if (_installedW <= 0 || _installedH <= 0)
                    return;


                var anchorX = (float)(e.ScaleOrigin.X * _installedW);
                var anchorY = (float)(e.ScaleOrigin.Y * _installedH);

                var deltaD = e.Scale - 1;

                // if (Math.Abs(deltaD) < 0.01)
                //     return;
                deltaD *= PinchZoomSensitivity;

                var delta =
                    deltaD >= int.MaxValue ? int.MaxValue :
                    deltaD <= int.MinValue ? int.MinValue :
                    (int)deltaD;

                if (delta != 0)
                {
                    _vm.ZoomAtPixel(anchorX, anchorY, delta);
                    // If your VM already triggers FrameReady, you can omit InvalidateSurface here.
                    Canvas.InvalidateSurface();
                }

                return;

            case GestureStatus.Completed:
            case GestureStatus.Canceled:
                _isPinching = false;
                return;
        }
    }

    protected override void OnAppearing()
    {
        base.OnAppearing();
        _vm.Start();
    }

    protected override void OnDisappearing()
    {
        _vm.Stop();
        base.OnDisappearing();
    }

    private void UpdateHud()
    {
        ApproachLabel.Text = _vm.HudApproach;
        FrameLabel.Text = _vm.HudStats;
    }

    private void OnPaintSurface(object? sender, SKPaintSurfaceEventArgs e)
    {
        _vm.SetViewport(e.Info.Width, e.Info.Height);

        var canvas = e.Surface.Canvas;
        canvas.Clear(SKColors.Black);

        var frame = _vm.TryGetLatestFrame();
        if (frame is null) return;

        var (frontPtr, w, h, rowBytes, _) = frame.Value;

        EnsureInstalledBitmap(frontPtr, w, h, rowBytes);
        canvas.DrawBitmap(_bitmap!, 0, 0);

        // Now it's safe to dispose targets retired by SetViewport, because we already reinstalled to current ptr.
        _vm.DisposeRetiredTargetsOnUiThread();
    }

    private void EnsureInstalledBitmap(IntPtr ptr, int w, int h, int rowBytes)
    {
        if (_bitmap is not null &&
            _installedPtr == ptr &&
            _installedW == w &&
            _installedH == h &&
            _installedRowBytes == rowBytes)
            return;

        _bitmap?.Dispose();

        // IMPORTANT: choose the format that matches your renderer output.
        var info = new SKImageInfo(w, h, SKColorType.Bgra8888, SKAlphaType.Premul);

        var bmp = new SKBitmap();
        if (!bmp.InstallPixels(info, ptr, rowBytes))
        {
            bmp.Dispose();
            throw new InvalidOperationException("InstallPixels failed.");
        }

        _bitmap = bmp;
        _installedPtr = ptr;
        _installedW = w;
        _installedH = h;
        _installedRowBytes = rowBytes;
    }


    private void OnCanvasTouch(object? sender, SKTouchEventArgs e)
    {
        if (_isPinching)
            return;
        switch (e.ActionType)
        {
            case SKTouchAction.Pressed:
                _dragStart = (e.Location.X, e.Location.Y);
                break;

            case SKTouchAction.Moved:
                if (_dragStart is not null)
                {
                    float dx = e.Location.X - _dragStart.Value.x;
                    float dy = e.Location.Y - _dragStart.Value.y;

                    _vm.PanByPixels(dx, dy);
                    _dragStart = (e.Location.X, e.Location.Y);
                }

                break;

            case SKTouchAction.Released:
            case SKTouchAction.Cancelled:
                _dragStart = null;
                break;
            case SKTouchAction.WheelChanged:
                var delta = e.WheelDelta;
                _vm.ZoomAtPixel(e.Location.X, e.Location.Y, delta);
                break;
        }

        e.Handled = true;
    }
}