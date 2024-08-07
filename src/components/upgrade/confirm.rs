use std::borrow::BorrowMut;

use dioxus::prelude::*;
use solana_extra_wasm::program::spl_token::amount_to_ui_amount;

use crate::{
    components::{BackButton, OreIcon, Spinner},
    gateway,
    hooks::{use_gateway, use_ore_balance, use_priority_fee, use_proof, PriorityFee},
};

use super::UpgradeStep;

#[component]
pub fn UpgradeConfirm(amount: u64, upgrade_step: Signal<UpgradeStep>) -> Element {
    let mut is_busy = use_signal(|| false);
    let mut priority_fee = use_priority_fee();
    let mut balance = use_ore_balance();
    let mut proof = use_proof();
    let gateway = use_gateway();
    // let price = gateway::get_recent_priority_fee_estimate(true).await + 20_000;
    let price = use_resource(move || { async move {
            let p = gateway::get_recent_priority_fee_estimate(true).await + 20_000;
            Some(p)
        }
    });
    priority_fee.set(PriorityFee(price.unwrap().unwrap_or(0)));

    rsx! {
        div {
            class: "flex flex-col h-full grow justify-between",
            div {
                class: "flex flex-col gap-4 -mt-3.5 mb-4",
                BackButton {
                    onclick: move |_| {
                        upgrade_step.borrow_mut().set(UpgradeStep::Edit);
                    }
                }
                div {
                    class: "flex flex-col gap-2",
                    h2 {
                        "Confirm"
                    }
                    p {
                        class: "text-lg",
                        "Please review your upgrade information for correctness."
                    }
                    p {
                        class: "text-sm text-gray-300",
                        "Once confirmed, this transaction cannot be undone."
                    }
                }
            }
            div {
                class: "flex flex-col gap-8",
                div {
                    class: "flex flex-col gap-2",
                    p { "Upgrade" }
                    div {
                        class: "flex flex-row gap-4",
                        div {
                            p {
                                class: "font-medium text-2xl",
                                "{amount_to_ui_amount(amount, ore_api::consts::TOKEN_DECIMALS_V1)} OREv1"
                            }
                        }
                        p {
                            class: "text-2xl",
                            "→"
                        }
                        div {
                            class: "flex flex-row gap-2",
                            OreIcon { class: "my-auto w-5 h-5" }
                            p {
                                class: "font-medium text-2xl",
                                "{amount_to_ui_amount(amount, ore_api::consts::TOKEN_DECIMALS_V1)} ORE"
                            }
                        }
                    }
                }
            }
            div {
                class: "flex flex-col gap-8",
                div {
                    class: "flex flex-row gap-8 justify-between mt-8",
                    div {
                        class: "flex flex-col gap-1",
                        p {
                            class: "font-semibold",
                            "Priority fee(with initial recommendation)"
                        }
                        p {
                            class: "text-xs opacity-80 max-w-96",
                            "Add a priority fee to increase your chances of landing a transaction only during blockchain congestion."
                        }
                    }
                    div {
                        class: "flex flex-row flex-shrink h-min gap-1 shrink mb-auto",
                        input {
                            disabled: *is_busy.read(),
                            class: "bg-transparent text-right px-1 mb-auto font-semibold",
                            dir: "rtl",
                            step: 100_000,
                            min: 0,
                            max: 50_000_000,
                            r#type: "number",
                            value: "{priority_fee.read().0}",
                            oninput: move |e| {
                                if let Ok(v) = e.value().parse::<u64>() {
                                    priority_fee.set(PriorityFee(v));
                                }
                            }
                        }
                        p {
                            class: "my-auto",
                            "microlamports"
                        }
                    }
                }
                div {
                    class: "flex flex-col sm:flex-row gap-2",
                    button {
                        class: "w-full py-3 rounded font-semibold transition-colors text-white bg-green-500 hover:bg-green-600 active:enabled:bg-green-700",
                        disabled: *is_busy.read(),
                        onclick: move |_| {
                            is_busy.set(true);
                            let gateway = gateway.clone();
                            spawn({
                                async move {

                                    // Upgrade
                                    match gateway.upgrade_ore(amount, priority_fee.read().0).await {
                                        Ok(sig) => {
                                            balance.restart();
                                            proof.restart();
                                            is_busy.set(false);
                                            upgrade_step.set(UpgradeStep::Done(sig));
                                        }
                                        Err(_err) => {
                                            // TODO Handle error
                                            is_busy.set(false);
                                            log::error!("Failed to upgrade!");
                                        }
                                    }
                                }
                            });
                        },
                        if *is_busy.read() {
                            Spinner {
                                class: "mx-auto"
                            }
                        } else {
                            "Confirm"
                        }
                    }
                }
            }
        }
    }
}
