using System.ComponentModel;
using mandelbrot.Render;

namespace mandelbrot.Services;

public class AppState : INotifyPropertyChanged
{
    public event PropertyChangedEventHandler? PropertyChanged;

    public Approach SelectedApproach
    {
        get;
        set
        {
            if (field == value) return;
            field = value;
            PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(nameof(SelectedApproach)));
        }
    } = Approach.CSharpBaseLine;
}