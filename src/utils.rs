use cached::proc_macro::cached;
use ore_api::consts::PROOF;
use solana_client_wasm::solana_sdk::pubkey::Pubkey;

pub fn asset_path(relative_path: &str) -> String {
    relative_path.to_string()
}

#[cached]
pub fn proof_pubkey(authority: Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[PROOF, authority.as_ref()], &ore_api::ID).0
}