namespace LmtDesktop.Core.Services;

public record CliResult(int ExitCode, string Output);
public record CliTimedResult(int ExitCode, string Output, double ElapsedMs);
public record CliErrorAction(string Message, string? ActionKey);

public interface ICliBridge
{
    Task<CliResult> RunCaptureAsync(string binary, string[] args, int timeoutSec = 25, CancellationToken ct = default);
    Task<CliTimedResult> RunCaptureTimedAsync(string binary, string[] args, int timeoutSec = 25, CancellationToken ct = default);
    (bool Success, string Message) LaunchInteractive(string binary, string[] args);
    string MapCliError(int exitCode, string output);
    CliErrorAction MapCliErrorAction(int exitCode, string output);
    bool IsWalletOpenFromOutput(int exitCode, string output);
    (bool Connected, bool? Synced) ParseNodeSyncHint(string output);
}
