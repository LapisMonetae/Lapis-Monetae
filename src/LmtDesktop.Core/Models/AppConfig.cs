using System.Text.Json.Serialization;

namespace LmtDesktop.Core.Models;

public class AppConfig
{
    [JsonPropertyName("_version")]
    public int Version { get; set; } = 2;

    [JsonPropertyName("active_profile")]
    public string ActiveProfile { get; set; } = "default";

    [JsonPropertyName("profiles")]
    public Dictionary<string, WalletProfile> Profiles { get; set; } = new()
    {
        ["default"] = new WalletProfile()
    };
}

public class WalletProfile
{
    [JsonPropertyName("cli_path")]
    public string CliPath { get; set; } = "";

    [JsonPropertyName("network")]
    public string Network { get; set; } = "mainnet";

    [JsonPropertyName("last_wallet")]
    public string LastWallet { get; set; } = "";

    [JsonPropertyName("contacts")]
    public List<Contact> Contacts { get; set; } = new();

    [JsonPropertyName("session_timeout_minutes")]
    public int SessionTimeoutMinutes { get; set; }

    [JsonPropertyName("auto_lock_on_timeout")]
    public bool AutoLockOnTimeout { get; set; } = true;

    [JsonPropertyName("seed_backup_confirmed")]
    public Dictionary<string, bool> SeedBackupConfirmed { get; set; } = new();
}
