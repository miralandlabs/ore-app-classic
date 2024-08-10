use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{gateway::FEE_URL, hooks::use_persistent::use_persistent};

const KEY: &str = "fee_url";

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct FeeUrl(pub String);

pub fn use_fee_url() -> Signal<FeeUrl> {
    let fee_url = use_context::<Signal<FeeUrl>>();
    let mut fee_url_persistent = use_persistent(KEY, || FeeUrl(FEE_URL.to_string()));
    use_effect(move || fee_url_persistent.set(fee_url.read().clone()));
    fee_url
}

pub fn use_fee_url_provider() {
    let fee_url = use_persistent(KEY, || FeeUrl(FEE_URL.to_string()));
    use_context_provider(|| Signal::new(fee_url.get()));
}
