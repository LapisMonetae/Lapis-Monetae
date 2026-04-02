using System;
using System.Collections.ObjectModel;
using System.Diagnostics;
using System.Linq;
using System.Runtime.InteropServices;
using System.Threading;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using LmtDesktop.Core.Helpers;
using LmtDesktop.Core.Validation;

namespace LmtDesktop.Miner.ViewModels;

public partial class MinerMainViewModel : ObservableObject
{
    // Bridge settings
    [ObservableProperty] private string _bridgePath = "";
    [ObservableProperty] private string _listenAddress = "0.0.0.0:3333";
    [ObservableProperty] private string _rpcUrl = "grpc://127.0.0.1:26110";
    [ObservableProperty] private string _payAddress = "";
    [ObservableProperty] private string _extraDataHex = "";
    [ObservableProperty] private int _refreshMs = 5000;
    [ObservableProperty] private bool _allowNonDaa = true;
    [ObservableProperty] private bool _autoRestartBridge = true;

    // Miner settings
    [ObservableProperty] private string _xmrigPath = "";
    [ObservableProperty] private string _xmrigUrl = "stratum+tcp://127.0.0.1:3333";
    [ObservableProperty] private string _xmrigExtra = "";
    [ObservableProperty] private bool _startMinerWithBridge;
    [ObservableProperty] private bool _autoRestartMiner;

    // Status
    [ObservableProperty] private string _bridgeStatus = "Stopped";
    [ObservableProperty] private string _minerStatus = "Stopped";
    [ObservableProperty] private string _bridgePillBg = "#6b7a94";
    [ObservableProperty] private string _minerPillBg = "#6b7a94";

    // Metrics
    [ObservableProperty] private string _hashrate10S = "—";
    [ObservableProperty] private string _hashrate60S = "—";
    [ObservableProperty] private string _hashrate15M = "—";
    [ObservableProperty] private string _accepted = "0";
    [ObservableProperty] private string _rejected = "0";
    [ObservableProperty] private string _poolLatency = "—";
    [ObservableProperty] private string _bridgeUptime = "—";
    [ObservableProperty] private string _minerUptime = "—";

    // Console
    public ObservableCollection<string> ConsoleLines { get; } = new();

    private Process? _bridgeProcess;
    private Process? _minerProcess;
    private DateTime? _bridgeStartTime;
    private DateTime? _minerStartTime;
    private int _bridgeRestarts;
    private int _minerRestarts;
    private CancellationTokenSource? _metricsCts;

    public MinerMainViewModel()
    {
        StartMetricsTimer();
    }

    private void ConsoleLog(string tag, string msg)
    {
        var ts = DateTime.Now.ToString("HH:mm:ss");
        ConsoleLines.Add($"[{ts}] [{tag}] {msg}");
        if (ConsoleLines.Count > 2000) ConsoleLines.RemoveAt(0);
    }

    [RelayCommand]
    private async Task StartBridge()
    {
        if (_bridgeProcess != null && !_bridgeProcess.HasExited)
        { ConsoleLog("system", "Bridge already running."); return; }

        // Validate
        if (string.IsNullOrWhiteSpace(BridgePath))
        { ConsoleLog("system", "Bridge binary path is empty."); return; }
        var addrResult = AddressValidator.Validate(PayAddress);
        if (!addrResult.Valid)
        { ConsoleLog("system", $"Pay address invalid: {addrResult.Error}"); return; }

        var args = $"--listen {ListenAddress} --rpc-url {RpcUrl} --pay-address {PayAddress} --refresh-ms {RefreshMs}";
        if (!string.IsNullOrWhiteSpace(ExtraDataHex)) args += $" --extra-data-hex {ExtraDataHex}";
        if (AllowNonDaa) args += " --allow-non-daa";

        ConsoleLog("bridge", $"Starting: {BridgePath} {args}");

        var psi = new ProcessStartInfo
        {
            FileName = BridgePath,
            Arguments = args,
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            UseShellExecute = false,
            CreateNoWindow = true,
        };

        _bridgeProcess = Process.Start(psi);
        if (_bridgeProcess == null) { ConsoleLog("system", "Failed to start bridge."); return; }

        _bridgeStartTime = DateTime.Now;
        BridgeStatus = "Running";
        BridgePillBg = "#22c55e";

        _ = Task.Run(() => PumpOutput(_bridgeProcess, "bridge"));

        if (StartMinerWithBridge)
        {
            await Task.Delay(2000);
            await StartMiner();
        }
    }

    [RelayCommand]
    private void StopBridge()
    {
        if (_bridgeProcess == null || _bridgeProcess.HasExited) return;
        ConsoleLog("bridge", "Stopping bridge...");
        try { _bridgeProcess.Kill(entireProcessTree: true); } catch { }
        BridgeStatus = "Stopped";
        BridgePillBg = "#6b7a94";
        _bridgeStartTime = null;
        _bridgeRestarts = 0;
        StopMiner();
    }

    [RelayCommand]
    private async Task StartMiner()
    {
        if (_minerProcess != null && !_minerProcess.HasExited)
        { ConsoleLog("system", "Miner already running."); return; }

        if (string.IsNullOrWhiteSpace(XmrigPath))
        { ConsoleLog("system", "XMRig binary path is empty."); return; }

        var args = $"-o {XmrigUrl} -u {PayAddress} -p x";
        if (!string.IsNullOrWhiteSpace(XmrigExtra)) args += $" {XmrigExtra}";

        ConsoleLog("miner", $"Starting: {XmrigPath} {args}");

        var psi = new ProcessStartInfo
        {
            FileName = XmrigPath,
            Arguments = args,
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            UseShellExecute = false,
            CreateNoWindow = true,
        };

        _minerProcess = Process.Start(psi);
        if (_minerProcess == null) { ConsoleLog("system", "Failed to start miner."); return; }

        _minerStartTime = DateTime.Now;
        MinerStatus = "Running";
        MinerPillBg = "#22c55e";

        _ = Task.Run(() => PumpOutput(_minerProcess, "miner"));
    }

    [RelayCommand]
    private void StopMiner()
    {
        if (_minerProcess == null || _minerProcess.HasExited) return;
        ConsoleLog("miner", "Stopping miner...");
        try { _minerProcess.Kill(entireProcessTree: true); } catch { }
        MinerStatus = "Stopped";
        MinerPillBg = "#6b7a94";
        _minerStartTime = null;
        _minerRestarts = 0;
    }

    private async Task PumpOutput(Process proc, string tag)
    {
        try
        {
            while (!proc.HasExited)
            {
                var line = await proc.StandardOutput.ReadLineAsync();
                if (line == null) break;

                Avalonia.Threading.Dispatcher.UIThread.Post(() =>
                {
                    ConsoleLog(tag, line);
                    if (tag == "miner")
                    {
                        var m = MetricsParser.ParseLine(line);
                        if (m != null)
                        {
                            if (!string.IsNullOrEmpty(m.Hashrate10S)) Hashrate10S = m.Hashrate10S;
                            if (!string.IsNullOrEmpty(m.Hashrate60S)) Hashrate60S = m.Hashrate60S;
                            if (!string.IsNullOrEmpty(m.Hashrate15M)) Hashrate15M = m.Hashrate15M;
                            if (m.Accepted > 0) Accepted = m.Accepted.ToString();
                            if (m.Rejected > 0) Rejected = m.Rejected.ToString();
                            if (m.LatencyMs.HasValue) PoolLatency = $"{m.LatencyMs} ms";
                        }
                    }
                });
            }
        }
        catch { }

        Avalonia.Threading.Dispatcher.UIThread.Post(() =>
        {
            if (tag == "bridge")
            { BridgeStatus = "Stopped"; BridgePillBg = "#6b7a94"; _bridgeStartTime = null; }
            else
            { MinerStatus = "Stopped"; MinerPillBg = "#6b7a94"; _minerStartTime = null; }
            ConsoleLog("system", $"{tag} process exited.");
        });
    }

    private void StartMetricsTimer()
    {
        _metricsCts = new CancellationTokenSource();
        _ = Task.Run(async () =>
        {
            while (!_metricsCts.Token.IsCancellationRequested)
            {
                await Task.Delay(1000, _metricsCts.Token);
                Avalonia.Threading.Dispatcher.UIThread.Post(() =>
                {
                    BridgeUptime = _bridgeStartTime.HasValue
                        ? (DateTime.Now - _bridgeStartTime.Value).ToString(@"hh\:mm\:ss") : "—";
                    MinerUptime = _minerStartTime.HasValue
                        ? (DateTime.Now - _minerStartTime.Value).ToString(@"hh\:mm\:ss") : "—";
                });
            }
        });
    }
}
