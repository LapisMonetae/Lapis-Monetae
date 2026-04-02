use crate::error::Error;
use crate::result::Result;
use lmt_consensus_core::constants::SOMPI_PER_LMT;
use std::fmt::Display;

pub fn try_parse_required_nonzero_lmt_as_sompi_u64<S: ToString + Display>(lmt_amount: Option<S>) -> Result<u64> {
    if let Some(lmt_amount) = lmt_amount {
        let sompi_amount = lmt_amount
            .to_string()
            .parse::<f64>()
            .map_err(|_| Error::custom(format!("Supplied LMT amount is not valid: '{lmt_amount}'")))?
            * SOMPI_PER_LMT as f64;
        if sompi_amount < 0.0 {
            Err(Error::custom("Supplied LMT amount is not valid: '{lmt_amount}'"))
        } else {
            let sompi_amount = sompi_amount as u64;
            if sompi_amount == 0 {
                Err(Error::custom("Supplied required Lapis Monetae amount must not be a zero: '{lmt_amount}'"))
            } else {
                Ok(sompi_amount)
            }
        }
    } else {
        Err(Error::custom("Missing LMT amount"))
    }
}

pub fn try_parse_required_lmt_as_sompi_u64<S: ToString + Display>(lmt_amount: Option<S>) -> Result<u64> {
    if let Some(lmt_amount) = lmt_amount {
        let sompi_amount = lmt_amount
            .to_string()
            .parse::<f64>()
            .map_err(|_| Error::custom(format!("Supplied LMT amount is not valid: '{lmt_amount}'")))?
            * SOMPI_PER_LMT as f64;
        if sompi_amount < 0.0 {
            Err(Error::custom("Supplied LMT amount is not valid: '{lmt_amount}'"))
        } else {
            Ok(sompi_amount as u64)
        }
    } else {
        Err(Error::custom("Missing LMT amount"))
    }
}

pub fn try_parse_optional_lmt_as_sompi_i64<S: ToString + Display>(lmt_amount: Option<S>) -> Result<Option<i64>> {
    if let Some(lmt_amount) = lmt_amount {
        let sompi_amount = lmt_amount
            .to_string()
            .parse::<f64>()
            .map_err(|_e| Error::custom(format!("Supplied LMT amount is not valid: '{lmt_amount}'")))?
            * SOMPI_PER_LMT as f64;
        if sompi_amount < 0.0 {
            Err(Error::custom("Supplied LMT amount is not valid: '{lmt_amount}'"))
        } else {
            Ok(Some(sompi_amount as i64))
        }
    } else {
        Ok(None)
    }
}
