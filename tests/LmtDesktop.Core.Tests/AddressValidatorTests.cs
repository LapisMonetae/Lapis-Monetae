using LmtDesktop.Core.Validation;

namespace LmtDesktop.Core.Tests;

public class AddressValidatorTests
{
    [Fact]
    public void TooShort_ReturnsInvalid()
    {
        var result = AddressValidator.Validate("lmt:abc");
        Assert.False(result.Valid);
        Assert.Contains("too short", result.Error, StringComparison.OrdinalIgnoreCase);
    }

    [Fact]
    public void MissingSeparator_ReturnsInvalid()
    {
        var result = AddressValidator.Validate("lmtqqqqqqqqqqqqqqqqqqqqqqqqqqqq");
        Assert.False(result.Valid);
        Assert.Contains("separator", result.Error, StringComparison.OrdinalIgnoreCase);
    }

    [Fact]
    public void UnknownPrefix_ReturnsInvalid()
    {
        var result = AddressValidator.Validate("btc:qpzry9x8gf2tvdw0s3jn54khce6mua7lqqqqqqqqq");
        Assert.False(result.Valid);
        Assert.Contains("prefix", result.Error, StringComparison.OrdinalIgnoreCase);
    }

    [Fact]
    public void EmptyPayload_ReturnsInvalid()
    {
        // "lmt:" is only 4 chars, so it hits "too short" before "empty payload"
        var result = AddressValidator.Validate("lmt:");
        Assert.False(result.Valid);
    }

    [Fact]
    public void InvalidCharacters_ReturnsInvalid()
    {
        var result = AddressValidator.Validate("lmt:qpzry9x8gf2tvdw0s3jn54khce6muaBBBBBBBBBB");
        Assert.False(result.Valid);
        Assert.Contains("Invalid characters", result.Error, StringComparison.OrdinalIgnoreCase);
    }

    [Fact]
    public void WrongNetwork_ReturnsInvalid()
    {
        var result = AddressValidator.Validate("lmttest:qpzry9x8gf2tvdw0s3jn54khce6mua7lqqqqqqqqq", "mainnet");
        Assert.False(result.Valid);
        Assert.Contains("does not match", result.Error, StringComparison.OrdinalIgnoreCase);
    }

    [Fact]
    public void NetworkFromPrefix_Mainnet()
    {
        Assert.Equal("mainnet", AddressValidator.NetworkFromPrefix("lmt:something"));
    }

    [Fact]
    public void NetworkFromPrefix_Testnet()
    {
        Assert.Equal("testnet", AddressValidator.NetworkFromPrefix("lmttest:something"));
    }

    [Fact]
    public void NetworkFromPrefix_Simnet()
    {
        Assert.Equal("simnet", AddressValidator.NetworkFromPrefix("lmtsim:something"));
    }

    [Fact]
    public void NetworkFromPrefix_Devnet()
    {
        Assert.Equal("devnet", AddressValidator.NetworkFromPrefix("lmtdev:something"));
    }

    [Fact]
    public void NullAddress_ReturnsInvalid()
    {
        var result = AddressValidator.Validate(null!);
        Assert.False(result.Valid);
    }

    [Fact]
    public void WhitespaceAddress_ReturnsInvalid()
    {
        var result = AddressValidator.Validate("   ");
        Assert.False(result.Valid);
    }
}
