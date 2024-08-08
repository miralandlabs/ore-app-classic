use dioxus::prelude::*;
// use ore_relayer_api::state::Escrow;
use solana_extra_wasm::program::spl_token::amount_to_ui_amount;

use crate::{
    components::{
        BackButton, MinerToolbarTopUpOpen,
        OreIcon, Spinner, MIN_BALANCE,
    },
    gateway,
    hooks::{
        use_gateway, use_miner_toolbar_state, use_power_level, use_priority_fee,
        use_proof, use_sol_balance, MinerStatus, MinerStatusMessage, PowerLevel, PriorityFee, ReadMinerToolbarState,
    },
    miner::WEB_WORKERS,
};

// TODO Activity history of hashes
// TODO Display for non-active states
// TODO Stop start button

pub fn Mine() -> Element {
    let sol_balance = use_sol_balance();
    let toolbar_state = use_miner_toolbar_state();
    let nav = use_navigator();

    if let Some(Ok(balance)) = *sol_balance.read() {
        if balance.lt(&MIN_BALANCE) {
            return rsx! {
                MinerToolbarTopUpOpen {}
            };
        }
    }

    // MI
    let mut priority_fee = use_priority_fee();
    // let gateway = use_gateway();
    use_future(move || async move {
        let price = gateway::get_recent_priority_fee_estimate(true).await + 20_000;
        priority_fee.set(PriorityFee(price));
    });

    rsx! {
        div {
            class: "flex flex-col gap-8 overflow-visible",
            div {
                class: "flex flex-col gap-4 -mt-3.5 mb-4",
                BackButton {
                    onclick: move |_| {
                        nav.go_back()
                    }
                }
                div {
                    class: "flex flex-col gap-2",
                    h2 {
                        "Miner"
                    }
                    match toolbar_state.status() {
                        MinerStatus::Active => {
                            rsx! {
                                match toolbar_state.status_message() {
                                    MinerStatusMessage::Searching => {
                                        rsx! {
                                            p {
                                                class: "text-lg text-white",
                                                "Searching for valid hashes... "
                                                // if time_remaining.read().gt(&0) {
                                                //     "({time_remaining} sec)"
                                                // }
                                            }
                                        }
                                    }
                                    MinerStatusMessage::Submitting => {
                                        rsx! {
                                            div {
                                                class: "flex flex-row gap-2",
                                                p {
                                                    class: "text-lg text-white",
                                                    "Submitting best hash..."
                                                }
                                                Spinner {
                                                    class: "my-auto"
                                                }
                                            }
                                        }
                                    }
                                    MinerStatusMessage::Error => {
                                        rsx! {
                                            p {
                                                class: "text-lg text-white",
                                                "Error submitting transaction"
                                            }
                                        }
                                    }
                                }
                                match toolbar_state.status_message() {
                                    MinerStatusMessage::Searching | MinerStatusMessage::Submitting => {
                                        rsx! {
                                            p {
                                                class: "font-mono text-sm truncate shrink text-gray-300",
                                                "{toolbar_state.display_hash()}"
                                            }
                                        }
                                    }
                                    _ => rsx! {}
                                }
                            }
                        }
                        _ => { rsx! {} },
                    }
                }
            }
            StakeBalanceDisplay {}
            MultiplierDisplay {}
            PowerLevelConfig {}
            PriorityFeeConfig {}
            // DownloadLink {}

        }
    }
}

pub fn StakeBalanceDisplay() -> Element {
    let mut proof = use_proof();

    use_future(move || async move {
        loop {
            async_std::task::sleep(std::time::Duration::from_secs(3)).await;
            proof.restart();
        }
    });

    rsx! {
            div {
                class: "flex flex-row gap-8 justify-between",
                    p {
                        class: "text-gray-300 font-medium text-sm my-auto",
                        "Stake"
                    }
               div {
                    class: "flex flex-row flex-shrink h-min gap-1 shrink mb-auto",
                    if let Some(proof) = *proof.read() {
                        if let Ok(proof) = proof {
                            div {
                                class: "flex flex-row gap-2",
                                OreIcon {
                                    class: "my-auto w-4 h-4"
                                }
                                p {
                                    class: "font-semibold",
                                    "{amount_to_ui_amount(proof.balance, ore_api::consts::TOKEN_DECIMALS)}"
                                }
                            }
                        } else {
                            div {
                                class: "flex flex-row gap-2",
                                OreIcon {
                                    class: "my-auto w-4 h-4"
                                }
                                p {
                                    class: "font-semibold",
                                    "0"
                                }
                            }
                        }
                    } else {
                        div {
                            class: "flex flex-row w-32 h-8 grow loading rounded",
                        }
                    }
                }
            }
    }
}

pub fn MultiplierDisplay() -> Element {
    let proof = use_proof();

    let multiplier = use_resource(move || async move {
        let gateway = use_gateway();
        if let Some(Ok(proof)) = *proof.read() {
            if let Ok(config) = gateway.get_config().await {
                return 1.0 + (proof.balance as f64 / config.top_balance as f64).min(1.0f64);
            }
        }
        1.0
    });

    rsx! {
            div {
                class: "flex flex-row gap-8 justify-between",
                    p {
                        class: "text-gray-300 font-medium text-sm my-auto",
                        "Multiplier"
                    }
               div {
                    class: "flex flex-row flex-shrink h-min gap-1 shrink mb-auto",
                    p {
                        class: "dark:text-white text-right px-1 mb-auto font-semibold",
                        "{multiplier.read().unwrap_or(1.0):.12}x"
                    }
                }
            }
            // p {
            //     class: "text-white text-xs opacity-80 max-w-96",
            //     "The multiplier you are earning on your mining rewards from staking."
            // }
        // }
    }
}

pub fn PowerLevelConfig() -> Element {
    let mut power_level = use_power_level();
    let max = *WEB_WORKERS as i64;

    rsx! {
        // div {
        //     class: "flex flex-col gap-2",
            div {
                class: "flex flex-row gap-8 justify-between",
                    p {
                        class: "text-gray-300 font-medium text-sm my-auto",
                        "Power"
                    }
                div {
                    class: "flex flex-row flex-shrink h-min gap-1 shrink mb-auto",
                    input {
                        class: "bg-transparent dark:text-white text-right px-1 mb-auto rounded font-semibold hover:bg-green-600 transition-colors",
                        dir: "rtl",
                        step: 1,
                        min: 1,
                        max: max,
                        r#type: "number",
                        value: "{power_level.read().0}",
                        oninput: move |e| {
                            if let Ok(v) = e.value().parse::<u64>() {
                                power_level.set(PowerLevel(v));
                            }
                        }
                    }
                    p {
                        class: "my-auto font-semibold",
                        "of {max} cores"
                    }
                }
            }
            // p {
            //     class: "text-white text-xs opacity-80 max-w-96",
            //     "The number of computer cores you have dedicated to mining."
            // }
        // }
    }
}

pub fn PriorityFeeConfig() -> Element {
    let mut priority_fee = use_priority_fee();

    rsx! {
        div {
            class: "flex flex-row gap-8 justify-between",
            div {
                class: "flex flex-col gap-1",
                p {
                    class: "text-gray-300 font-medium text-sm my-auto",
                    "Priority fee(with initial recommendation)"
                }
                p {
                    class: "text-gray-300 text-xs opacity-80 max-w-96",
                    "Add a priority fee to increase your chances of landing a transaction during blockchain congestion."
                }
           }
           div {
                class: "flex flex-row flex-shrink h-min gap-1 shrink mb-auto",
                input {
                    class: "bg-transparent dark:text-white text-right px-1 mb-auto rounded font-semibold hover:bg-green-600 transition-colors",
                    dir: "rtl",
                    step: 100_000,
                    min: 0,
                    max: 10_000_000,
                    r#type: "number",
                    value: "{priority_fee.read().0}",
                    oninput: move |e| {
                        if let Ok(v) = e.value().parse::<u64>() {
                            priority_fee.set(PriorityFee(v));
                        }
                    }
                }
                p {
                    class: "my-auto font-semibold",
                    "microlamports"
                }
            }
        }
    }
}

// fn DownloadLink() -> Element {
//     // if cfg!(feature = "web") {
//     //     rsx! {
//     //         div {
//     //             class: "flex flex-row gap-2 mt-8 p-2.5 rounded bg-green-600",
//     //             WarningIcon {
//     //                 class: "w-4 h-4 mt-0.5 shrink-0"
//     //             }
//     //             p {
//     //                 class: "text-sm my-auto",
//     //                 "You are mining from a web browser. For better performance, "
//     //                 Link {
//     //                     to: Route::Download {},
//     //                     class: "font-medium underline",
//     //                     "download the app."
//     //                 }
//     //             }
//     //         }
//     //     }
//     // } else {
//     //     None
//     // }
//     rsx! {}
// }
