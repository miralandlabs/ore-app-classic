use dioxus::prelude::*;

use crate::{
    components::PlayIcon,
    hooks::{use_miner_toolbar_state, UpdateMinerToolbarState},
};

pub fn MinerToolbarNotStarted() -> Element {
    rsx! {
        div {
            class: "flex flex-row w-full justify-end my-auto px-4 sm:px-8",
            div {
                class: "flex flex-row gap-2 sm:gap-4",
                StartButton {}
            }
        }
    }
}

pub fn StartButton() -> Element {
    let mut toolbar_state = use_miner_toolbar_state();
    rsx! {
        button {
            class: "transition transition-colors flex flex-row gap-2 w-full h-10 px-4 justify-center rounded-full bg-green-500 text-white hover:bg-green-600 active:bg-green-700 text-nowrap",
            onclick: move |e| {
                toolbar_state.start();
                e.stop_propagation();
            },
            PlayIcon {
                class: "w-6 h-6 my-auto"
            }
            p {
                class: "my-auto font-semibold",
                "Mine"
            }
        }
    }
}
