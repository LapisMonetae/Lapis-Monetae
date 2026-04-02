using System.Text.RegularExpressions;

namespace LmtDesktop.Core.Helpers;

public record MinerMetrics(
    string Hashrate10S,
    string Hashrate60S,
    string Hashrate15M,
    string HashrateMax,
    int Accepted,
    int Rejected,
    int? LatencyMs,
    bool PoolReconnect);

public static partial class MetricsParser
{
    [GeneratedRegex(@"speed\s+\S+\s+([\d.]+|n/a)\s+([\d.]+|n/a)\s+([\d.]+|n/a)\s+H/s\s+max\s+([\d.]+|n/a)")]
    private static partial Regex HashrateRegex();

    [GeneratedRegex(@"accepted\s+\((\d+)/(\d+)\)")]
    private static partial Regex SharesRegex();

    [GeneratedRegex(@"accepted.*?\((\d+)\s*ms\)")]
    private static partial Regex LatencyRegex();

    [GeneratedRegex(@"\[(?:pool|net)\].*")]
    private static partial Regex PoolUseRegex();

    public static MinerMetrics? ParseLine(string line)
    {
        if (string.IsNullOrWhiteSpace(line))
            return null;

        var hr = HashrateRegex().Match(line);
        if (hr.Success)
        {
            return new MinerMetrics(
                hr.Groups[1].Value, hr.Groups[2].Value,
                hr.Groups[3].Value, hr.Groups[4].Value,
                0, 0, null, false);
        }

        var shares = SharesRegex().Match(line);
        if (shares.Success)
        {
            var total = int.Parse(shares.Groups[1].Value);
            var rejected = int.Parse(shares.Groups[2].Value);
            var accepted = total - rejected;
            var latency = LatencyRegex().Match(line);
            return new MinerMetrics(
                "", "", "", "", accepted, rejected,
                latency.Success ? int.Parse(latency.Groups[1].Value) : null,
                false);
        }

        if (PoolUseRegex().IsMatch(line))
            return new MinerMetrics("", "", "", "", 0, 0, null, true);

        return null;
    }
}
