using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace mandelbrot.Views;

public partial class Settings : ContentPage
{
    public Settings()
    {
        InitializeComponent();
        if (Application.Current is App app)
        {
            app.UserAppTheme = app.PlatformAppTheme;
        }
    }

    private void OnToggleTheme(object? sender, EventArgs e)
    {
        if (Application.Current is not App app) return;
        var theme = app.UserAppTheme;
        app.UserAppTheme = theme == AppTheme.Light? AppTheme.Dark : AppTheme.Light;
    }
}