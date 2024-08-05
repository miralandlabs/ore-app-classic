use dioxus::prelude::*;
use solana_client_wasm::solana_sdk::native_token::LAMPORTS_PER_SOL;

use crate::{
    components::{try_start_mining, Spinner},
    hooks::{
        use_gateway, use_miner_toolbar_state, use_sol_balance, MinerStatus, MinerStatusMessage,
        UpdateMinerToolbarState,
    },
    miner::Miner,
    route::Route,
};

// MI, 1000=> 0.001SOL
pub const MIN_BALANCE: u64 = LAMPORTS_PER_SOL.saturating_div(1000);

#[component]
pub fn MinerToolbarActivating(miner: Signal<Miner>) -> Element {
    let nav = use_navigator();
    let gateway = use_gateway();
    let sol_balance = use_sol_balance();
    // let mut sufficient_balance = use_signal(|| true);
    let mut toolbar_state = use_miner_toolbar_state();

    // use_effect(move || {
    //     if let Some(Ok(sol_balance)) = *sol_balance.read() {
    //         sufficient_balance.set(sol_balance.ge(&MIN_BALANCE));
    //     } else {
    //         sufficient_balance.set(false);
    //     }
    // });

    let _ = use_resource(move || {
        let gateway = gateway.clone();
        async move {
            if let Some(Ok(balance)) = *sol_balance.read() {
                if balance.ge(&MIN_BALANCE) {
                    match try_start_mining(gateway, miner, &mut toolbar_state).await {
                        Ok(()) => {
                            toolbar_state.set_status(MinerStatus::Active);
                        }
                        Err(err) => {
                            log::error!("Failed to start mining: {:?}", err);
                            toolbar_state.set_status(MinerStatus::Error);
                            toolbar_state.set_status_message(MinerStatusMessage::Error);
                        }
                    }
                }
            }
        }
    });

    if let Some(Ok(balance)) = *sol_balance.read() {
        if balance.lt(&MIN_BALANCE) {
            nav.push(Route::Mine {});
        }
    }

    rsx! {
        div {
            class: "flex flex-row w-full justify-end my-auto px-4 sm:px-8",
            div {
                class: "flex w-10 h-10",
                Spinner {
                    class: "m-auto text-white"
                }
            }
        }
    }
}
