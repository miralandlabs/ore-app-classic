use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::hooks::use_persistent::use_persistent;

const KEY: &str = "priority_fee";

pub const DEFAULT_PRIORITY_FEE: u64 = 5000; // microlamports

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct PriorityFee(pub u64);

// initialize to 5000 microlamports 
pub fn use_priority_fee() -> Signal<PriorityFee> {
    let priority_fee = use_context::<Signal<PriorityFee>>();
    let mut priority_fee_persistent = use_persistent(KEY, || PriorityFee(DEFAULT_PRIORITY_FEE));
    use_effect(move || priority_fee_persistent.set(*priority_fee.read()));
    priority_fee
}

pub fn use_priority_fee_provider() {
    let priority_fee = use_persistent(KEY, || PriorityFee(DEFAULT_PRIORITY_FEE)).get();
    use_context_provider(|| Signal::new(priority_fee));
}
