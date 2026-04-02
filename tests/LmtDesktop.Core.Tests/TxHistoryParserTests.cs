using LmtDesktop.Core.Helpers;

namespace LmtDesktop.Core.Tests;

public class TxHistoryParserTests
{
    [Fact]
    public void ParsesTransactionWithTxId()
    {
        var txid = new string('a', 64);
        var output = $"received +10.5 LMT txid {txid} confirmed";
        var rows = TxHistoryParser.ParseHistoryOutput(0, output);
        Assert.Single(rows);
        Assert.Equal("in", rows[0].Direction);
        Assert.Equal("+10.5", rows[0].Amount);
        Assert.Equal("confirmed", rows[0].Status);
        Assert.Equal(txid, rows[0].FullTxId);
    }

    [Fact]
    public void ParsesSentTransaction()
    {
        var txid = new string('b', 64);
        var output = $"sent -5.0 LMT txid {txid} pending";
        var rows = TxHistoryParser.ParseHistoryOutput(0, output);
        Assert.Single(rows);
        Assert.Equal("out", rows[0].Direction);
        Assert.Equal("-5.0", rows[0].Amount);
        Assert.Equal("pending", rows[0].Status);
    }

    [Fact]
    public void IgnoresLinesWithoutTxId()
    {
        var output = "some random output without a txid\n";
        var rows = TxHistoryParser.ParseHistoryOutput(0, output);
        Assert.Empty(rows);
    }

    [Fact]
    public void NonZeroExitCode_ReturnsEmpty()
    {
        var txid = new string('c', 64);
        var rows = TxHistoryParser.ParseHistoryOutput(1, $"txid {txid}");
        Assert.Empty(rows);
    }

    [Fact]
    public void StripsAnsiBeforeParsing()
    {
        var txid = new string('d', 64);
        var output = $"\x1b[32mreceived +1.0 LMT\x1b[0m txid {txid}";
        var rows = TxHistoryParser.ParseHistoryOutput(0, output);
        Assert.Single(rows);
        Assert.Equal("+1.0", rows[0].Amount);
    }

    [Fact]
    public void ParsesTestnetTicker()
    {
        var txid = new string('e', 64);
        var output = $"received +2.5 TLMT txid {txid}";
        var rows = TxHistoryParser.ParseHistoryOutput(0, output);
        Assert.Single(rows);
        Assert.Equal("+2.5", rows[0].Amount);
    }
}
