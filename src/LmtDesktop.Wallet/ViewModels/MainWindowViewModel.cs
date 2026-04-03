using System;
using System.Collections.ObjectModel;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using LmtDesktop.Core.Helpers;
using LmtDesktop.Core.Models;
using LmtDesktop.Core.Services;
using LmtDesktop.Core.Validation;

namespace LmtDesktop.Wallet.ViewModels;

public partial class MainWindowViewModel : ViewModelBase
{
    private readonly ICliBridge _cli = new CliBridgeService();
    private readonly ConfigService _configService = new();
    private AppConfig _config;
    private WalletProfile _profile;
    private CancellationTokenSource? _pollCts;
    private DateTime _lastActivity = DateTime.Now;
    private CancellationTokenSource? _sessionCts;

    // ── Screens ──
    [ObservableProperty] private bool _showSetup = true;
    [ObservableProperty] private bool _showFirstRun;
    [ObservableProperty] private bool _showMain;

    // ── State ──
    [ObservableProperty] private string _pillText = "NO CLI";
    [ObservableProperty] private string _pillBg = "#94a3b8";
    [ObservableProperty] private string _backupPillVisible = "False";
    [ObservableProperty] private string _statusText = "Select CLI binary";
    [ObservableProperty] private string _nodeStatusText = "Disconnected";
    [ObservableProperty] private bool _walletOpen;

    // ── Setup ──
    [ObservableProperty] private string _cliPath = "";
    [ObservableProperty] private string _setupError = "";
    [ObservableProperty] private string _setupStatus = "";
    [ObservableProperty] private bool _setupBusy;

    // ── Config ──
    [ObservableProperty] private string _selectedNetwork = "mainnet";
    [ObservableProperty] private string _walletName = "";
    [ObservableProperty] private int _sessionTimeoutMinutes;
    [ObservableProperty] private bool _autoLockOnTimeout = true;

    // ── Collections ──
    public ObservableCollection<string> OutputLines { get; } = new();
    public ObservableCollection<HistoryEntry> HistoryEntries { get; } = new();
    public ObservableCollection<TxRow> Transactions { get; } = new();
    public ObservableCollection<Contact> Contacts { get; } = new();

    // ── Node ──
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
    // Seed backup checklist
    [ObservableProperty] private bool _checkSafePlace;
    [ObservableProperty] private bool _checkShownOnce;
    [ObservableProperty] private bool _checkNoScreenshot;
    public bool AllChecksComplete => CheckSafePlace && CheckShownOnce && CheckNoScreenshot;
    partial void OnCheckSafePlaceChanged(bool v) => OnPropertyChanged(nameof(AllChecksComplete));
    partial void OnCheckShownOnceChanged(bool v) => OnPropertyChanged(nameof(AllChecksComplete));
    partial void OnCheckNoScreenshotChanged(bool v) => OnPropertyChanged(nameof(AllChecksComplete));

    // ── Send dialog ──
    [ObservableProperty] private bool _showSendDialog;
    [ObservableProperty] private bool _showSendConfirm;
    [ObservableProperty] private string _sendAddress = "";
    [ObservableProperty] private string _sendAmount = "";
    [ObservableProperty] private string _sendFee = "0";
    [ObservableProperty] private string _sendError = "";
    [ObservableProperty] private int _selectedContactIndex = -1;
    // Confirmation display
    [ObservableProperty] private string _confirmNetwork = "";
    [ObservableProperty] private string _confirmAmount = "";
    [ObservableProperty] private string _confirmFee = "";
    [ObservableProperty] private string _confirmTotal = "";
    [ObservableProperty] private string _confirmAddress = "";

    // ── Transfer dialog ──
    [ObservableProperty] private bool _showTransferDialog;
    [ObservableProperty] private string _transferAccount = "";
    [ObservableProperty] private string _transferAmount = "";
    [ObservableProperty] private string _transferFee = "0";
    [ObservableProperty] private string _transferError = "";
    public ObservableCollection<string> AccountSuggestions { get; } = new();

    // ── Contacts dialog ──
    [ObservableProperty] private bool _showContactsDialog;
    [ObservableProperty] private string _contactName = "";
    [ObservableProperty] private string _contactAddress = "";
    [ObservableProperty] private string _contactNote = "";
    [ObservableProperty] private string _contactError = "";
    [ObservableProperty] private int _selectedContactEditIndex = -1;

    // ── Error action dialog ──
    [ObservableProperty] private bool _showErrorDialog;
    [ObservableProperty] private string _errorDialogMessage = "";
    [ObservableProperty] private string _errorDialogAction = "";
    [ObservableProperty] private string _errorDialogActionKey = "";

    private string[] _mnemonicArray = Array.Empty<string>();
    private string? _cliBinary;

    // ── Computed ──
    public bool IsStep0 => WizardStep == 0;
    public bool IsStep1 => WizardStep == 1;
    public bool IsStep2 => WizardStep == 2;
    public bool IsStep3 => WizardStep == 3;
    public bool IsStep4 => WizardStep == 4; // checklist (create only)
    public bool IsImportFlow => WizardFlow == "import";
    public string VerifyLabel1 => $"Word #{VerifyIdx1 + 1}:";
    public string VerifyLabel2 => $"Word #{VerifyIdx2 + 1}:";
    public string VerifyLabel3 => $"Word #{VerifyIdx3 + 1}:";

    partial void OnWizardStepChanged(int v) { OnPropertyChanged(nameof(IsStep0)); OnPropertyChanged(nameof(IsStep1)); OnPropertyChanged(nameof(IsStep2)); OnPropertyChanged(nameof(IsStep3)); OnPropertyChanged(nameof(IsStep4)); }
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
        _sessionTimeoutMinutes = _profile.SessionTimeoutMinutes;
        _autoLockOnTimeout = _profile.AutoLockOnTimeout;

        // Load contacts
        foreach (var c in _profile.Contacts)
            Contacts.Add(c);

        if (_cliBinary != null && System.IO.File.Exists(_cliBinary))
        {
            ShowSetup = false;
            if (_configService.HasAnyWallet(_config)) GoToMain();
            else GoToFirstRun();
        }
    }

    private void RecordActivity() => _lastActivity = DateTime.Now;

    private void GoToMain()
    {
        ShowSetup = false; ShowFirstRun = false; ShowMain = true;
        UpdatePill();
        StartNodePolling();
        StartSessionTimer();
    }

    private void GoToFirstRun()
    {
        ShowSetup = false; ShowFirstRun = true; ShowMain = false;
        WizardStep = 0; WizardFlow = "";
    }

    private void UpdatePill()
    {
        if (_cliBinary == null) { PillText = "NO CLI"; PillBg = "#94a3b8"; }
        else if (IsBusy) { PillText = "RUNNING..."; PillBg = "#d97706"; }
        else if (WalletOpen) { PillText = "WALLET OPEN"; PillBg = "#16a34a"; }
        else { PillText = "READY"; PillBg = "#2563eb"; }

        // Backup warning
        if (WalletOpen && !string.IsNullOrEmpty(WalletName))
        {
            var backed = _profile.SeedBackupConfirmed.TryGetValue(WalletName, out var v) && v;
            BackupPillVisible = backed ? "False" : "True";
        }
        else BackupPillVisible = "False";

        StatusText = WalletOpen ? $"Wallet: {WalletName}" : (_cliBinary != null ? "Ready" : "CLI not found");
    }

    private void Log(string text)
    {
        OutputLines.Add($"[{DateTime.Now:HH:mm:ss}] {text}");
    }

    private void AddHistory(ActionCategory cat, string desc, StatusType status, string? detail = null)
    {
        HistoryEntries.Insert(0, new HistoryEntry(DateTime.Now.ToString("HH:mm:ss"), cat, desc, status, detail));
        if (HistoryEntries.Count > 200) HistoryEntries.RemoveAt(200);
    }

    // ═══════════════════════════════════
    //  SETUP
    // ═══════════════════════════════════

    [RelayCommand]
    private async Task ValidateAndContinue()
    {
        SetupError = ""; SetupStatus = "";
        var path = CliPath.Trim();
        if (string.IsNullOrEmpty(path)) { SetupError = "Please enter or browse for the lmt-cli binary."; return; }
        if (!System.IO.File.Exists(path)) { SetupError = $"File not found: {path}"; return; }

        SetupBusy = true; SetupStatus = "Validating...";
        try
        {
            var result = await _cli.RunCaptureAsync(path, new[] { "--version" }, 10);
            if (result.ExitCode != 0) { SetupError = "Binary did not respond. Is this lmt-cli?"; return; }
            SetupStatus = $"OK: {AnsiStripper.Strip(result.Output).Trim()}";
            _cliBinary = path;
            _profile.CliPath = path;
            _configService.Save(_config);
            await _cli.RunCaptureAsync(_cliBinary, new[] { "network", _selectedNetwork }, 10);
            await Task.Delay(400);
            if (_configService.HasAnyWallet(_config)) GoToMain(); else GoToFirstRun();
        }
        catch (Exception ex) { SetupError = $"Error: {ex.Message}"; }
        finally { SetupBusy = false; }
    }

    [RelayCommand]
    private void NetworkChanged()
    {
        _profile.Network = SelectedNetwork;
        _configService.Save(_config);
        if (ShowMain) { Log($"Network: {SelectedNetwork}"); ShowToast($"Network: {SelectedNetwork}", ToastKind.Info); }
        if (_cliBinary != null) _ = _cli.RunCaptureAsync(_cliBinary, new[] { "network", SelectedNetwork }, 10);
    }

    // ═══════════════════════════════════
    //  WIZARD
    // ═══════════════════════════════════

    [RelayCommand] private void StartCreateWallet() { ResetWizardFields(); WizardFlow = "create"; WizardStep = 4; /* checklist first */ }
    [RelayCommand] private void StartImportWallet() { ResetWizardFields(); WizardFlow = "import"; WizardStep = 1; }

    [RelayCommand]
    private void WizardBack()
    {
        WizardError = "";
        if (WizardStep == 4) { WizardStep = 0; WizardFlow = ""; } // checklist back to welcome
        else if (WizardStep <= 1) { WizardStep = 0; WizardFlow = ""; }
        else WizardStep--;
    }

    [RelayCommand]
    private void ChecklistProceed()
    {
        if (!AllChecksComplete) return;
        WizardStep = 1; // go to name+password
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
                if (string.IsNullOrWhiteSpace(NewWalletName)) { WizardError = "Wallet name is required."; return; }
                if ((NewPassword ?? "").Length < 8) { WizardError = "Password must be at least 8 characters."; return; }
                if (NewPassword != ConfirmPassword) { WizardError = "Passwords do not match."; return; }
                WizardBusy = true;
                try
                {
                    var mnemonic = await CreateWalletViaCliAsync(NewWalletName.Trim(), NewPassword);
                    if (mnemonic == null) { WizardError = "Wallet creation failed. Check lmt-cli."; return; }
                    _mnemonicArray = mnemonic;
                    MnemonicDisplay = string.Join("  ", _mnemonicArray.Select((w, i) => $"{i + 1}. {w}"));
                    var rng = new Random();
                    var idx = Enumerable.Range(0, _mnemonicArray.Length).OrderBy(_ => rng.Next()).Take(3).OrderBy(x => x).ToArray();
                    VerifyIdx1 = idx[0]; VerifyIdx2 = idx[1]; VerifyIdx3 = idx[2];
                    VerifyWord1 = ""; VerifyWord2 = ""; VerifyWord3 = "";
                    WizardStep = 2;
                }
                finally { WizardBusy = false; }
                return;
            case 2: WizardStep = 3; return;
            case 3:
                if (!string.Equals((VerifyWord1 ?? "").Trim(), _mnemonicArray[VerifyIdx1], StringComparison.OrdinalIgnoreCase) ||
                    !string.Equals((VerifyWord2 ?? "").Trim(), _mnemonicArray[VerifyIdx2], StringComparison.OrdinalIgnoreCase) ||
                    !string.Equals((VerifyWord3 ?? "").Trim(), _mnemonicArray[VerifyIdx3], StringComparison.OrdinalIgnoreCase))
                { WizardError = "Words incorrect. Check your backup."; return; }
                _profile.SeedBackupConfirmed[NewWalletName.Trim()] = true;
                FinishWizard();
                return;
        }
    }

    private async Task HandleImportFlow()
    {
        if (WizardStep != 1) return;
        if (string.IsNullOrWhiteSpace(NewWalletName)) { WizardError = "Wallet name is required."; return; }
        var words = (ImportMnemonic ?? "").Trim().Split(' ', StringSplitOptions.RemoveEmptyEntries);
        if (words.Length != 12 && words.Length != 24) { WizardError = $"Need 12 or 24 words (got {words.Length})."; return; }
        if ((NewPassword ?? "").Length < 8) { WizardError = "Password must be at least 8 characters."; return; }
        if (NewPassword != ConfirmPassword) { WizardError = "Passwords do not match."; return; }
        WizardBusy = true;
        try
        {
            var ok = await ImportWalletViaCliAsync(NewWalletName.Trim(), NewPassword, string.Join(" ", words));
            if (!ok) { WizardError = "Import failed. Check mnemonic."; return; }
            FinishWizard();
        }
        finally { WizardBusy = false; }
    }

    private async Task<string[]?> CreateWalletViaCliAsync(string name, string password)
    {
        if (_cliBinary == null) return null;
        try
        {
            var psi = new System.Diagnostics.ProcessStartInfo { FileName = _cliBinary, RedirectStandardInput = true, RedirectStandardOutput = true, RedirectStandardError = true, UseShellExecute = false, CreateNoWindow = true };
            psi.ArgumentList.Add("wallet"); psi.ArgumentList.Add("create"); psi.ArgumentList.Add(name);
            using var proc = new System.Diagnostics.Process { StartInfo = psi };
            proc.Start();
            var writer = proc.StandardInput;
            var sb = new System.Text.StringBuilder();
            var readTask = Task.Run(async () => { while (!proc.HasExited) { var l = await proc.StandardOutput.ReadLineAsync(); if (l == null) break; sb.AppendLine(l); } });
            await Task.Delay(500); await writer.WriteLineAsync(password);
            await Task.Delay(300); await writer.WriteLineAsync(password);
            await Task.Delay(300); await writer.WriteLineAsync(""); // phishing
            await Task.Delay(300); await writer.WriteLineAsync(""); // bip39
            using var cts = new CancellationTokenSource(TimeSpan.FromSeconds(30));
            try { await proc.WaitForExitAsync(cts.Token); } catch { proc.Kill(); return null; }
            await readTask;
            return ExtractMnemonic(AnsiStripper.Strip(sb.ToString()));
        }
        catch (Exception ex) { Log($"Create error: {ex.Message}"); return null; }
    }

    private async Task<bool> ImportWalletViaCliAsync(string name, string password, string mnemonic)
    {
        if (_cliBinary == null) return false;
        try
        {
            var psi = new System.Diagnostics.ProcessStartInfo { FileName = _cliBinary, RedirectStandardInput = true, RedirectStandardOutput = true, RedirectStandardError = true, UseShellExecute = false, CreateNoWindow = true };
            psi.ArgumentList.Add("wallet"); psi.ArgumentList.Add("import"); psi.ArgumentList.Add(name);
            using var proc = new System.Diagnostics.Process { StartInfo = psi };
            proc.Start();
            var writer = proc.StandardInput;
            await Task.Delay(500); await writer.WriteLineAsync(password);
            await Task.Delay(300); await writer.WriteLineAsync(password);
            await Task.Delay(300); await writer.WriteLineAsync("");
            await Task.Delay(300); await writer.WriteLineAsync("");
            await Task.Delay(300); await writer.WriteLineAsync(mnemonic);
            using var cts = new CancellationTokenSource(TimeSpan.FromSeconds(30));
            try { await proc.WaitForExitAsync(cts.Token); } catch { proc.Kill(); return false; }
            return proc.ExitCode == 0;
        }
        catch (Exception ex) { Log($"Import error: {ex.Message}"); return false; }
    }

    private static string[]? ExtractMnemonic(string output)
    {
        foreach (var line in output.Split('\n'))
        {
            var words = line.Trim().Split(' ', StringSplitOptions.RemoveEmptyEntries);
            if (words.Length is 12 or 24 && words.All(w => w.All(char.IsLetter) && w == w.ToLowerInvariant()))
                return words;
        }
        var collecting = false;
        var collected = new System.Collections.Generic.List<string>();
        foreach (var line in output.Split('\n'))
        {
            if (line.ToLowerInvariant().Contains("mnemonic")) { collecting = true; continue; }
            if (collecting)
            {
                foreach (var w in line.Trim().Split(' ', StringSplitOptions.RemoveEmptyEntries))
                {
                    var c = w.Trim().ToLowerInvariant();
                    if (c.All(char.IsLetter) && c.Length >= 3) collected.Add(c);
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
        NewPassword = ""; ConfirmPassword = ""; _mnemonicArray = Array.Empty<string>(); MnemonicDisplay = ""; ImportMnemonic = "";
        ShowToast($"Wallet '{WalletName}' created!", ToastKind.Ok);
        Log($"Wallet '{WalletName}' opened.");
        AddHistory(ActionCategory.Wallet, $"Wallet '{WalletName}' opened", StatusType.Ok);
        GoToMain();
    }

    private void ResetWizardFields()
    {
        WizardError = ""; NewWalletName = ""; NewPassword = ""; ConfirmPassword = "";
        MnemonicDisplay = ""; ImportMnemonic = "";
        VerifyWord1 = ""; VerifyWord2 = ""; VerifyWord3 = "";
        CheckSafePlace = false; CheckShownOnce = false; CheckNoScreenshot = false;
    }

    // ═══════════════════════════════════
    //  SEND DIALOG
    // ═══════════════════════════════════

    [RelayCommand]
    private void OpenSendDialog()
    {
        RecordActivity();
        if (!WalletOpen) { ShowToast("Open a wallet first.", ToastKind.Warn); return; }
        SendAddress = ""; SendAmount = ""; SendFee = "0"; SendError = ""; SelectedContactIndex = -1;
        ShowSendDialog = true; ShowSendConfirm = false;
    }

    [RelayCommand]
    private void CloseSendDialog() { ShowSendDialog = false; ShowSendConfirm = false; }

    [RelayCommand]
    private void FillFromContact()
    {
        if (SelectedContactIndex >= 0 && SelectedContactIndex < Contacts.Count)
            SendAddress = Contacts[SelectedContactIndex].Address;
    }

    [RelayCommand]
    private void SubmitSend()
    {
        SendError = "";
        var addrResult = AddressValidator.Validate(SendAddress, SelectedNetwork);
        if (!addrResult.Valid) { SendError = addrResult.Error ?? "Invalid address"; return; }
        var amount = AmountValidator.ParsePositiveAmount(SendAmount);
        if (amount == null) { SendError = "Amount must be greater than 0."; return; }
        var fee = AmountValidator.ParseNonnegativeFee(SendFee);
        if (fee == null) { SendError = "Fee must be 0 or more."; return; }

        ConfirmNetwork = SelectedNetwork;
        ConfirmAmount = $"{amount:F8} LMT";
        ConfirmFee = $"{fee:F8} LMT";
        ConfirmTotal = $"{amount + fee:F8} LMT";
        ConfirmAddress = SendAddress.Trim();
        ShowSendConfirm = true;
    }

    [RelayCommand]
    private void ConfirmSend()
    {
        ShowSendDialog = false; ShowSendConfirm = false;
        LaunchInteractive($"send {SendAddress.Trim()} {SendAmount.Trim()} {SendFee.Trim()}");
        ShowToast("Send launched in terminal.", ToastKind.Info);
        AddHistory(ActionCategory.Send, $"Send {SendAmount} LMT", StatusType.Pending);
    }

    // ═══════════════════════════════════
    //  TRANSFER DIALOG
    // ═══════════════════════════════════

    [RelayCommand]
    private async Task OpenTransferDialog()
    {
        RecordActivity();
        if (!WalletOpen) { ShowToast("Open a wallet first.", ToastKind.Warn); return; }
        TransferAccount = ""; TransferAmount = ""; TransferFee = "0"; TransferError = "";
        AccountSuggestions.Clear();
        ShowTransferDialog = true;
        // Load accounts
        if (_cliBinary != null)
        {
            var r = await _cli.RunCaptureAsync(_cliBinary, new[] { "list" }, 10);
            foreach (var line in AnsiStripper.Strip(r.Output).Split('\n'))
            {
                var trimmed = line.Trim();
                if (trimmed.StartsWith("\u2022") || trimmed.StartsWith("*") || trimmed.StartsWith("-"))
                {
                    var name = trimmed.TrimStart('\u2022', '*', '-', ' ');
                    if (!string.IsNullOrWhiteSpace(name)) AccountSuggestions.Add(name);
                }
            }
        }
    }

    [RelayCommand] private void CloseTransferDialog() => ShowTransferDialog = false;

    [RelayCommand]
    private void SubmitTransfer()
    {
        TransferError = "";
        if (string.IsNullOrWhiteSpace(TransferAccount)) { TransferError = "Account required."; return; }
        var amount = AmountValidator.ParsePositiveAmount(TransferAmount);
        if (amount == null) { TransferError = "Amount must be > 0."; return; }
        ShowTransferDialog = false;
        LaunchInteractive($"transfer {TransferAccount.Trim()} {TransferAmount.Trim()} {TransferFee.Trim()}");
        ShowToast("Transfer launched in terminal.", ToastKind.Info);
        AddHistory(ActionCategory.Transfer, $"Transfer {TransferAmount} LMT", StatusType.Pending);
    }

    // ═══════════════════════════════════
    //  CONTACTS DIALOG
    // ═══════════════════════════════════

    [RelayCommand] private void OpenContactsDialog() { ContactName = ""; ContactAddress = ""; ContactNote = ""; ContactError = ""; SelectedContactEditIndex = -1; ShowContactsDialog = true; }
    [RelayCommand] private void CloseContactsDialog() => ShowContactsDialog = false;

    [RelayCommand]
    private void AddContact()
    {
        ContactError = "";
        if (string.IsNullOrWhiteSpace(ContactName)) { ContactError = "Name required."; return; }
        var v = AddressValidator.Validate(ContactAddress, SelectedNetwork);
        if (!v.Valid) { ContactError = v.Error ?? "Invalid address."; return; }
        if (Contacts.Any(c => c.Address.Equals(ContactAddress.Trim(), StringComparison.OrdinalIgnoreCase)))
        { ContactError = "Address already in contacts."; return; }
        Contacts.Add(new Contact(ContactName.Trim(), ContactAddress.Trim(), ContactNote.Trim()));
        SaveContacts();
        ContactName = ""; ContactAddress = ""; ContactNote = "";
        ShowToast("Contact added.", ToastKind.Ok);
    }

    [RelayCommand]
    private void RemoveContact()
    {
        if (SelectedContactEditIndex >= 0 && SelectedContactEditIndex < Contacts.Count)
        {
            Contacts.RemoveAt(SelectedContactEditIndex);
            SaveContacts();
            ShowToast("Contact removed.", ToastKind.Info);
        }
    }

    private void SaveContacts()
    {
        _profile.Contacts = Contacts.ToList();
        _configService.Save(_config);
    }

    // ═══════════════════════════════════
    //  CONFIG
    // ═══════════════════════════════════

    [RelayCommand]
    private async Task SaveCliPath()
    {
        RecordActivity();
        _profile.CliPath = CliPath.Trim();
        _configService.Save(_config);
        _cliBinary = _configService.ResolveCliBinary(_config);
        if (_cliBinary != null)
        {
            var r = await _cli.RunCaptureAsync(_cliBinary, new[] { "--version" }, 5);
            ShowToast(r.ExitCode == 0 ? "CLI path saved." : "CLI binary failed.", r.ExitCode == 0 ? ToastKind.Ok : ToastKind.Error);
        }
        UpdatePill();
    }

    [RelayCommand]
    private void SaveSessionSettings()
    {
        _profile.SessionTimeoutMinutes = SessionTimeoutMinutes;
        _profile.AutoLockOnTimeout = AutoLockOnTimeout;
        _configService.Save(_config);
        ShowToast("Session settings saved.", ToastKind.Ok);
    }

    // ═══════════════════════════════════
    //  ACTIONS
    // ═══════════════════════════════════

    [RelayCommand]
    private async Task RunCliCommand(string args)
    {
        RecordActivity();
        if (_cliBinary == null) { ShowToast("CLI not configured.", ToastKind.Error); return; }
        SetBusy($"Running: {args}"); UpdatePill();
        try
        {
            Log($"> lmt-cli {args}");
            var result = await _cli.RunCaptureAsync(_cliBinary, args.Split(' ', StringSplitOptions.RemoveEmptyEntries));
            var stripped = AnsiStripper.Strip(result.Output);
            Log(stripped);
            if (result.ExitCode == 0)
            {
                ShowToast("Command completed.", ToastKind.Ok);
                AddHistory(ActionCategory.Wallet, args, StatusType.Ok, stripped);
            }
            else
            {
                var action = _cli.MapCliErrorAction(result.ExitCode, result.Output);
                ShowToast(action.Message, ToastKind.Error);
                AddHistory(ActionCategory.Wallet, args, StatusType.Error, stripped);
                if (action.ActionKey != null)
                {
                    ErrorDialogMessage = action.Message;
                    ErrorDialogActionKey = action.ActionKey;
                    ErrorDialogAction = action.ActionKey switch
                    {
                        "open_wallet" => "Open Wallet",
                        "select_network" => "Change Network",
                        "check_node" => "Check Node",
                        "wait" => "OK",
                        _ => "Dismiss"
                    };
                    ShowErrorDialog = true;
                }
            }
        }
        finally { ClearBusy(); UpdatePill(); }
    }

    [RelayCommand]
    private void LaunchInteractive(string args)
    {
        RecordActivity();
        if (_cliBinary == null) { ShowToast("CLI not configured.", ToastKind.Error); return; }
        _ = _cli.RunCaptureAsync(_cliBinary, new[] { "network", SelectedNetwork }, 5);
        var parts = args.Split(' ', StringSplitOptions.RemoveEmptyEntries);
        var (success, msg) = _cli.LaunchInteractive(_cliBinary, parts);
        if (success) { ShowToast($"Launched: {args}", ToastKind.Info); Log($"Interactive: {args}"); }
        else { ShowToast(msg, ToastKind.Error); }
        AddHistory(ActionCategory.Wallet, args, success ? StatusType.Pending : StatusType.Error);
    }

    [RelayCommand]
    private async Task LockWallet()
    {
        if (_cliBinary == null) return;
        await _cli.RunCaptureAsync(_cliBinary, new[] { "wallet", "close" });
        WalletOpen = false; WalletName = "";
        UpdatePill();
        ShowToast("Wallet locked.", ToastKind.Info);
        Log("Wallet locked.");
        AddHistory(ActionCategory.Wallet, "Locked", StatusType.Ok);
    }

    [RelayCommand]
    private async Task OpenWallet()
    {
        RecordActivity();
        LaunchInteractive("wallet open");
        ShowToast("Opening wallet in terminal...", ToastKind.Info);
        for (int i = 0; i < 15; i++)
        {
            await Task.Delay(2000);
            if (_cliBinary == null) break;
            var check = await _cli.RunCaptureAsync(_cliBinary, new[] { "list" }, 5);
            if (_cli.IsWalletOpenFromOutput(check.ExitCode, check.Output))
            {
                WalletOpen = true; WalletName = _profile.LastWallet;
                UpdatePill();
                ShowToast("Wallet opened!", ToastKind.Ok);
                AddHistory(ActionCategory.Wallet, "Opened", StatusType.Ok);
                return;
            }
        }
    }

    [RelayCommand] private async Task RefreshBalances() { RecordActivity(); await RunCliCommand("list"); }
    [RelayCommand] private async Task GetAddress() { RecordActivity(); await RunCliCommand("address"); }
    [RelayCommand] private async Task NewAddress() { RecordActivity(); await RunCliCommand("address new"); }

    [RelayCommand]
    private async Task RefreshTransactions()
    {
        RecordActivity();
        if (_cliBinary == null) return;
        SetBusy("Loading transactions...");
        try
        {
            var result = await _cli.RunCaptureAsync(_cliBinary, new[] { "history", "list", "30" });
            Transactions.Clear();
            foreach (var tx in TxHistoryParser.ParseHistoryOutput(result.ExitCode, result.Output))
                Transactions.Add(tx);
            ShowToast($"{Transactions.Count} transactions loaded.", ToastKind.Ok);
        }
        finally { ClearBusy(); }
    }

    // ── Error dialog ──
    [RelayCommand] private void DismissErrorDialog() => ShowErrorDialog = false;
    [RelayCommand]
    private void ExecuteErrorAction()
    {
        ShowErrorDialog = false;
        switch (ErrorDialogActionKey)
        {
            case "open_wallet": _ = OpenWallet(); break;
            case "select_network": /* focus config tab */ break;
            case "check_node": /* focus node tab */ break;
        }
    }

    // ═══════════════════════════════════
    //  SESSION TIMEOUT
    // ═══════════════════════════════════

    private void StartSessionTimer()
    {
        _sessionCts?.Cancel();
        if (SessionTimeoutMinutes <= 0) return;
        _sessionCts = new CancellationTokenSource();
        _ = SessionTimerLoop(_sessionCts.Token);
    }

    private async Task SessionTimerLoop(CancellationToken ct)
    {
        while (!ct.IsCancellationRequested)
        {
            await Task.Delay(60_000, ct);
            if (!WalletOpen || SessionTimeoutMinutes <= 0) continue;
            var elapsed = (DateTime.Now - _lastActivity).TotalMinutes;
            if (elapsed >= SessionTimeoutMinutes)
            {
                if (AutoLockOnTimeout)
                {
                    await LockWallet();
                    ShowToast("Wallet locked (inactivity).", ToastKind.Warn);
                }
                // If not auto-lock, we'd show a dialog — for now just warn
            }
        }
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
                else { NodeStatusText = "Disconnected"; DaaScore = Peers = TipHash = Difficulty = NetworkName = "—"; }
            }
            catch (OperationCanceledException) { break; }
            catch { NodeStatusText = "Error"; }
        }
    }
}
