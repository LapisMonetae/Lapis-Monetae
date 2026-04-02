using LmtDesktop.Core.Validation;

namespace LmtDesktop.Core.Tests;

public class AmountValidatorTests
{
    [Theory]
    [InlineData("10.5", 10.5)]
    [InlineData("0.001", 0.001)]
    [InlineData("  100  ", 100.0)]
    public void ParsePositiveAmount_ValidValues(string input, double expected)
    {
        var result = AmountValidator.ParsePositiveAmount(input);
        Assert.NotNull(result);
        Assert.Equal(expected, result!.Value, 6);
    }

    [Theory]
    [InlineData("0")]
    [InlineData("-5")]
    [InlineData("abc")]
    [InlineData("")]
    [InlineData(null)]
    public void ParsePositiveAmount_InvalidValues(string? input)
    {
        Assert.Null(AmountValidator.ParsePositiveAmount(input));
    }

    [Theory]
    [InlineData("0", 0.0)]
    [InlineData("10.5", 10.5)]
    [InlineData("", 0.0)]
    [InlineData(null, 0.0)]
    [InlineData("  ", 0.0)]
    public void ParseNonnegativeFee_ValidValues(string? input, double expected)
    {
        var result = AmountValidator.ParseNonnegativeFee(input);
        Assert.NotNull(result);
        Assert.Equal(expected, result!.Value, 6);
    }

    [Theory]
    [InlineData("-1")]
    [InlineData("abc")]
    public void ParseNonnegativeFee_InvalidValues(string input)
    {
        Assert.Null(AmountValidator.ParseNonnegativeFee(input));
    }
}
