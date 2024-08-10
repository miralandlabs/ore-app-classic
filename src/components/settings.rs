use std::str::FromStr;

use dioxus::prelude::*;
use is_url::is_url;
use solana_client_wasm::solana_sdk::native_token::lamports_to_sol;

use crate::{
    components::{Appearance, BackupKeypairWarning, Copyable},
    gateway::{FEE_URL, PRIORITY_FEE_CAP, RPC_URL},
    hooks::{
        use_appearance, use_explorer, use_pubkey, use_fee_url, use_rpc_url, use_priority_fee_cap,
        use_show_backup_warning, use_sol_balance, Explorer, FeeUrl, PriorityFeeCap, RpcUrl,
    },
    route::Route,
};

pub fn Settings() -> Element {
    let mut explorer = use_explorer();
    let mut appearance = use_appearance();
    let show_backup_warning = use_show_backup_warning();
    let pubkey = use_pubkey();
    let sol_balance = use_sol_balance();

    let mut rpc_url = use_rpc_url();
    let mut rpc_url_input = use_signal(|| rpc_url.read().0.clone());
    let mut rpc_url_error = use_signal::<Option<String>>(|| None);
    let is_rpc_url_edited = rpc_url.read().0.ne(&*rpc_url_input.read());

    // MI
    let mut fee_url = use_fee_url();
    let mut fee_url_input = use_signal(|| fee_url.read().0.clone());
    let mut fee_url_error = use_signal::<Option<String>>(|| None);
    let is_fee_url_edited = fee_url.read().0.ne(&*fee_url_input.read());

    // MI
    let mut priority_fee_cap = use_priority_fee_cap();
    let mut priority_fee_cap_input = use_signal(|| priority_fee_cap.read().0.clone());
    let mut priority_fee_cap_error = use_signal::<Option<u64>>(|| None);
    let is_priority_fee_cap_edited = priority_fee_cap.read().0.ne(&*priority_fee_cap_input.read());

    let container_class = "flex flex-row gap-8 justify-between w-full sm:px-1";
    let section_title_class = "text-lg md:text-2xl font-bold";
    let data_title_class = "font-medium text-sm opacity-50 my-auto";

    rsx! {
        div {
            class: "flex flex-col gap-16 w-full pb-24",
            div {
                class: "flex flex-col gap-4 w-full",
                h2 {
                    "Settings"
                }
                if cfg!(feature = "web") && show_backup_warning.read().0 {
                    div {
                        class: "mt-8",
                        BackupKeypairWarning {}
                    }
                }
                h2 {
                    class: "{section_title_class} mt-8",
                    "Account"
                }
                div {
                    class: "{container_class}",
                    p {
                        class: "{data_title_class}",
                        "Address"
                    }
                    Copyable {
                        value: pubkey.to_string(),
                        Link {
                            class: "font-mono sm:px-2 py-1 rounded hover-100 active-200 transition-colors truncate font-medium",
                            to: Route::User {
                                id: pubkey.to_string()
                            },
                            "{pubkey}"
                        }
                    }
                }
                div {
                    class: "{container_class}",
                    p {
                        class: "{data_title_class}",
                        "Balance"
                    }
                    if let Some(Ok(balance)) = *sol_balance.read() {
                        p {
                            "{lamports_to_sol(balance)} SOL"
                        }
                    } else {
                        div {
                            class: "flex w-32 loading rounded",
                        }
                    }
                }
                div {
                    class: "{container_class}",
                    p {
                        class: "{data_title_class}",
                        "Keypair"
                    }
                    div {
                        class: "flex flex-row gap-2 -mr-2",
                        Link {
                            to: Route::ImportKey {},
                            class: "font-semibold hover-100 active-200 transition-colors px-4 py-1 rounded",
                            "Import"
                        }
                        Link {
                            to: Route::ExportKey {},
                            class: "font-semibold hover-100 active-200 transition-colors px-4 py-1 rounded",
                            "Export"
                        }
                    }
                }
            }
            div {
                class: "flex flex-col gap-4",
                h2 {
                    class: "{section_title_class}",
                    "Display"
                }
                div {
                    class: "{container_class}",
                    p {
                        class: "{data_title_class}",
                        "Appearance"
                    }
                    select {
                        class: "text-right bg-transparent dark:text-white hover:cursor-pointer py-1",
                        onchange: move |e| {
                            if let Ok(a) = Appearance::from_str(&e.value()) {
                                appearance.set(a);
                            }
                        },
                        option { initial_selected: appearance.read().eq(&Appearance::Dark), value: "{Appearance::Dark}", "{Appearance::Dark}" }
                        option { initial_selected: appearance.read().eq(&Appearance::Light), value: "{Appearance::Light}", "{Appearance::Light}" }
                    }
                }
                div {
                    class: "{container_class}",
                    p {
                        class: "{data_title_class}",
                        "Explorer"
                    }
                    select {
                        class: "text-right bg-transparent dark:text-white hover:cursor-pointer py-1",
                        onchange: move |e| {
                            if let Ok(e) = Explorer::from_str(&e.value()) {
                                explorer.set(e);
                            }
                        },
                        option { initial_selected: explorer.read().eq(&Explorer::Solana), value: "{Explorer::Solana}", "{Explorer::Solana}" }
                        option { initial_selected: explorer.read().eq(&Explorer::SolanaFm), value: "{Explorer::SolanaFm}", "{Explorer::SolanaFm}" }
                        option { initial_selected: explorer.read().eq(&Explorer::Solscan), value: "{Explorer::Solscan}", "{Explorer::Solscan}" }
                        option { initial_selected: explorer.read().eq(&Explorer::Xray), value: "{Explorer::Xray}", "{Explorer::Xray}" }
                    }
                }
            }
            div {
                class: "flex flex-col gap-4",
                h2 {
                    class: "{section_title_class}",
                    "Network"
                }
                div {
                    class: "flex flex-col gap-2",
                    div {
                        class: "{container_class} flex-auto",
                        div {
                            p {
                                class: "{data_title_class}",
                                "RPC"
                            }
                            p {
                                class: "text-left text-orange-500 max-w-144",
                                "The default rpc charges 0.0001SOL per transaction as tip. You can use your own rpc to reduce the tip by half to 0.00005SOL per transaction."
                            }
                            // ul {
                            //     class: "text-left text-sm text-red-500",
                            //     li {
                            //         "The default rpc will charge 0.0001SOL per transaction as tip. "
                            //     }
                            //     li {
                            //         "Instead you can replace it with your own rpc to save cost. "
                            //     }
                            // }
                        }
                        div {
                            class: "flex flex-auto flex-col gap-2",
                            input {
                                autofocus: false,
                                class: "w-full text-right placeholder-gray-300 dark:placeholder-gray-800 bg-transparent",
                                value: "{rpc_url_input}",
                                placeholder: "{RPC_URL}",
                                oninput: move |evt| {
                                    let s = evt.value();
                                    rpc_url_input.set(s.clone());
                                    if !is_url(&s) {
                                        rpc_url_error.set(Some("Invalid url".to_string()));
                                    } else {
                                        rpc_url_error.set(None);
                                    }
                                },
                            }
                            // MI
                            div {
                                class: "flex flex-shrink gap-2 justify-end",
                                if let Some(err_str) = rpc_url_error.read().clone() {
                                    p {
                                        class: "text-sm text-red-500 text-right",
                                        "{err_str}"
                                    }
                                }
                                div {
                                    class: "flex flex-row gap-2",
                                    if rpc_url.read().0.ne(RPC_URL) {
                                        button {
                                            class: "hover-100 active-200 rounded shrink ml-auto transition-colors px-2 py-1 font-semibold",
                                            onclick: move |_| {
                                                rpc_url.set(RpcUrl(RPC_URL.to_string()));
                                                rpc_url_input.set(RPC_URL.to_string());
                                                rpc_url_error.set(None);
                                            },
                                            "Reset to default"
                                        }
                                    }
                                    if is_rpc_url_edited && rpc_url_error.read().is_none() {
                                        button {
                                            class: "bg-green-500 hover:bg-green-600 active:bg-green-700 text-white rounded shrink ml-auto transition-colors px-2 py-1",
                                            onclick: move |_| {
                                                rpc_url.set(RpcUrl(rpc_url_input.read().clone()));
                                            },
                                            "Save"
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // MI
                    div {
                        class: "{container_class} flex-auto",
                        div {
                            p {
                                class: "{data_title_class}",
                                "Priority Fee Estimate URL"
                            }
                            p {
                                class: "text-left dark:text-white max-w-144",
                                "This url is free for now and provided as-is. You are encouraged to use your own fee estimation url instead."
                            }
                        }
                        div {
                            class: "flex flex-auto flex-col gap-2",
                            input {
                                autofocus: false,
                                class: "w-full text-right placeholder-gray-300 dark:placeholder-gray-800 bg-transparent",
                                value: "{fee_url_input}",
                                placeholder: "{FEE_URL}",
                                oninput: move |evt| {
                                    let s = evt.value();
                                    fee_url_input.set(s.clone());
                                    if !is_url(&s) {
                                        fee_url_error.set(Some("Invalid url".to_string()));
                                    } else {
                                        fee_url_error.set(None);
                                    }
                                },
                            }
                            div {
                                class: "flex flex-shrink gap-2 justify-end",
                                if let Some(err_str) = fee_url_error.read().clone() {
                                    p {
                                        class: "text-sm text-red-500 text-right",
                                        "{err_str}"
                                    }
                                }
                                div {
                                    class: "flex flex-row gap-2",
                                    if fee_url.read().0.ne(FEE_URL) {
                                        button {
                                            class: "hover-100 active-200 rounded shrink ml-auto transition-colors px-2 py-1 font-semibold",
                                            onclick: move |_| {
                                                fee_url.set(FeeUrl(FEE_URL.to_string()));
                                                fee_url_input.set(FEE_URL.to_string());
                                                fee_url_error.set(None);
                                            },
                                            "Reset to default"
                                        }
                                    }
                                    if is_fee_url_edited && fee_url_error.read().is_none() {
                                        button {
                                            class: "bg-green-500 hover:bg-green-600 active:bg-green-700 text-white rounded shrink ml-auto transition-colors px-2 py-1",
                                            onclick: move |_| {
                                                fee_url.set(FeeUrl(fee_url_input.read().clone()));
                                            },
                                            "Save"
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div {
                        class: "{container_class} flex-auto",
                        div {
                            p {
                                class: "{data_title_class}",
                                "Priority Fee Cap"
                            }
                            p {
                                class: "text-left dark:text-white max-w-144",
                                "You can set your own priority fee max value."
                            }
                        }
                        div {
                            class: "flex flex-auto flex-col gap-2",
                            div {
                                class: "flex flex-row flex-shrink h-min gap-1 shrink mb-auto",
                                input {
                                    class: "bg-transparent disabled:opacity-50 dark:text-white text-right px-1 mb-auto rounded font-semibold hover:bg-green-600 transition-colors",
                                    dir: "rtl",
                                    step: 100_000,
                                    min: 0,
                                    max: 10_000_000,
                                    r#type: "number",
                                    value: "{priority_fee_cap.read().0}",
                                    oninput: move |e| {
                                        if let Ok(v) = e.value().parse::<u64>() {
                                            priority_fee_cap.set(PriorityFeeCap(v));
                                        }
                                    }
                                }
                                p {
                                    class: "my-auto font-semibold",
                                    "microlamports"
                                }
                            }
                            div {
                                class: "flex flex-shrink gap-2 justify-end",
                                if let Some(err_str) = priority_fee_cap_error.read().clone() {
                                    p {
                                        class: "text-sm text-red-500 text-right",
                                        "{err_str}"
                                    }
                                }
                                div {
                                    class: "flex flex-row gap-2",
                                    if priority_fee_cap.read().0.ne(&PRIORITY_FEE_CAP) {
                                        button {
                                            class: "hover-100 active-200 rounded shrink ml-auto transition-colors px-2 py-1 font-semibold",
                                            onclick: move |_| {
                                                priority_fee_cap.set(PriorityFeeCap(PRIORITY_FEE_CAP));
                                                priority_fee_cap_input.set(PRIORITY_FEE_CAP);
                                                priority_fee_cap_error.set(None);
                                            },
                                            "Reset to default"
                                        }
                                    }
                                    if is_priority_fee_cap_edited && priority_fee_cap_error.read().is_none() {
                                        button {
                                            class: "bg-green-500 hover:bg-green-600 active:bg-green-700 text-white rounded shrink ml-auto transition-colors px-2 py-1",
                                            onclick: move |_| {
                                                priority_fee_cap.set(PriorityFeeCap(priority_fee_cap_input.read().clone()));
                                            },
                                            "Save"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
