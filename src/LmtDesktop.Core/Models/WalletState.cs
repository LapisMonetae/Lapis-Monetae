using CommunityToolkit.Mvvm.ComponentModel;

namespace LmtDesktop.Core.Models;

public partial class WalletState : ObservableObject
{
    [ObservableProperty] private string? _cliBinary;
    [ObservableProperty] private bool _walletOpen;
    [ObservableProperty] private string _walletName = "";
    [ObservableProperty] private string _network = "mainnet";
    [ObservableProperty] private bool _busy;
    [ObservableProperty] private bool _nodeConnected;
    [ObservableProperty] private bool? _nodeSynced;
    [ObservableProperty] private bool _pendingWalletOpen;

    public bool CanRunAction => CliBinary != null && !Busy;
    public bool CanSend => CanRunAction && WalletOpen;

    public (string Text, string Bg) PillInfo
    {
        get
        {
            if (CliBinary == null) return ("NO CLI", "#6b7a94");
            if (Busy) return ("RUNNING...", "#f59e0b");
            if (PendingWalletOpen) return ("OPENING...", "#f59e0b");
            if (WalletOpen) return ("WALLET OPEN", "#22c55e");
            return ("READY", "#3b82f6");
        }
    }

    public string StatusText
    {
        get
        {
            var parts = new List<string>();
            if (CliBinary == null) parts.Add("CLI not found");
            else if (WalletOpen) parts.Add($"Wallet: {WalletName}");
            else parts.Add("No wallet open");

            if (NodeConnected)
                parts.Add(NodeSynced == true ? "Node: synced" : "Node: syncing");
            else
                parts.Add("Node: disconnected");

            return string.Join(" | ", parts);
        }
    }

    public void OpenWallet(string name)
    {
        WalletName = name;
        WalletOpen = true;
        PendingWalletOpen = false;
        OnPropertyChanged(nameof(CanRunAction));
        OnPropertyChanged(nameof(CanSend));
        OnPropertyChanged(nameof(PillInfo));
        OnPropertyChanged(nameof(StatusText));
    }

    public void CloseWallet()
    {
        WalletOpen = false;
        WalletName = "";
        PendingWalletOpen = false;
        OnPropertyChanged(nameof(CanRunAction));
        OnPropertyChanged(nameof(CanSend));
        OnPropertyChanged(nameof(PillInfo));
        OnPropertyChanged(nameof(StatusText));
    }

    public void SetNodeStatus(bool connected, bool? synced)
    {
        NodeConnected = connected;
        NodeSynced = synced;
        OnPropertyChanged(nameof(StatusText));
    }
}
