using LmtDesktop.Core.Models;

namespace LmtDesktop.Core.Services;

public interface IConfigService
{
    AppConfig Load();
    void Save(AppConfig config);
    WalletProfile ActiveProfile(AppConfig config);
    string? ResolveCliBinary(AppConfig config);
    bool HasAnyWallet(AppConfig config);
}
