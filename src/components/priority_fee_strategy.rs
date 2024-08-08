use std::{fmt, io, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq)]
pub enum PriorityFeeStrategy {
    #[default]
    Static,
    Dynamic,
}

impl FromStr for PriorityFeeStrategy {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Static priority fee" => Ok(PriorityFeeStrategy::Static),
            "Dynamic priority fee" => Ok(PriorityFeeStrategy::Dynamic),
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
            PriorityFeeStrategy::Static => write!(f, "Static priority fee"),
            PriorityFeeStrategy::Dynamic => write!(f, "Dynamic priority fee"),
        }
    }
}
