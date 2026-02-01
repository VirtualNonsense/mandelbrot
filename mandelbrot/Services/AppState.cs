using System.ComponentModel;
using mandelbrot.Render;

namespace mandelbrot.Services;

public class AppState : INotifyPropertyChanged
{
    public event PropertyChangedEventHandler? PropertyChanged;

    private Approach _selectedApproach = Approach.CSharpBaseLine;

    public Approach SelectedApproach
    {
        get => _selectedApproach;
        set
        {
            if (_selectedApproach == value) return;
            _selectedApproach = value;
            PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(nameof(SelectedApproach)));
        }
    }
}