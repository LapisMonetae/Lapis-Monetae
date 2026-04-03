using System.Collections.ObjectModel;
using CommunityToolkit.Mvvm.ComponentModel;

namespace LmtDesktop.Core.Models;

/// <summary>
/// Base ViewModel with toast notifications and busy state.
/// Both wallet and miner GUIs inherit from this.
/// </summary>
public partial class ViewModelBase : ObservableObject
{
    // ── Toast ──
    [ObservableProperty] private ToastMessage? _activeToast;
    [ObservableProperty] private bool _toastVisible;

    private System.Threading.CancellationTokenSource? _toastCts;

    public async void ShowToast(string message, ToastKind kind = ToastKind.Info)
    {
        _toastCts?.Cancel();
        _toastCts = new System.Threading.CancellationTokenSource();
        var ct = _toastCts.Token;

        ActiveToast = new ToastMessage(message, kind);
        ToastVisible = true;

        try
        {
            await System.Threading.Tasks.Task.Delay(2500, ct);
            ToastVisible = false;
            await System.Threading.Tasks.Task.Delay(300, ct); // fade-out duration
            ActiveToast = null;
        }
        catch (System.Threading.Tasks.TaskCanceledException) { }
    }

    // ── Busy ──
    [ObservableProperty] private bool _isBusy;
    [ObservableProperty] private string _busyText = "";

    public bool IsNotBusy => !IsBusy;

    partial void OnIsBusyChanged(bool value)
    {
        OnPropertyChanged(nameof(IsNotBusy));
    }

    protected void SetBusy(string text = "Working...")
    {
        BusyText = text;
        IsBusy = true;
    }

    protected void ClearBusy()
    {
        IsBusy = false;
        BusyText = "";
    }
}
