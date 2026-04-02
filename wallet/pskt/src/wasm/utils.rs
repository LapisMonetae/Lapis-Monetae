use lmt_consensus_core::constants::*;
use lmt_consensus_core::network::NetworkType;
use separator::{separated_float, separated_int, separated_uint_with_output, Separatable};

#[inline]
pub fn sompi_to_lmt(sompi: u64) -> f64 {
    sompi as f64 / SOMPI_PER_LMT as f64
}

#[inline]
pub fn lmt_to_sompi(lmt: f64) -> u64 {
    (lmt * SOMPI_PER_LMT as f64) as u64
}

#[inline]
pub fn sompi_to_lmt_string(sompi: u64) -> String {
    sompi_to_lmt(sompi).separated_string()
}

#[inline]
pub fn sompi_to_lmt_string_with_trailing_zeroes(sompi: u64) -> String {
    separated_float!(format!("{:.8}", sompi_to_lmt(sompi)))
}

pub fn lmt_suffix(network_type: &NetworkType) -> &'static str {
    match network_type {
        NetworkType::Mainnet => "LMT",
        NetworkType::Testnet => "TLMT",
        NetworkType::Simnet => "SLMT",
        NetworkType::Devnet => "DLMT",
    }
}

#[inline]
pub fn sompi_to_lmt_string_with_suffix(sompi: u64, network_type: &NetworkType) -> String {
    let lmt = sompi_to_lmt_string(sompi);
    let suffix = lmt_suffix(network_type);
    format!("{lmt} {suffix}")
}
