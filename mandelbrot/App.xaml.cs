

namespace mandelbrot
{
    public partial class App : Application
    {
        public App()
        {
            InitializeComponent();
        }

        protected override Window CreateWindow(IActivationState? activationState)
        {
            var t = RustFractals.NativeMethods.test();
            return new Window(new AppShell());
        }
    }
}