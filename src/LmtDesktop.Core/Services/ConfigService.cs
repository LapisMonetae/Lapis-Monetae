using System.Runtime.InteropServices;
using System.Text.Json;
using LmtDesktop.Core.Models;

namespace LmtDesktop.Core.Services;

public class ConfigService : IConfigService
{
    private readonly string _configPath;

    public static readonly string[] NetworkChoices = { "mainnet", "testnet-10", "testnet-11" };
    public static readonly int[] TimeoutChoices = { 0, 5, 10, 15, 30, 60 };

    public ConfigService(string? configPath = null)
    {
        _configPath = configPath ?? GetDefaultConfigPath();
    }

    public string ConfigPath => _configPath;

    private static string GetDefaultConfigPath()
    {
        var home = RuntimeInformation.IsOSPlatform(OSPlatform.Windows)
            ? Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData)
            : Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);
        var dir = Path.Combine(home, RuntimeInformation.IsOSPlatform(OSPlatform.Windows) ? "lapis-monetae" : ".lapis-monetae");
        Directory.CreateDirectory(dir);
        return Path.Combine(dir, "wallet-gui-config.json");
    }

    public AppConfig Load()
    {
        if (!File.Exists(_configPath))
            return new AppConfig();
        try
        {
            var json = File.ReadAllText(_configPath);
            return JsonSerializer.Deserialize<AppConfig>(json) ?? new AppConfig();
        }
        catch
        {
            return new AppConfig();
        }
    }

    public void Save(AppConfig config)
    {
        var dir = Path.GetDirectoryName(_configPath);
        if (dir != null) Directory.CreateDirectory(dir);

        var json = JsonSerializer.Serialize(config, new JsonSerializerOptions { WriteIndented = true });
        File.WriteAllText(_configPath, json);

        if (!RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
        {
            try { File.SetUnixFileMode(_configPath, UnixFileMode.UserRead | UnixFileMode.UserWrite); }
            catch { }
        }
    }

    public WalletProfile ActiveProfile(AppConfig config)
    {
        var name = config.ActiveProfile ?? "default";
        if (config.Profiles.TryGetValue(name, out var profile))
            return profile;
        return new WalletProfile();
    }

    public string? ResolveCliBinary(AppConfig config)
    {
        var profile = ActiveProfile(config);
        var candidates = new List<string>();

        if (!string.IsNullOrWhiteSpace(profile.CliPath))
            candidates.Add(profile.CliPath.Trim());

        var envBin = Environment.GetEnvironmentVariable("LMT_CLI_BIN");
        if (!string.IsNullOrWhiteSpace(envBin))
            candidates.Add(envBin.Trim());

        candidates.Add("lmt-cli");
        if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
            candidates.Add("lmt-cli.exe");

        foreach (var candidate in candidates)
        {
            if (File.Exists(candidate))
                return Path.GetFullPath(candidate);
            var found = FindInPath(candidate);
            if (found != null)
                return found;
        }
        return null;
    }

    public bool HasAnyWallet(AppConfig config)
    {
        return config.Profiles.Values.Any(p => !string.IsNullOrWhiteSpace(p.LastWallet));
    }

    private static string? FindInPath(string binary)
    {
        var pathVar = Environment.GetEnvironmentVariable("PATH") ?? "";
        var sep = RuntimeInformation.IsOSPlatform(OSPlatform.Windows) ? ';' : ':';
        foreach (var dir in pathVar.Split(sep, StringSplitOptions.RemoveEmptyEntries))
        {
            var full = Path.Combine(dir.Trim(), binary);
            if (File.Exists(full))
                return full;
        }
        return null;
    }
}
