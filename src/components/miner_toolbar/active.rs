use dioxus::prelude::*;
use solana_client_wasm::solana_sdk::blake3::Hash as Blake3Hash;

use crate::{
    components::StopButton,
    hooks::{
        use_miner_toolbar_state, MinerStatusMessage, ReadMinerToolbarState, UpdateMinerToolbarState,
    },
    miner::Miner,
};

#[component]
pub fn MinerToolbarActive(miner: Signal<Miner>) -> Element {
    let mut toolbar_state = use_miner_toolbar_state();

    // Animate the hash in the miner toolbar to visualize mining.
    use_future(move || async move {
        loop {
            async_std::task::sleep(std::time::Duration::from_millis(125)).await;
            if let MinerStatusMessage::Searching = toolbar_state.status_message() {
                toolbar_state.set_display_hash(Blake3Hash::new_unique());
            }
        }
    });

    rsx! {
        div {
            class: "flex flex-row gap-2 max-w-screen w-screen justify-between my-auto px-4 sm:px-8",
            div {
                class: "flex flex-row gap-2 flex-shrink flex-auto truncate my-auto",
                match toolbar_state.status_message() {
                    MinerStatusMessage::Searching => {
                        rsx! {
                            p {
                                class: "font-mono text-sm truncate flex-shrink flex-auto opacity-80 my-auto ml-2",
                                "{toolbar_state.display_hash()}"
                            }
                        }
                    }
                    MinerStatusMessage::Submitting(attempts, priority_fee) => {
                        rsx! {
                            p {
                                class: "truncate flex-shrink flex-auto text-sm text-white font-medium opacity-80 my-auto ml-2",
                                "Submitting transaction... (attempt {attempts}, priority fee {priority_fee})"
                            }
                        }
                    }
                    MinerStatusMessage::Error => {
                        rsx! {
                            p {
                                class: "truncate flex-shrink flex-auto text-sm text-white opacity-80 my-auto ml-2",
                                "Error submitting transaction"
                            }
                        }
                    }
                }
            }
            div {
                class: "flex-shrink-0 flex-none flex flex-row gap-2 ml-auto my-auto",
                // p {
                //     class: "font-semibold text-white flex-shrink-0 flex-none my-auto",
                //     "Mining"
                // }
                StopButton {}
            }
        }
    }
}
