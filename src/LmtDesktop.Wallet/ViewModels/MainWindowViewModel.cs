using System;
using System.Collections.ObjectModel;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using LmtDesktop.Core.Helpers;
using LmtDesktop.Core.Models;
using LmtDesktop.Core.Services;

namespace LmtDesktop.Wallet.ViewModels;

public partial class MainWindowViewModel : ObservableObject
{
    private readonly ICliBridge _cli = new CliBridgeService();
    private readonly ConfigService _configService = new();
    private AppConfig _config;
    private WalletProfile _profile;
    private CancellationTokenSource? _pollCts;

    // ── App screens: setup → firstrun → main ──
    [ObservableProperty] private bool _showSetup = true;     // binary selector
    [ObservableProperty] private bool _showFirstRun;          // wallet wizard
    [ObservableProperty] private bool _showMain;              // tabbed interface

    // ── State ──
    [ObservableProperty] private string _pillText = "NO CLI";
    [ObservableProperty] private string _pillBg = "#94a3b8";
    [ObservableProperty] private string _statusText = "Select CLI binary to get started";
    [ObservableProperty] private string _nodeStatusText = "Disconnected";
    [ObservableProperty] private bool _walletOpen;

    // ── Setup screen ──
    [ObservableProperty] private string _cliPath = "";
    [ObservableProperty] private string _setupError = "";
    [ObservableProperty] private string _setupStatus = "";
    [ObservableProperty] private bool _setupBusy;

    // ── Config tab ──
    [ObservableProperty] private string _selectedNetwork = "mainnet";
    [ObservableProperty] private string _walletName = "";

    // ── Collections ──
    public ObservableCollection<string> OutputLines { get; } = new();
    public ObservableCollection<HistoryEntry> HistoryEntries { get; } = new();
    public ObservableCollection<TxRow> Transactions { get; } = new();

    // ── Node info ──
    [ObservableProperty] private string _daaScore = "—";
    [ObservableProperty] private string _peers = "—";
    [ObservableProperty] private string _tipHash = "—";
    [ObservableProperty] private string _difficulty = "—";
    [ObservableProperty] private string _networkName = "—";
    [ObservableProperty] private string _latency = "—";

    // ── Wizard ──
    [ObservableProperty] private int _wizardStep;
    [ObservableProperty] private string _wizardFlow = "";
    [ObservableProperty] private string _newWalletName = "";
    [ObservableProperty] private string _newPassword = "";
    [ObservableProperty] private string _confirmPassword = "";
    [ObservableProperty] private string _mnemonicDisplay = "";
    [ObservableProperty] private string _importMnemonic = "";
    [ObservableProperty] private string _verifyWord1 = "";
    [ObservableProperty] private string _verifyWord2 = "";
    [ObservableProperty] private string _verifyWord3 = "";
    [ObservableProperty] private int _verifyIdx1;
    [ObservableProperty] private int _verifyIdx2;
    [ObservableProperty] private int _verifyIdx3;
    [ObservableProperty] private string _wizardError = "";
    [ObservableProperty] private bool _wizardBusy;

    private string[] _mnemonicArray = Array.Empty<string>();
    private string? _cliBinary;

    // ── Computed ──
    public bool IsStep0 => WizardStep == 0;
    public bool IsStep1 => WizardStep == 1;
    public bool IsStep2 => WizardStep == 2;
    public bool IsStep3 => WizardStep == 3;
    public bool IsImportFlow => WizardFlow == "import";
    public string VerifyLabel1 => $"Word #{VerifyIdx1 + 1}:";
    public string VerifyLabel2 => $"Word #{VerifyIdx2 + 1}:";
    public string VerifyLabel3 => $"Word #{VerifyIdx3 + 1}:";

    partial void OnWizardStepChanged(int v) { OnPropertyChanged(nameof(IsStep0)); OnPropertyChanged(nameof(IsStep1)); OnPropertyChanged(nameof(IsStep2)); OnPropertyChanged(nameof(IsStep3)); }
    partial void OnWizardFlowChanged(string v) => OnPropertyChanged(nameof(IsImportFlow));
    partial void OnVerifyIdx1Changed(int v) => OnPropertyChanged(nameof(VerifyLabel1));
    partial void OnVerifyIdx2Changed(int v) => OnPropertyChanged(nameof(VerifyLabel2));
    partial void OnVerifyIdx3Changed(int v) => OnPropertyChanged(nameof(VerifyLabel3));

    public MainWindowViewModel()
    {
        _config = _configService.Load();
        _profile = _configService.ActiveProfile(_config);
        _cliBinary = _configService.ResolveCliBinary(_config);
        _cliPath = _profile.CliPath;
        _selectedNetwork = _profile.Network;
        _walletName = _profile.LastWallet;

        if (_cliBinary != null && File.Exists(_cliBinary))
        {
            // CLI already configured — skip setup
            ShowSetup = false;
            if (_configService.HasAnyWallet(_config))
                GoToMain();
            else
                GoToFirstRun();
        }
    }

    private void GoToMain()
    {
        ShowSetup = false; ShowFirstRun = false; ShowMain = true;
        UpdatePill();
        StartNodePolling();
    }

    private void GoToFirstRun()
    {
        ShowSetup = false; ShowFirstRun = true; ShowMain = false;
        WizardStep = 0; WizardFlow = "";
    }

    private void UpdatePill()
    {
        if (_cliBinary == null) { PillText = "NO CLI"; PillBg = "#94a3b8"; }
        else if (WalletOpen) { PillText = "WALLET OPEN"; PillBg = "#16a34a"; }
        else { PillText = "READY"; PillBg = "#2563eb"; }
        StatusText = WalletOpen ? $"Wallet: {WalletName}" : (_cliBinary != null ? "Ready" : "CLI not found");
    }

    private void Log(string text)
    {
        var ts = DateTime.Now.ToString("HH:mm:ss");
        OutputLines.Add($"[{ts}] {text}");
    }

    private void AddHistory(ActionCategory cat, string desc, StatusType status, string? detail = null)
    {
        HistoryEntries.Insert(0, new HistoryEntry(DateTime.Now.ToString("HH:mm:ss"), cat, desc, status, detail));
        if (HistoryEntries.Count > 200) HistoryEntries.RemoveAt(200);
    }

    // ═══════════════════════════════════
    //  SETUP — Binary Selection
    // ═══════════════════════════════════

    [RelayCommand]
    private async Task ValidateAndContinue()
    {
        SetupError = ""; SetupStatus = "";
        var path = CliPath.Trim();

        if (string.IsNullOrEmpty(path))
        { SetupError = "Please enter or browse for the lmt-cli binary path."; return; }

        if (!File.Exists(path))
        { SetupError = $"File not found: {path}"; return; }

        SetupBusy = true;
        SetupStatus = "Validating binary...";

        try
        {
            var result = await _cli.RunCaptureAsync(path, new[] { "--version" }, 10);
            if (result.ExitCode != 0)
            { SetupError = "Binary did not respond to --version. Is this the correct lmt-cli?"; return; }

            SetupStatus = $"Verified: {AnsiStripper.Strip(result.Output).Trim()}";
            _cliBinary = path;
            _profile.CliPath = path;
            _configService.Save(_config);

            // Select network
            await _cli.RunCaptureAsync(_cliBinary, new[] { "network", _selectedNetwork }, 10);

            await Task.Delay(500); // brief pause to show status

            if (_configService.HasAnyWallet(_config))
                GoToMain();
            else
                GoToFirstRun();
        }
        catch (Exception ex)
        {
            SetupError = $"Error: {ex.Message}";
        }
        finally
        {
            SetupBusy = false;
        }
    }

    [RelayCommand]
    private void NetworkChanged()
    {
        _profile.Network = SelectedNetwork;
        _configService.Save(_config);
        if (ShowMain)
        {
            Log($"Network set to {SelectedNetwork}");
            AddHistory(ActionCategory.Network, $"Network: {SelectedNetwork}", StatusType.Ok);
            // Apply network change via CLI
            if (_cliBinary != null)
                _ = _cli.RunCaptureAsync(_cliBinary, new[] { "network", SelectedNetwork }, 10);
        }
    }

    // ═══════════════════════════════════
    //  WIZARD — Create / Import
    // ═══════════════════════════════════

    [RelayCommand]
    private void StartCreateWallet() { ResetWizardFields(); WizardFlow = "create"; WizardStep = 1; }

    [RelayCommand]
    private void StartImportWallet() { ResetWizardFields(); WizardFlow = "import"; WizardStep = 1; }

    [RelayCommand]
    private void WizardBack()
    {
        WizardError = "";
        if (WizardStep <= 1) { WizardStep = 0; WizardFlow = ""; }
        else WizardStep--;
    }

    [RelayCommand]
    private async Task WizardNext()
    {
        WizardError = "";
        if (WizardFlow == "create") await HandleCreateFlow();
        else if (WizardFlow == "import") await HandleImportFlow();
    }

    private async Task HandleCreateFlow()
    {
        switch (WizardStep)
        {
            case 1:
                if (string.IsNullOrWhiteSpace(NewWalletName))
                { WizardError = "Wallet name is required."; return; }
                if ((NewPassword ?? "").Length < 8)
                { WizardError = "Password must be at least 8 characters."; return; }
                if (NewPassword != ConfirmPassword)
                { WizardError = "Passwords do not match."; return; }

                WizardBusy = true;
                try
                {
                    // Create wallet via CLI stdin/stdout automation
                    var mnemonic = await CreateWalletViaCliAsync(NewWalletName.Trim(), NewPassword);
                    if (mnemonic == null)
                    {
                        WizardError = "Failed to create wallet. Check that lmt-cli is working.";
                        return;
                    }
                    _mnemonicArray = mnemonic;
                    MnemonicDisplay = string.Join("  ", _mnemonicArray.Select((w, i) => $"{i + 1}. {w}"));

                    var rng = new Random();
                    var indices = Enumerable.Range(0, _mnemonicArray.Length).OrderBy(_ => rng.Next()).Take(3).OrderBy(x => x).ToArray();
                    VerifyIdx1 = indices[0]; VerifyIdx2 = indices[1]; VerifyIdx3 = indices[2];
                    VerifyWord1 = ""; VerifyWord2 = ""; VerifyWord3 = "";
                    WizardStep = 2;
                }
                finally { WizardBusy = false; }
                return;

            case 2:
                WizardStep = 3;
                return;

            case 3:
                if (!string.Equals((VerifyWord1 ?? "").Trim(), _mnemonicArray[VerifyIdx1], StringComparison.OrdinalIgnoreCase) ||
                    !string.Equals((VerifyWord2 ?? "").Trim(), _mnemonicArray[VerifyIdx2], StringComparison.OrdinalIgnoreCase) ||
                    !string.Equals((VerifyWord3 ?? "").Trim(), _mnemonicArray[VerifyIdx3], StringComparison.OrdinalIgnoreCase))
                {
                    WizardError = "One or more words are incorrect. Check your backup.";
                    return;
                }
                FinishWizard();
                return;
        }
    }

    private async Task HandleImportFlow()
    {
        if (WizardStep != 1) return;

        if (string.IsNullOrWhiteSpace(NewWalletName))
        { WizardError = "Wallet name is required."; return; }
        var words = (ImportMnemonic ?? "").Trim().Split(' ', StringSplitOptions.RemoveEmptyEntries);
        if (words.Length != 12 && words.Length != 24)
        { WizardError = $"Mnemonic must be 12 or 24 words (got {words.Length})."; return; }
        if ((NewPassword ?? "").Length < 8)
        { WizardError = "Password must be at least 8 characters."; return; }
        if (NewPassword != ConfirmPassword)
        { WizardError = "Passwords do not match."; return; }

        WizardBusy = true;
        try
        {
            var success = await ImportWalletViaCliAsync(NewWalletName.Trim(), NewPassword, string.Join(" ", words));
            if (!success)
            { WizardError = "Import failed. Check your mnemonic and try again."; return; }
            FinishWizard();
        }
        finally { WizardBusy = false; }
    }

    /// <summary>
    /// Automate wallet creation via lmt-cli stdin/stdout.
    /// Returns the 12-word mnemonic or null on failure.
    /// </summary>
    private async Task<string[]?> CreateWalletViaCliAsync(string name, string password)
    {
        if (_cliBinary == null) return null;
        try
        {
            var psi = new ProcessStartInfo
            {
                FileName = _cliBinary,
                RedirectStandardInput = true,
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                UseShellExecute = false,
                CreateNoWindow = true,
            };
            psi.ArgumentList.Add("wallet");
            psi.ArgumentList.Add("create");
            psi.ArgumentList.Add(name);

            using var proc = new Process { StartInfo = psi };
            proc.Start();

            var writer = proc.StandardInput;
            var output = new System.Text.StringBuilder();

            // Read output async while sending inputs
            var readTask = Task.Run(async () =>
            {
                while (!proc.HasExited)
                {
                    var line = await proc.StandardOutput.ReadLineAsync();
                    if (line == null) break;
                    output.AppendLine(line);
                }
            });

            // The CLI prompts: wallet encryption password (twice), optional BIP39 passphrase, phishing hint
            await Task.Delay(500);
            await writer.WriteLineAsync(password);      // encryption password
            await Task.Delay(300);
            await writer.WriteLineAsync(password);      // confirm password
            await Task.Delay(300);
            await writer.WriteLineAsync("");             // skip phishing hint
            await Task.Delay(300);
            await writer.WriteLineAsync("");             // skip BIP39 passphrase

            // Wait for process to complete
            using var cts = new CancellationTokenSource(TimeSpan.FromSeconds(30));
            try { await proc.WaitForExitAsync(cts.Token); }
            catch { proc.Kill(); return null; }

            await readTask;

            // Parse mnemonic from output — look for sequence of BIP39 words
            var text = AnsiStripper.Strip(output.ToString());
            return ExtractMnemonicFromOutput(text);
        }
        catch (Exception ex)
        {
            Log($"Wallet creation error: {ex.Message}");
            return null;
        }
    }

    /// <summary>
    /// Automate wallet import via lmt-cli stdin/stdout.
    /// </summary>
    private async Task<bool> ImportWalletViaCliAsync(string name, string password, string mnemonic)
    {
        if (_cliBinary == null) return false;
        try
        {
            var psi = new ProcessStartInfo
            {
                FileName = _cliBinary,
                RedirectStandardInput = true,
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                UseShellExecute = false,
                CreateNoWindow = true,
            };
            psi.ArgumentList.Add("wallet");
            psi.ArgumentList.Add("import");
            psi.ArgumentList.Add(name);

            using var proc = new Process { StartInfo = psi };
            proc.Start();

            var writer = proc.StandardInput;

            await Task.Delay(500);
            await writer.WriteLineAsync(password);      // encryption password
            await Task.Delay(300);
            await writer.WriteLineAsync(password);      // confirm password
            await Task.Delay(300);
            await writer.WriteLineAsync("");             // skip phishing hint
            await Task.Delay(300);
            await writer.WriteLineAsync("");             // skip BIP39 passphrase (import)
            await Task.Delay(300);
            await writer.WriteLineAsync(mnemonic);       // the mnemonic words

            using var cts = new CancellationTokenSource(TimeSpan.FromSeconds(30));
            try { await proc.WaitForExitAsync(cts.Token); }
            catch { proc.Kill(); return false; }

            return proc.ExitCode == 0;
        }
        catch (Exception ex)
        {
            Log($"Wallet import error: {ex.Message}");
            return false;
        }
    }

    /// <summary>
    /// Extract 12 or 24 BIP39 mnemonic words from CLI output.
    /// Looks for a line containing 12+ lowercase words that match BIP39 patterns.
    /// </summary>
    private static string[]? ExtractMnemonicFromOutput(string output)
    {
        foreach (var line in output.Split('\n'))
        {
            var trimmed = line.Trim();
            var words = trimmed.Split(' ', StringSplitOptions.RemoveEmptyEntries);
            // BIP39 mnemonics are 12 or 24 lowercase alpha words
            if (words.Length is 12 or 24 && words.All(w => w.All(char.IsLetter) && w == w.ToLowerInvariant()))
                return words;
        }
        // Fallback: collect words across multiple lines after "mnemonic" keyword
        var collecting = false;
        var collected = new System.Collections.Generic.List<string>();
        foreach (var line in output.Split('\n'))
        {
            if (line.ToLowerInvariant().Contains("mnemonic"))
            { collecting = true; continue; }
            if (collecting)
            {
                var words = line.Trim().Split(' ', StringSplitOptions.RemoveEmptyEntries);
                foreach (var w in words)
                {
                    var clean = w.Trim().ToLowerInvariant();
                    if (clean.All(char.IsLetter) && clean.Length >= 3)
                        collected.Add(clean);
                }
                if (collected.Count is 12 or 24) return collected.ToArray();
                if (collected.Count > 24) break;
            }
        }
        return null;
    }

    private void FinishWizard()
    {
        WalletName = NewWalletName.Trim();
        WalletOpen = true;
        _profile.LastWallet = WalletName;
        _configService.Save(_config);
        // Clear sensitive data
        NewPassword = ""; ConfirmPassword = "";
        _mnemonicArray = Array.Empty<string>();
        MnemonicDisplay = ""; ImportMnemonic = "";
        Log($"Wallet '{WalletName}' created and opened.");
        AddHistory(ActionCategory.Wallet, $"Wallet '{WalletName}' opened", StatusType.Ok);
        GoToMain();
    }

    private void ResetWizardFields()
    {
        WizardError = "";
        NewWalletName = ""; NewPassword = ""; ConfirmPassword = "";
        MnemonicDisplay = ""; ImportMnemonic = "";
        VerifyWord1 = ""; VerifyWord2 = ""; VerifyWord3 = "";
    }

    // ═══════════════════════════════════
    //  CONFIG
    // ═══════════════════════════════════

    [RelayCommand]
    private async Task SaveCliPath()
    {
        _profile.CliPath = CliPath.Trim();
        _configService.Save(_config);
        _cliBinary = _configService.ResolveCliBinary(_config);
        if (_cliBinary != null)
        {
            var result = await _cli.RunCaptureAsync(_cliBinary, new[] { "--version" }, 5);
            Log(result.ExitCode == 0 ? $"CLI verified: {AnsiStripper.Strip(result.Output).Trim()}" : "CLI binary failed validation.");
        }
        UpdatePill();
        AddHistory(ActionCategory.System, "CLI path updated", _cliBinary != null ? StatusType.Ok : StatusType.Error);
    }

    // ═══════════════════════════════════
    //  ACTIONS — All wired to real CLI
    // ═══════════════════════════════════

    [RelayCommand]
    private async Task RunCliCommand(string args)
    {
        if (_cliBinary == null) { Log("CLI binary not configured."); return; }
        var parts = args.Split(' ', StringSplitOptions.RemoveEmptyEntries);
        Log($"> lmt-cli {args}");
        var result = await _cli.RunCaptureAsync(_cliBinary, parts);
        Log(AnsiStripper.Strip(result.Output));
        AddHistory(ActionCategory.Wallet, args,
            result.ExitCode == 0 ? StatusType.Ok : StatusType.Error,
            AnsiStripper.Strip(result.Output));
        if (result.ExitCode != 0)
        {
            var action = _cli.MapCliErrorAction(result.ExitCode, result.Output);
            if (!string.IsNullOrEmpty(action.Message)) Log($"Error: {action.Message}");
        }
    }

    [RelayCommand]
    private void LaunchInteractive(string args)
    {
        if (_cliBinary == null) { Log("CLI binary not configured."); return; }

        // Ensure network is selected before interactive commands
        _ = _cli.RunCaptureAsync(_cliBinary, new[] { "network", SelectedNetwork }, 5);

        var parts = args.Split(' ', StringSplitOptions.RemoveEmptyEntries);
        var (success, msg) = _cli.LaunchInteractive(_cliBinary, parts);
        Log(success ? $"Launched: {args}" : msg);
        AddHistory(ActionCategory.Wallet, args, success ? StatusType.Pending : StatusType.Error);
    }

    [RelayCommand]
    private async Task LockWallet()
    {
        if (_cliBinary == null) return;
        var result = await _cli.RunCaptureAsync(_cliBinary, new[] { "wallet", "close" });
        WalletOpen = false; WalletName = "";
        UpdatePill();
        Log("Wallet locked.");
        AddHistory(ActionCategory.Wallet, "Wallet locked", StatusType.Ok);
    }

    [RelayCommand]
    private async Task OpenWallet()
    {
        if (_cliBinary == null) return;
        // Launch interactive so user can enter password
        LaunchInteractive("wallet open");
        // Poll to detect when wallet is open
        for (int i = 0; i < 15; i++)
        {
            await Task.Delay(2000);
            var check = await _cli.RunCaptureAsync(_cliBinary, new[] { "list" }, 5);
            if (_cli.IsWalletOpenFromOutput(check.ExitCode, check.Output))
            {
                WalletOpen = true;
                WalletName = _profile.LastWallet;
                UpdatePill();
                Log($"Wallet opened.");
                AddHistory(ActionCategory.Wallet, "Wallet opened", StatusType.Ok);
                return;
            }
        }
    }

    [RelayCommand] private async Task RefreshBalances() => await RunCliCommand("list");
    [RelayCommand] private async Task GetAddress() => await RunCliCommand("address");
    [RelayCommand] private async Task NewAddress() => await RunCliCommand("address new");

    [RelayCommand]
    private async Task RefreshTransactions()
    {
        if (_cliBinary == null) return;
        var result = await _cli.RunCaptureAsync(_cliBinary, new[] { "history", "list", "30" });
        Transactions.Clear();
        foreach (var tx in TxHistoryParser.ParseHistoryOutput(result.ExitCode, result.Output))
            Transactions.Add(tx);
        AddHistory(ActionCategory.Wallet, "Refreshed transactions", StatusType.Ok);
    }

    // ═══════════════════════════════════
    //  NODE POLLING
    // ═══════════════════════════════════

    private void StartNodePolling()
    {
        _pollCts?.Cancel();
        _pollCts = new CancellationTokenSource();
        _ = PollNodeLoop(_pollCts.Token);
    }

    private async Task PollNodeLoop(CancellationToken ct)
    {
        while (!ct.IsCancellationRequested)
        {
            await Task.Delay(12_000, ct).ConfigureAwait(false);
            if (_cliBinary == null) continue;
            try
            {
                var ping = await _cli.RunCaptureTimedAsync(_cliBinary, new[] { "ping" }, 10, ct);
                var connected = ping.ExitCode == 0;
                Latency = connected ? $"{ping.ElapsedMs:F0} ms" : "—";
                if (connected)
                {
                    var dag = await _cli.RunCaptureAsync(_cliBinary, new[] { "rpc", "get_block_dag_info" }, 10, ct);
                    var peer = await _cli.RunCaptureAsync(_cliBinary, new[] { "rpc", "get_connected_peer_info" }, 10, ct);
                    var info = NodeInfoParser.Parse(dag.Output, peer.Output, ping.ElapsedMs);
                    DaaScore = info.DaaScore; Peers = info.Peers;
                    TipHash = info.TipHash.Length > 16 ? info.TipHash[..16] + "..." : info.TipHash;
                    Difficulty = info.Difficulty; NetworkName = info.NetworkName;
                    NodeStatusText = $"Synced | {info.Peers} peers";
                }
                else
                {
                    NodeStatusText = "Disconnected";
                    DaaScore = Peers = TipHash = Difficulty = NetworkName = "—";
                }
            }
            catch (OperationCanceledException) { break; }
            catch { NodeStatusText = "Error"; }
        }
    }
}
