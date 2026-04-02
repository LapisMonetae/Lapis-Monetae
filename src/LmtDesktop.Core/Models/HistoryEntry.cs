namespace LmtDesktop.Core.Models;

public enum StatusType { Ok, Error, Pending, Info }
public enum ActionCategory { Wallet, Address, Send, Transfer, Network, System }

public record HistoryEntry(
    string Timestamp,
    ActionCategory Category,
    string Description,
    StatusType Status,
    string? Detail = null)
{
    public string StatusSymbol => Status switch
    {
        StatusType.Ok => "\u2713",
        StatusType.Error => "\u2717",
        StatusType.Pending => "\u2026",
        StatusType.Info => "\u25CF",
        _ => "?"
    };
}
