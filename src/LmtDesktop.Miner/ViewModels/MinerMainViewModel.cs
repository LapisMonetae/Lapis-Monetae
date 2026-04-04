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
using LmtDesktop.Core.Models;
using LmtDesktop.Core.Validation;

namespace LmtDesktop.Miner.ViewModels;

public partial class MinerMainViewModel : ViewModelBase
{
    // Bridge
    [ObservableProperty] private string _bridgePath = "";
    [ObservableProperty] private string _listenAddress = "0.0.0.0:3333";
    [ObservableProperty] private string _rpcUrl = "grpc://127.0.0.1:26110";
    [ObservableProperty] private string _payAddress = "";
    [ObservableProperty] private string _extraDataHex = "";
    [ObservableProperty] private int _refreshMs = 5000;
    [ObservableProperty] private bool _allowNonDaa = true;
    [ObservableProperty] private bool _autoRestartBridge = true;

    // Miner
    [ObservableProperty] private string _xmrigPath = "";
    [ObservableProperty] private string _xmrigUrl = "stratum+tcp://127.0.0.1:3333";
    [ObservableProperty] private string _xmrigExtra = "";
    [ObservableProperty] private bool _startMinerWithBridge;
    [ObservableProperty] private bool _autoRestartMiner;

    // Status
    [ObservableProperty] private string _bridgeStatus = "Stopped";
    [ObservableProperty] private string _minerStatus = "Stopped";
    [ObservableProperty] private string _bridgePillBg = "#94a3b8";
    [ObservableProperty] private string _minerPillBg = "#94a3b8";
    [ObservableProperty] private bool _bridgeRunning;
    [ObservableProperty] private bool _minerRunning;

    // Metrics
    [ObservableProperty] private string _hashrate10S = "—";
    [ObservableProperty] private string _hashrate60S = "—";
    [ObservableProperty] private string _hashrate15M = "—";
    [ObservableProperty] private string _accepted = "0";
    [ObservableProperty] private string _rejected = "0";
    [ObservableProperty] private string _poolLatency = "—";
    [ObservableProperty] private string _bridgeUptime = "—";
    [ObservableProperty] private string _minerUptime = "—";

    public ObservableCollection<string> ConsoleLines { get; } = new();

    private Process? _bridgeProcess;
    private Process? _minerProcess;
    private DateTime? _bridgeStartTime;
    private DateTime? _minerStartTime;
    private int _bridgeRestartCount;
    private int _minerRestartCount;
    private DateTime? _bridgeLastStableTime;
    private DateTime? _minerLastStableTime;

    private const int MAX_RESTARTS = 5;
    private const int RESTART_RESET_SEC = 60;

    public MinerMainViewModel()
    {
        _ = MetricsTimerLoop();
    }

    private void ConsoleLog(string tag, string msg)
    {
        ConsoleLines.Add($"[{DateTime.Now:HH:mm:ss}] [{tag}] {msg}");
        if (ConsoleLines.Count > 2000) ConsoleLines.RemoveAt(0);
    }

    // ═══════════════════════════════════
    //  BRIDGE
    // ═══════════════════════════════════

    [RelayCommand]
    private async Task StartBridge()
    {
        if (BridgeRunning) { ShowToast("Bridge already running.", ToastKind.Warn); return; }
        if (string.IsNullOrWhiteSpace(BridgePath)) { ShowToast("Set bridge binary path.", ToastKind.Error); return; }
        if (!System.IO.File.Exists(BridgePath)) { ShowToast("Bridge binary not found.", ToastKind.Error); return; }
        var addrResult = AddressValidator.Validate(PayAddress);
        if (!addrResult.Valid) { ShowToast($"Pay address: {addrResult.Error}", ToastKind.Error); return; }

        var args = $"--listen {ListenAddress} --rpc-url {RpcUrl} --pay-address {PayAddress} --refresh-ms {RefreshMs}";
        if (!string.IsNullOrWhiteSpace(ExtraDataHex)) args += $" --extra-data-hex {ExtraDataHex}";
        if (AllowNonDaa) args += " --allow-non-daa";

        ConsoleLog("bridge", $"Starting: {args}");
        SetBusy("Starting bridge...");

        try
        {
            var psi = new ProcessStartInfo { FileName = BridgePath, Arguments = args, RedirectStandardOutput = true, RedirectStandardError = true, UseShellExecute = false, CreateNoWindow = true };
            _bridgeProcess = Process.Start(psi);
            if (_bridgeProcess == null) { ShowToast("Failed to start bridge.", ToastKind.Error); return; }
            _bridgeStartTime = DateTime.Now;
            _bridgeLastStableTime = DateTime.Now;
            BridgeRunning = true; BridgeStatus = "Running"; BridgePillBg = "#16a34a";
            ShowToast("Bridge started.", ToastKind.Ok);
            _ = Task.Run(() => PumpOutput(_bridgeProcess, "bridge"));

            if (StartMinerWithBridge)
            {
                ConsoleLog("system", "Waiting for bridge port...");
                await Task.Delay(2000);
                await StartMiner();
            }
        }
        catch (Exception ex) { ShowToast($"Bridge error: {ex.Message}", ToastKind.Error); ConsoleLog("error", ex.Message); }
        finally { ClearBusy(); }
    }

    [RelayCommand]
    private void StopBridge()
    {
        if (_bridgeProcess == null || _bridgeProcess.HasExited) return;
        ConsoleLog("bridge", "Stopping...");
        try { _bridgeProcess.Kill(entireProcessTree: true); } catch { }
        BridgeRunning = false; BridgeStatus = "Stopped"; BridgePillBg = "#94a3b8";
        _bridgeStartTime = null; _bridgeRestartCount = 0;
        ShowToast("Bridge stopped.", ToastKind.Info);
        StopMiner();
    }

    // ═══════════════════════════════════
    //  MINER
    // ═══════════════════════════════════

    [RelayCommand]
    private async Task StartMiner()
    {
        if (MinerRunning) { ShowToast("Miner already running.", ToastKind.Warn); return; }
        if (string.IsNullOrWhiteSpace(XmrigPath)) { ShowToast("Set XMRig binary path.", ToastKind.Error); return; }
        if (!System.IO.File.Exists(XmrigPath)) { ShowToast("XMRig binary not found.", ToastKind.Error); return; }

        var args = $"-o {XmrigUrl} -u {PayAddress} -p x";
        if (!string.IsNullOrWhiteSpace(XmrigExtra)) args += $" {XmrigExtra}";

        ConsoleLog("miner", $"Starting: {args}");

        try
        {
            var psi = new ProcessStartInfo { FileName = XmrigPath, Arguments = args, RedirectStandardOutput = true, RedirectStandardError = true, UseShellExecute = false, CreateNoWindow = true };
            _minerProcess = Process.Start(psi);
            if (_minerProcess == null) { ShowToast("Failed to start miner.", ToastKind.Error); return; }
            _minerStartTime = DateTime.Now;
            _minerLastStableTime = DateTime.Now;
            MinerRunning = true; MinerStatus = "Running"; MinerPillBg = "#16a34a";
            Hashrate10S = "—"; Hashrate60S = "—"; Hashrate15M = "—"; Accepted = "0"; Rejected = "0"; PoolLatency = "—";
            ShowToast("Miner started.", ToastKind.Ok);
            _ = Task.Run(() => PumpOutput(_minerProcess, "miner"));
        }
        catch (Exception ex) { ShowToast($"Miner error: {ex.Message}", ToastKind.Error); ConsoleLog("error", ex.Message); }
    }

    [RelayCommand]
    private void StopMiner()
    {
        if (_minerProcess == null || _minerProcess.HasExited) return;
        ConsoleLog("miner", "Stopping...");
        try { _minerProcess.Kill(entireProcessTree: true); } catch { }
        MinerRunning = false; MinerStatus = "Stopped"; MinerPillBg = "#94a3b8";
        _minerStartTime = null; _minerRestartCount = 0;
        ShowToast("Miner stopped.", ToastKind.Info);
    }

    // ═══════════════════════════════════
    //  OUTPUT PUMP + AUTO-RESTART
    // ═══════════════════════════════════

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
                            if (!string.IsNullOrEmpty(m.Hashrate10S)) Hashrate10S = $"{m.Hashrate10S} H/s";
                            if (!string.IsNullOrEmpty(m.Hashrate60S)) Hashrate60S = $"{m.Hashrate60S} H/s";
                            if (!string.IsNullOrEmpty(m.Hashrate15M)) Hashrate15M = $"{m.Hashrate15M} H/s";
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
            ConsoleLog("system", $"{tag} exited.");
            if (tag == "bridge")
            {
                BridgeRunning = false; BridgeStatus = "Stopped"; BridgePillBg = "#94a3b8"; _bridgeStartTime = null;
                if (AutoRestartBridge) TryAutoRestart(tag);
            }
            else
            {
                MinerRunning = false; MinerStatus = "Stopped"; MinerPillBg = "#94a3b8"; _minerStartTime = null;
                if (AutoRestartMiner) TryAutoRestart(tag);
            }
        });
    }

    private async void TryAutoRestart(string tag)
    {
        var isBridge = tag == "bridge";
        var count = isBridge ? _bridgeRestartCount : _minerRestartCount;
        var lastStable = isBridge ? _bridgeLastStableTime : _minerLastStableTime;

        if (lastStable.HasValue && (DateTime.Now - lastStable.Value).TotalSeconds > RESTART_RESET_SEC)
            count = 0;

        if (count >= MAX_RESTARTS)
        {
            ShowToast($"{tag} crashed {MAX_RESTARTS}x. Auto-restart disabled.", ToastKind.Error);
            ConsoleLog("system", $"{tag} exceeded max restarts.");
            return;
        }

        count++;
        if (isBridge) _bridgeRestartCount = count; else _minerRestartCount = count;

        var backoff = Math.Min(60, (int)Math.Pow(2, count - 1));
        ConsoleLog("system", $"{tag} auto-restart {count}/{MAX_RESTARTS} in {backoff}s...");
        ShowToast($"{tag} restarting in {backoff}s...", ToastKind.Warn);

        await Task.Delay(backoff * 1000);

        if (isBridge) { await StartBridge(); _bridgeLastStableTime = DateTime.Now; }
        else { await StartMiner(); _minerLastStableTime = DateTime.Now; }
    }

    // ═══════════════════════════════════
    //  HEALTH CHECKS
    // ═══════════════════════════════════

    [RelayCommand]
    private async Task TestRpc()
    {
        SetBusy("Testing RPC...");
        try
        {
            var uri = new Uri(RpcUrl.Replace("grpc://", "http://").Replace("grpcs://", "https://"));
            using var client = new System.Net.Sockets.TcpClient();
            var connectTask = client.ConnectAsync(uri.Host, uri.Port);
            if (await Task.WhenAny(connectTask, Task.Delay(5000)) == connectTask)
            {
                await connectTask;
                ShowToast($"RPC reachable at {uri.Host}:{uri.Port}", ToastKind.Ok);
                ConsoleLog("system", $"RPC OK: {uri.Host}:{uri.Port}");
            }
            else
            {
                ShowToast("RPC connection timed out.", ToastKind.Error);
                ConsoleLog("system", "RPC timeout.");
            }
        }
        catch (Exception ex) { ShowToast($"RPC error: {ex.Message}", ToastKind.Error); }
        finally { ClearBusy(); }
    }

    [RelayCommand]
    private async Task TestStratum()
    {
        SetBusy("Testing Stratum...");
        try
        {
            var parts = ListenAddress.Split(':');
            var host = parts[0] == "0.0.0.0" ? "127.0.0.1" : parts[0];
            var port = int.Parse(parts[1]);
            using var client = new System.Net.Sockets.TcpClient();
            var connectTask = client.ConnectAsync(host, port);
            if (await Task.WhenAny(connectTask, Task.Delay(5000)) == connectTask)
            {
                await connectTask;
                ShowToast($"Stratum reachable at {host}:{port}", ToastKind.Ok);
                ConsoleLog("system", $"Stratum OK: {host}:{port}");
            }
            else
            {
                ShowToast("Stratum not reachable.", ToastKind.Error);
            }
        }
        catch (Exception ex) { ShowToast($"Stratum error: {ex.Message}", ToastKind.Error); }
        finally { ClearBusy(); }
    }

    // ═══════════════════════════════════
    //  METRICS TIMER
    // ═══════════════════════════════════

    private async Task MetricsTimerLoop()
    {
        while (true)
        {
            await Task.Delay(1000);
            Avalonia.Threading.Dispatcher.UIThread.Post(() =>
            {
                BridgeUptime = _bridgeStartTime.HasValue ? (DateTime.Now - _bridgeStartTime.Value).ToString(@"hh\:mm\:ss") : "—";
                MinerUptime = _minerStartTime.HasValue ? (DateTime.Now - _minerStartTime.Value).ToString(@"hh\:mm\:ss") : "—";
            });
        }
    }
}
