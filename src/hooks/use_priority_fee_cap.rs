use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{gateway::PRIORITY_FEE_CAP, hooks::use_persistent::use_persistent};

const KEY: &str = "priority_fee_cap";

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct PriorityFeeCap(pub u64);

pub fn use_priority_fee_cap() -> Signal<PriorityFeeCap> {
    let priority_fee_cap = use_context::<Signal<PriorityFeeCap>>();
    let mut priority_fee_cap_persistent = use_persistent(KEY, || PriorityFeeCap(PRIORITY_FEE_CAP));
    use_effect(move || priority_fee_cap_persistent.set(priority_fee_cap.read().clone()));
    priority_fee_cap
}

pub fn use_priority_fee_cap_provider() {
    let priority_fee_cap = use_persistent(KEY, || PriorityFeeCap(PRIORITY_FEE_CAP));
    use_context_provider(|| Signal::new(priority_fee_cap.get()));
}
