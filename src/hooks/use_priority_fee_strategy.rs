use dioxus::prelude::*;

use crate::{components::PriorityFeeStrategy, hooks::use_persistent::use_persistent};

const KEY: &str = "priority_fee_strategy";

pub fn use_priority_fee_strategy() -> Signal<PriorityFeeStrategy> {
    let priority_fee_strategy = use_context::<Signal<PriorityFeeStrategy>>();
    let mut priority_fee_strategy_persistent = use_persistent(KEY, || PriorityFeeStrategy::Estimate);
    use_effect(move || priority_fee_strategy_persistent.set(*priority_fee_strategy.read()));
    priority_fee_strategy
}

pub fn use_priority_fee_strategy_provider() {
    let priority_fee_strategy = use_persistent(KEY, || PriorityFeeStrategy::Estimate).get();
    use_context_provider(|| Signal::new(priority_fee_strategy));
}
