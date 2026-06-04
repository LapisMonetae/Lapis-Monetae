/// Bech32 address validator for LMT addresses.
/// Prefixes: lmt: (mainnet), lmttest: (testnet), lmtsim: (simnet), lmtdev: (devnet)

const CHARSET: &str = "qpzry9x8gf2tvdw0s3jn54khce6mua7l";

fn charset_index(c: char) -> Option<u32> {
    CHARSET.find(c).map(|i| i as u32)
}

fn polymod(values: &[u32]) -> u32 {
    let gen: [u32; 5] = [0x3b6a57b2, 0x26508e6d, 0x1ea119fa, 0x3d4233dd, 0x2a1462b3];
    let mut chk: u32 = 1;
    for &v in values {
        let b = chk >> 25;
        chk = ((chk & 0x1ffffff) << 5) ^ v;
        for (i, &g) in gen.iter().enumerate() {
            if (b >> i) & 1 != 0 {
                chk ^= g;
            }
        }
    }
    chk
}

fn hrp_expand(hrp: &str) -> Vec<u32> {
    let mut ret: Vec<u32> = hrp.chars().map(|c| (c as u32) >> 5).collect();
    ret.push(0);
    ret.extend(hrp.chars().map(|c| (c as u32) & 31));
    ret
}

pub fn validate_address(address: &str, network: &str) -> Result<(), String> {
    if address.is_empty() {
        return Err("Address is empty".into());
    }

    let expected_prefix = match network {
        "mainnet" => "lmt:",
        "testnet-10" | "testnet-11" => "lmttest:",
        "simnet" => "lmtsim:",
        "devnet" => "lmtdev:",
        _ => "lmt:",
    };

    let lower = address.to_lowercase();
    if !lower.starts_with(expected_prefix) {
        return Err(format!("Address must start with '{expected_prefix}' for {network}"));
    }

    let colon_pos = lower.find(':').unwrap();
    let hrp = &lower[..colon_pos];
    let data_part = &lower[colon_pos + 1..];

    if data_part.len() < 6 {
        return Err("Address data too short".into());
    }

    let mut data_values = Vec::new();
    for c in data_part.chars() {
        match charset_index(c) {
            Some(v) => data_values.push(v),
            None => return Err(format!("Invalid character '{c}' in address")),
        }
    }

    let mut check_data = hrp_expand(hrp);
    check_data.extend(&data_values);
    if polymod(&check_data) != 1 {
        return Err("Invalid address checksum".into());
    }

    Ok(())
}

pub fn validate_amount(s: &str) -> Result<f64, String> {
    let amount: f64 = s.parse().map_err(|_| "Invalid amount".to_string())?;
    if amount <= 0.0 {
        return Err("Amount must be positive".into());
    }
    Ok(amount)
}

pub fn validate_fee(s: &str) -> Result<f64, String> {
    if s.is_empty() {
        return Ok(0.0);
    }
    let fee: f64 = s.parse().map_err(|_| "Invalid fee".to_string())?;
    if fee < 0.0 {
        return Err("Fee cannot be negative".into());
    }
    Ok(fee)
}
