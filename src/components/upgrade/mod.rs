mod confirm;
mod done;
mod edit;

use confirm::*;
use done::*;
use edit::*;

use dioxus::prelude::*;
use solana_client_wasm::solana_sdk::signature::Signature;

pub enum UpgradeStep {
    Edit,
    Confirm,
    Done(Signature),
}

#[component]
pub fn Upgrade() -> Element {
    let upgrade_step = use_signal(|| UpgradeStep::Edit);
    let amount_input = use_signal(|| "".to_string());
    let parsed_amount: u64 = match amount_input.read().parse::<f64>() {
        Ok(n) => (n * 10f64.powf(ore_api::consts::TOKEN_DECIMALS_V1.into())) as u64,
        Err(_) => 0,
    };

    let e = match *upgrade_step.read() {
        UpgradeStep::Edit => {
            rsx! {
                UpgradeEdit {
                    upgrade_step,
                    amount_input,
                    parsed_amount,
                }
            }
        }
        UpgradeStep::Confirm => {
            rsx! {
                UpgradeConfirm {
                    upgrade_step,
                    amount: parsed_amount,
                }
            }
        }
        UpgradeStep::Done(signature) => {
            rsx! {
                UpgradeDone {
                    signature,
                    amount: parsed_amount,
                }
            }
        }
    };

    e
}
