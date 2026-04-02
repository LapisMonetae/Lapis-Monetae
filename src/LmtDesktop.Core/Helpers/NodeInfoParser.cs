using System.Text.RegularExpressions;

namespace LmtDesktop.Core.Helpers;

public record NodeRichInfo(
    string DaaScore,
    string Peers,
    string TipHash,
    string Headers,
    string Blocks,
    string Difficulty,
    string NetworkName,
    double LatencyMs);

public static partial class NodeInfoParser
{
    [GeneratedRegex(@"daa.?score[:\s]+(\d+)", RegexOptions.IgnoreCase)]
    private static partial Regex DaaScoreRegex();

    [GeneratedRegex(@"RpcPeerInfo", RegexOptions.IgnoreCase)]
    private static partial Regex PeerInfoRegex();

    [GeneratedRegex(@"tip.?hash[es]*[:\s]+([a-f0-9]{64})", RegexOptions.IgnoreCase)]
    private static partial Regex TipHashRegex();

    [GeneratedRegex(@"header.?count[:\s]+(\d+)", RegexOptions.IgnoreCase)]
    private static partial Regex HeaderCountRegex();

    [GeneratedRegex(@"block.?count[:\s]+(\d+)", RegexOptions.IgnoreCase)]
    private static partial Regex BlockCountRegex();

    [GeneratedRegex(@"difficulty[:\s]+([\d.eE+\-]+)", RegexOptions.IgnoreCase)]
    private static partial Regex DifficultyRegex();

    [GeneratedRegex(@"network.?name[:\s]+(\S+)", RegexOptions.IgnoreCase)]
    private static partial Regex NetworkNameRegex();

    public static NodeRichInfo Parse(string dagInfoOutput, string peerInfoOutput, double latencyMs)
    {
        var dag = AnsiStripper.Strip(dagInfoOutput ?? "");
        var peer = AnsiStripper.Strip(peerInfoOutput ?? "");

        string Extract(Regex rx, string text) =>
            rx.Match(text) is { Success: true } m ? m.Groups[1].Value : "—";

        var peerCount = PeerInfoRegex().Matches(peer).Count.ToString();

        return new NodeRichInfo(
            DaaScore: Extract(DaaScoreRegex(), dag),
            Peers: peerCount,
            TipHash: Extract(TipHashRegex(), dag),
            Headers: Extract(HeaderCountRegex(), dag),
            Blocks: Extract(BlockCountRegex(), dag),
            Difficulty: Extract(DifficultyRegex(), dag),
            NetworkName: Extract(NetworkNameRegex(), dag),
            LatencyMs: latencyMs
        );
    }
}
