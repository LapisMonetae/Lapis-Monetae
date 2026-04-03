namespace LmtDesktop.Core.Models;

public enum ToastKind { Ok, Error, Info, Warn }

public record ToastMessage(string Text, ToastKind Kind)
{
    public string Icon => Kind switch
    {
        ToastKind.Ok => "\u2713",     // ✓
        ToastKind.Error => "\u2717",  // ✗
        ToastKind.Info => "\u25CF",   // ●
        ToastKind.Warn => "\u26A0",  // ⚠
        _ => ""
    };

    public string Background => Kind switch
    {
        ToastKind.Ok => "#f0fdf4",
        ToastKind.Error => "#fef2f2",
        ToastKind.Info => "#eff6ff",
        ToastKind.Warn => "#fffbeb",
        _ => "#f4f6fa"
    };

    public string Foreground => Kind switch
    {
        ToastKind.Ok => "#166534",
        ToastKind.Error => "#991b1b",
        ToastKind.Info => "#1e3a5f",
        ToastKind.Warn => "#92400e",
        _ => "#1e293b"
    };

    public string AccentColor => Kind switch
    {
        ToastKind.Ok => "#16a34a",
        ToastKind.Error => "#dc2626",
        ToastKind.Info => "#2563eb",
        ToastKind.Warn => "#d97706",
        _ => "#64748b"
    };
}
