using System.Text.RegularExpressions;

namespace LmtDesktop.Core.Helpers;

public record TxRow(string When, string TxId, string FullTxId, string Direction, string Amount, string Status, string Raw);

public static partial class TxHistoryParser
{
    [GeneratedRegex(@"\b[a-fA-F0-9]{64}\b")]
    private static partial Regex TxIdRegex();

    [GeneratedRegex(@"([+-]?\d+(?:\.\d+)?)\s+(?:LMT|TLMT|SLMT|DLMT)\b")]
    private static partial Regex AmountRegex();

    public static List<TxRow> ParseHistoryOutput(int exitCode, string output)
    {
        var rows = new List<TxRow>();
        if (exitCode != 0 || string.IsNullOrWhiteSpace(output))
            return rows;

        var lines = AnsiStripper.Strip(output).Split('\n');
        foreach (var rawLine in lines)
        {
            var line = rawLine.Trim();
            if (string.IsNullOrEmpty(line) || line.StartsWith("exit code", StringComparison.OrdinalIgnoreCase))
                continue;

            var txIdMatch = TxIdRegex().Match(line);
            if (!txIdMatch.Success)
                continue;

            var fullTxId = txIdMatch.Value;
            var shortTxId = fullTxId[..8] + "...";

            var amountMatch = AmountRegex().Match(line);
            var amount = amountMatch.Success ? amountMatch.Groups[1].Value : "—";

            var lower = line.ToLowerInvariant();
            var direction = (lower.Contains("received") || lower.Contains("inbound")) ? "in" : "out";
            var status = lower.Contains("pending") ? "pending" : "confirmed";
            var when = DateTime.Now.ToString("HH:mm:ss");

            rows.Add(new TxRow(when, shortTxId, fullTxId, direction, amount, status, line));
        }

        return rows;
    }
}
