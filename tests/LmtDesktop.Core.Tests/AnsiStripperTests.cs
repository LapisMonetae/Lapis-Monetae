using LmtDesktop.Core.Helpers;

namespace LmtDesktop.Core.Tests;

public class AnsiStripperTests
{
    [Fact]
    public void StripAnsiCodes()
    {
        var input = "\x1b[31mERROR\x1b[0m: something failed";
        Assert.Equal("ERROR: something failed", AnsiStripper.Strip(input));
    }

    [Fact]
    public void NoAnsiCodes_Unchanged()
    {
        var input = "plain text";
        Assert.Equal("plain text", AnsiStripper.Strip(input));
    }

    [Fact]
    public void NullInput_ReturnsEmpty()
    {
        Assert.Equal("", AnsiStripper.Strip(null!));
    }
}
