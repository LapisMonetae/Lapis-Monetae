using System.Text.RegularExpressions;

namespace LmtDesktop.Core.Helpers;

public static partial class AnsiStripper
{
    [GeneratedRegex(@"\x1b\[[0-9;]*m")]
    private static partial Regex AnsiEscapeRegex();

    public static string Strip(string text) =>
        AnsiEscapeRegex().Replace(text ?? "", "");
}
