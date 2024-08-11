use std::{fmt, io, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq)]
pub enum PriorityFeeStrategy {
    #[default]
    Estimate,
    Static,
    // Static(/* priority fee: */ u64),
}

impl FromStr for PriorityFeeStrategy {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Estimate priority fee" => Ok(PriorityFeeStrategy::Estimate),
            "Static priority fee" => Ok(PriorityFeeStrategy::Static),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Unknown priority fee strategy",
            )),
        }
    }
}

impl fmt::Display for PriorityFeeStrategy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PriorityFeeStrategy::Estimate => write!(f, "Estimate priority fee"),
            PriorityFeeStrategy::Static => write!(f, "Static priority fee"),
        }
    }
}
