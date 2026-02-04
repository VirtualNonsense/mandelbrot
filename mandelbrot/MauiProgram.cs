using mandelbrot.Model;
using mandelbrot.Render;
using mandelbrot.Services;
using mandelbrot.ViewModel;
using Microsoft.Extensions.Logging;
using SkiaSharp.Views.Maui.Controls.Hosting;

namespace mandelbrot
{
    public static class MauiProgram
    {
        public static MauiApp CreateMauiApp()
        {
            var builder = MauiApp.CreateBuilder();
            builder
                .UseSkiaSharp()
                .UseMauiApp<App>()
                .ConfigureFonts(fonts =>
                {
                    fonts.AddFont("OpenSans-Regular.ttf", "OpenSansRegular");
                    fonts.AddFont("OpenSans-Semibold.ttf", "OpenSansSemibold");
                });

            builder.Services.AddSingleton<AppState>(_ =>
            {
                var state = new AppState();
                state.SelectedApproach = Approach.CSharpBaseLine;
                return state;
            });
            builder.Services.AddSingleton<IColorProvider, ClassicColormapProvider>();
            builder.Services.AddSingleton<RendererRegistry>(sp => new RendererRegistry([
                (Approach.CSharpBaseLine, new MandelbrotBaselineRenderer(sp.GetService<IColorProvider>())),
                (Approach.NaiveRustRenderer, new NaiveRustCallMandelbrotIFractalRenderer()),
            ]));
            builder.Services.AddSingleton<CameraViewModel>(_ =>
            {
                var initialCamera = new CameraState(
                    CenterWorld: WorldPoint.Of(-0.5f, 0.0f),
                    InitialZoom: 300,
                    Zoom: 300,
                    ViewportPx: PixelSize.Of(1, 1)
                );
                return new CameraViewModel(initialCamera, 200);
            });

            builder.Services.AddTransient<RenderViewModel>();

#if DEBUG
            builder.Logging.AddDebug();
#endif

            return builder.Build();
        }
    }
}