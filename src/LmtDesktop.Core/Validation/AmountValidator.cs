using System.Globalization;

namespace LmtDesktop.Core.Validation;

/// <summary>
/// Validates LMT amounts and fees.
/// Ported from Python wallet_app/validators.py.
/// </summary>
public static class AmountValidator
{
    /// <summary>Parse a positive amount (must be greater than zero).</summary>
    public static double? ParsePositiveAmount(string? text)
    {
        if (string.IsNullOrWhiteSpace(text)) return null;
        if (!double.TryParse(text.Trim(), NumberStyles.Float, CultureInfo.InvariantCulture, out var amount))
            return null;
        return amount > 0 ? amount : null;
    }

    /// <summary>Parse a non-negative fee (zero is allowed, empty string returns 0).</summary>
    public static double? ParseNonnegativeFee(string? text)
    {
        if (string.IsNullOrWhiteSpace(text)) return 0.0;
        if (!double.TryParse(text.Trim(), NumberStyles.Float, CultureInfo.InvariantCulture, out var fee))
            return null;
        return fee >= 0 ? fee : null;
    }
}
