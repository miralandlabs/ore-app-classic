use std::borrow::BorrowMut;

use dioxus::prelude::*;
use solana_client_wasm::solana_sdk::pubkey::Pubkey;
use solana_extra_wasm::program::spl_token::amount_to_ui_amount;

use crate::{
    components::{BackButton, OreIcon, Spinner},
    gateway,
    hooks::{use_gateway, use_ore_balance, use_priority_fee, PriorityFee},
};

use super::SendStep;

#[component]
pub fn SendConfirm(
    send_step: Signal<SendStep>,
    amount: u64,
    recipient: Pubkey,
    memo: String,
) -> Element {
    let mut is_busy = use_signal(|| false);
    let mut priority_fee = use_priority_fee();
    let mut ore_balance = use_ore_balance();
    let gateway = use_gateway();

    use_future(move || async move {
        let price = gateway::get_recent_priority_fee_estimate(true).await + 20_000;
        priority_fee.set(PriorityFee(price));
    });

    rsx! {
        div {
            class: "flex flex-col h-full grow gap-12",
            div {
                class: "flex flex-col gap-2",
                BackButton {
                    onclick: move |_| {
                        send_step.borrow_mut().set(SendStep::Edit);
                    }
                }
                h2 {
                    "Confirm"
                }
                p {
                    class: "text-lg",
                    "Please review your transfer information for correctness."
                }
                p {
                    class: "text-sm text-gray-300",
                    "Once confirmed, this transaction cannot be undone."
                }
            }
            div {
                class: "flex flex-col gap-8",
                div {
                    class: "flex flex-col gap-2",
                    p {
                        "Amount"
                    }
                    div {
                        class: "flex flex-row gap-2",
                        OreIcon {
                            class: "my-auto w-5 h-5"
                        }
                        p {
                            class: "text-2xl",
                            "{amount_to_ui_amount(amount, ore_api::consts::TOKEN_DECIMALS)}"
                       }
                    }
                }
                div {
                    class: "flex flex-col gap-2",
                    p {
                        "To"
                    }
                    p {
                        class: "text-2xl",
                        "{recipient.to_string()}"
                    }
                }
                div {
                    class: "flex flex-col gap-2",
                    p {
                        "Memo"
                    }
                    p {
                        class: "text-2xl",
                        "{memo}"
                    }
                }
            }
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
                class: "flex flex-col mt-auto sm:flex-row gap-2",
                button {
                    class: "w-full py-3 rounded font-semibold transition-colors text-white bg-green-500 hover:bg-green-600 active:enabled:bg-green-700",
                    disabled: *is_busy.read(),
                    onclick: move |_| {
                        let gateway = gateway.clone();
                        let memo = memo.clone();
                        is_busy.set(true);
                        spawn(async move {
                            match gateway.transfer_ore(amount, recipient, memo, priority_fee.read().0).await {
                                Ok(sig) => {
                                    log::info!("Transfer: {:?}", sig);
                                    ore_balance.restart();
                                    is_busy.set(false);
                                    send_step.set(SendStep::Done);
                                }
                                Err(err) => {
                                    // TODO Handle error
                                    is_busy.set(false);
                                    log::error!("Failed to send: {:?}", err);
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
