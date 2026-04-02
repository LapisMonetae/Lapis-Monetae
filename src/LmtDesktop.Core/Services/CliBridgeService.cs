using System.Diagnostics;
using System.Runtime.InteropServices;
using LmtDesktop.Core.Helpers;

namespace LmtDesktop.Core.Services;

public class CliBridgeService : ICliBridge
{
    public async Task<CliResult> RunCaptureAsync(string binary, string[] args, int timeoutSec = 25, CancellationToken ct = default)
    {
        var psi = new ProcessStartInfo
        {
            FileName = binary,
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            UseShellExecute = false,
            CreateNoWindow = true,
        };
        foreach (var arg in args)
            psi.ArgumentList.Add(arg);

        using var proc = new Process { StartInfo = psi };
        proc.Start();

        var stdout = proc.StandardOutput.ReadToEndAsync(ct);
        var stderr = proc.StandardError.ReadToEndAsync(ct);

        using var cts = CancellationTokenSource.CreateLinkedTokenSource(ct);
        cts.CancelAfter(TimeSpan.FromSeconds(timeoutSec));

        try
        {
            await proc.WaitForExitAsync(cts.Token);
        }
        catch (OperationCanceledException)
        {
            try { proc.Kill(entireProcessTree: true); } catch { }
            return new CliResult(-1, "Command timed out");
        }

        var output = await stdout + "\n" + await stderr;
        return new CliResult(proc.ExitCode, output.Trim());
    }

    public async Task<CliTimedResult> RunCaptureTimedAsync(string binary, string[] args, int timeoutSec = 25, CancellationToken ct = default)
    {
        var sw = Stopwatch.StartNew();
        var result = await RunCaptureAsync(binary, args, timeoutSec, ct);
        sw.Stop();
        return new CliTimedResult(result.ExitCode, result.Output, sw.Elapsed.TotalMilliseconds);
    }

    public (bool Success, string Message) LaunchInteractive(string binary, string[] args)
    {
        try
        {
            var psi = new ProcessStartInfo
            {
                FileName = binary,
                UseShellExecute = true,
                CreateNoWindow = false,
            };
            foreach (var arg in args)
                psi.ArgumentList.Add(arg);

            if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
            {
                // On Windows, UseShellExecute=true opens a new console window
            }

            Process.Start(psi);
            return (true, "Interactive command launched");
        }
        catch (Exception ex)
        {
            return (false, $"Failed to launch: {ex.Message}");
        }
    }

    public string MapCliError(int exitCode, string output)
    {
        var lower = output.ToLowerInvariant();
        if (lower.Contains("wallet is not open"))
            return "No wallet is currently open. Please open a wallet first.";
        if (lower.Contains("insufficient") || lower.Contains("not enough"))
            return "Insufficient funds for this transaction.";
        if (lower.Contains("network") && lower.Contains("select"))
            return "Please select a network first.";
        if (lower.Contains("syncing") || lower.Contains("not synced"))
            return "The node is still syncing. Please wait.";
        if (lower.Contains("connection") && (lower.Contains("refused") || lower.Contains("failed")))
            return "Could not connect to the node. Check that lmtd is running.";
        if (exitCode == -1)
            return "Command timed out.";
        if (exitCode != 0)
            return $"Command failed (exit code {exitCode}).";
        return "";
    }

    public CliErrorAction MapCliErrorAction(int exitCode, string output)
    {
        var lower = output.ToLowerInvariant();
        if (lower.Contains("wallet is not open"))
            return new CliErrorAction("Open a wallet first.", "open_wallet");
        if (lower.Contains("network") && lower.Contains("select"))
            return new CliErrorAction("Select a network.", "select_network");
        if (lower.Contains("syncing") || lower.Contains("not synced"))
            return new CliErrorAction("Node is syncing. Please wait.", "wait");
        if (lower.Contains("connection") && (lower.Contains("refused") || lower.Contains("failed")))
            return new CliErrorAction("Node connection failed.", "check_node");
        return new CliErrorAction(MapCliError(exitCode, output), null);
    }

    public bool IsWalletOpenFromOutput(int exitCode, string output)
    {
        if (exitCode != 0) return false;
        var lower = output.ToLowerInvariant();
        return !lower.Contains("wallet is not open") && !lower.Contains("not connected");
    }

    public (bool Connected, bool? Synced) ParseNodeSyncHint(string output)
    {
        var lower = output.ToLowerInvariant();
        if (lower.Contains("wallet is not connected") || lower.Contains("not connected"))
            return (false, null);
        if (lower.Contains("is currently syncing"))
            return (true, false);
        return (true, true);
    }
}
