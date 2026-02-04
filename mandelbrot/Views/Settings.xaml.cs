using System.Collections.ObjectModel;
using mandelbrot.Render;
using mandelbrot.Services;

namespace mandelbrot.Views;

public sealed record EnumOption<TEnum>(TEnum Value, string Label)
    where TEnum : struct, Enum
{
    public override string ToString()
    {
        return Label;
    }
};

public partial class Settings : ContentPage
{
    public ObservableCollection<EnumOption<Approach>> Approaches { get; } =
        new(Enum.GetValues<Approach>().Select(v => new EnumOption<Approach>(v, ToLabel(v))));

    private static string ToLabel(Approach approach) => approach switch
    {
        Approach.CSharpBaseLine => "CsharpBaseline",
        Approach.NaiveRustRenderer => "RustRenderer",
        _ => "unknown"
    };

    public EnumOption<Approach> SelectedApproach
    {
        get => new(_state.SelectedApproach, ToLabel(_state.SelectedApproach));
        set => _state.SelectedApproach = value.Value;
    }


    private AppState _state;

    public Settings(AppState state)
    {
        _state = state;
        InitializeComponent();
        BindingContext = this;
        if (Application.Current is App app)
        {
            app.UserAppTheme = app.PlatformAppTheme;
        }
    }


    private void OnToggleTheme(object? sender, EventArgs e)
    {
        if (Application.Current is not App app) return;
        var theme = app.UserAppTheme;
        app.UserAppTheme = theme == AppTheme.Light ? AppTheme.Dark : AppTheme.Light;
    }
}