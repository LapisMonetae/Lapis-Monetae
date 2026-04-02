namespace LmtDesktop.Core.Validation;

/// <summary>
/// Validates LMT addresses using Bech32/CashAddr-style checksums.
/// Ported from Python wallet_app/validators.py.
/// </summary>
public static class AddressValidator
{
    private const string Bech32Charset = "qpzry9x8gf2tvdw0s3jn54khce6mua7l";

    private static readonly Dictionary<char, int> Bech32Map =
        Bech32Charset.Select((c, i) => (c, i)).ToDictionary(x => x.c, x => x.i);

    private static readonly HashSet<char> Bech32Set = new(Bech32Charset);

    private static readonly ulong[] Generators =
    {
        0x98f2bc8e61, 0x79b76d99e2, 0xf33e5fb3c4, 0xae2eabe2a8, 0x1e4f43e470
    };

    private static readonly Dictionary<string, string> NetworkPrefixes = new()
    {
        ["mainnet"] = "lmt",
        ["testnet-10"] = "lmttest",
        ["testnet-11"] = "lmttest",
    };

    private static readonly HashSet<string> AllKnownPrefixes = new()
    {
        "lmt", "lmttest", "lmtsim", "lmtdev"
    };

    /// <summary>
    /// CashAddr-style polymod checksum calculation.
    /// </summary>
    private static ulong Polymod(IEnumerable<int> values)
    {
        ulong c = 1;
        foreach (var d in values)
        {
            var c0 = c >> 35;
            c = ((c & 0x07_FFFF_FFFF) << 5) ^ (ulong)d;
            for (int i = 0; i < 5; i++)
            {
                if ((c0 & (1UL << i)) != 0)
                    c ^= Generators[i];
            }
        }
        return c ^ 1;
    }

    private static bool VerifyChecksum(string prefix, string payload)
    {
        var prefix5Bit = prefix.Select(b => (int)(b & 0x1F)).ToList();
        var payload5Bit = new List<int>();
        foreach (var ch in payload)
        {
            if (!Bech32Map.TryGetValue(ch, out var val))
                return false;
            payload5Bit.Add(val);
        }

        var combined = new List<int>(prefix5Bit.Count + 1 + payload5Bit.Count);
        combined.AddRange(prefix5Bit);
        combined.Add(0);
        combined.AddRange(payload5Bit);

        return Polymod(combined) == 0;
    }

    /// <summary>
    /// Validate an LMT address for a specific network.
    /// </summary>
    /// <param name="address">The address string to validate.</param>
    /// <param name="network">Network name (mainnet, testnet-10, testnet-11). Null to accept any known prefix.</param>
    /// <returns>Validation result with error message if invalid.</returns>
    public static AddressValidationResult Validate(string address, string? network = null)
    {
        var normalized = (address ?? "").Trim().ToLowerInvariant();

        if (normalized.Length < 24)
            return AddressValidationResult.Fail("Address is too short.");

        var colonIdx = normalized.IndexOf(':');
        if (colonIdx < 0)
            return AddressValidationResult.Fail("Address must contain a ':' separator.");

        var prefix = normalized[..colonIdx];
        var payload = normalized[(colonIdx + 1)..];

        if (network != null)
        {
            if (!NetworkPrefixes.TryGetValue(network, out var expectedPrefix))
                return AddressValidationResult.Fail($"Unknown network: {network}");
            if (prefix != expectedPrefix)
                return AddressValidationResult.Fail($"Address prefix '{prefix}' does not match network '{network}' (expected '{expectedPrefix}').");
        }
        else
        {
            if (!AllKnownPrefixes.Contains(prefix))
                return AddressValidationResult.Fail($"Unknown address prefix: {prefix}");
        }

        if (string.IsNullOrEmpty(payload))
            return AddressValidationResult.Fail("Address payload is empty.");

        var invalidChars = payload.Where(c => !Bech32Set.Contains(c)).Distinct().OrderBy(c => c).ToArray();
        if (invalidChars.Length > 0)
            return AddressValidationResult.Fail($"Invalid characters in address: {new string(invalidChars)}");

        if (!VerifyChecksum(prefix, payload))
            return AddressValidationResult.Fail("Address checksum is invalid.");

        return AddressValidationResult.Ok();
    }

    /// <summary>
    /// Detect network from address prefix (for miner GUI network consistency check).
    /// </summary>
    public static string NetworkFromPrefix(string address)
    {
        var addr = (address ?? "").Trim().ToLowerInvariant();
        if (addr.StartsWith("lmttest:")) return "testnet";
        if (addr.StartsWith("lmtsim:")) return "simnet";
        if (addr.StartsWith("lmtdev:")) return "devnet";
        return "mainnet";
    }
}

public readonly record struct AddressValidationResult(bool Valid, string? Error)
{
    public static AddressValidationResult Ok() => new(true, null);
    public static AddressValidationResult Fail(string error) => new(false, error);
}
