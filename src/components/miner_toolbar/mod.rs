mod activating;
mod active;
mod error;
mod insufficient_sol;
mod layout;
mod not_started;
mod utils;

pub use activating::*;
pub use active::*;
pub use error::*;
pub use insufficient_sol::*;
pub use layout::*;
pub use not_started::*;
use ore_relayer_api::consts::ESCROW;
use solana_sdk::pubkey::Pubkey;
pub use utils::*;

use dioxus::prelude::*;

use crate::hooks::{
    use_gateway, use_miner, use_miner_toolbar_state,
    use_wallet_adapter::{use_wallet_adapter, WalletAdapter, RELAYER_PUBKEY},
    MinerStatus, ReadMinerToolbarState, UpdateMinerToolbarState,
};

#[component]
pub fn MinerToolbar(hidden: bool) -> Element {
    let mut toolbar_state = use_miner_toolbar_state();
    let wallet_adapter = use_wallet_adapter();
    let miner = use_miner();
    let gateway = use_gateway();

    use_future(move || {
        let gateway = gateway.clone();
        async move {
            match *wallet_adapter.read() {
                WalletAdapter::Disconnected => {
                    log::info!("Disconnected");
                }
                WalletAdapter::Connected(pubkey) => {
                    log::info!("Pub: {:?}", pubkey);
                    if let Ok(escrow) = gateway.get_escrow(pubkey).await {
                        log::info!("EScrow: {:?}", escrow);
                        let escrow_address = Pubkey::find_program_address(
                            &[ESCROW, pubkey.as_ref(), RELAYER_PUBKEY.as_ref()],
                            &ore_relayer_api::id(),
                        )
                        .0;
                        toolbar_state.set_escrow_address(escrow_address);
                    } else {
                        log::info!("No escrow");
                    }
                }
            }
        }
    });

    let class =
        "fixed transition-height transition-colors flex flex-row justify-between inset-x-0 bottom-0 drop-shadow-md";
    let height = if toolbar_state.is_open() {
        "max-h-[80vh] shrink overflow-y-scroll"
    } else {
        "h-16 cursor-pointer"
    };

    let bg = match toolbar_state.status() {
        MinerStatus::Active => "bg-green-500 text-white",
        MinerStatus::Error => "bg-red-500 text-white",
        MinerStatus::NotStarted => {
            if toolbar_state.is_open() {
                "bg-white dark:bg-gray-900"
            } else {
                "bg-gray-100 dark:bg-gray-900"
            }
        }
        _ => "bg-gray-100 dark:bg-gray-900",
    };

    let display = if hidden { "hidden" } else { "" };

    if let WalletAdapter::Disconnected = *wallet_adapter.read() {
        return None;
    }

    rsx! {
        div {
            class: "{class} {height} {bg} {display}",
            onclick: move |e| {
                toolbar_state.set_is_open(true);
                e.stop_propagation();
            },
            div {
                class: "flex flex-row justify-between w-full max-w-[96rem] mx-auto h-full",
                match toolbar_state.status() {
                    MinerStatus::NotStarted => {
                        rsx! {
                            MinerToolbarNotStarted {}
                        }
                    }
                    MinerStatus::Activating => {
                        rsx! {
                            MinerToolbarActivating { miner }
                        }
                    }
                    MinerStatus::Active => {
                        rsx! {
                            MinerToolbarActive { miner }
                        }
                    }
                    MinerStatus::Error => {
                        rsx! {
                            MinerToolbarError {}
                        }
                    }
                }
            }
        }
    }
}
