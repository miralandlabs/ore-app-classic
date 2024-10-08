use dioxus::prelude::*;

use crate::{
    hooks::{use_show_backup_warning, ShowBackupWarning},
    route::Route,
};

pub fn BackupKeypairWarning() -> Element {
    let mut show_backup_warning = use_show_backup_warning();
    rsx! {
        div {
            class: "flex flex-col gap-3 bg-orange-500 w-full rounded px-4 py-5 text-white",
            p {
                class: "font-bold text-2xl",
                "ATTENTION: Before funding with SOL, please export your key pair for backup!"
            }
            ul {
                class: "list-disc list-outside pl-4 space-y-1.5",
                li {
                    "Your miner keypair is stored in your browser's local storage. "
                }
                li {
                    "Clearing cookies can delete your keypair, leaving your funds unrecoverable. "
                }
                li {
                    Link {
                        to: Route::ExportKey {},
                        class: "font-bold text-blue-300 hover:underline",
                        "ACTION: Export My Keypair"
                    }
                    " and back it up somewhere safe."
                }
            }
            div {
                class: "flex flex-row justify-end",
                button {
                    onclick: move |_| {
                        show_backup_warning.set(ShowBackupWarning(false));
                    },
                    class: "flex flex-row gap-2 hover:bg-orange-600 active:bg-orange-700 text-white px-3 py-2 rounded font-semibold transition-colors",
                    // EyeSlashIcon {
                    //     class: "w-4 h-4 my-auto"
                    // }
                    "I have backed up my keypair"
                }
            }
        }
    }
}
