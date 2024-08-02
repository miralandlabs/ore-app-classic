use dioxus::prelude::*;
use solana_client_wasm::solana_sdk::signature::Signature;
use solana_extra_wasm::program::spl_token::amount_to_ui_amount;

use crate::{components::{CheckCircleIcon, OreIcon}, route::Route};

#[component]
pub fn UpgradeDone(signature: Signature, amount: u64) -> Element {
    rsx! {
        div {
            class: "flex flex-col grow justify-between",
            div {
                class: "flex flex-col gap-3",
                h2 {
                    "Success!"
                }
                div {
                    class: "flex flex-row gap-2",
                    OreIcon {
                        class: "my-auto w-5 h-5"
                    }
                    p {
                        class: "text-2xl",
                        "{amount_to_ui_amount(amount, ore_api::consts::TOKEN_DECIMALS_V1)} has been upgraded from v1 to v2"
                    }
                }
            }
            div {
                class: "flex flex-col gap-8 w-full",
                CheckCircleIcon { class: "h-12 w-12 mx-auto" }
                // Link {
                //     class: "font-mono text-nowrap truncate mx-auto w-full p-2 rounded hover-100 active-200",
                //     to: Route::Tx {
                //         sig: signature.to_string(),
                //     },
                //     "{signature.to_string()}"
                // }
            }
        }
        div {
            class: "flex flex-col gap-3",
            div {
                class: "h-full"
            }
            Link {
                class: "w-full py-3 rounded font-semibold transition-colors text-center text-white bg-green-500 hover:bg-green-600 active:bg-green-700",
                to: Route::Home {},
                "Done"
            }
        }
    }
}
