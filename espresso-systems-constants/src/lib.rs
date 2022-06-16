//! Constants used by the Espresso Systems company. This is stored in a central area to make sure we don't accidentally introduce duplicates.
//!
//! These are grouped by project. In the descendent projects you can re-export the module and use all the constants in there.
//!
//! ```ignore
//! // in CAP
//! use tagged_base64_espresso_systems_constants::cap;
//! 
//! #[tagged_blob(cap::ASSET_SEED)]
//! pub struct AssetCodeSeed(...);
//! ```

pub mod jellyfish {
    pub const VERKEY: &str = "VERKEY";
    pub const SIGNKEYPAIR: &str = "SIGNKEYPAIR";
    pub const SIG: &str = "SIG";
    pub const NODE: &str = "NODE";
    pub const LEAF: &str = "LEAF";
    pub const PROOF: &str = "PROOF";
    pub const BATCHPROOF: &str = "BATCHPROOF";
}

pub mod espresso {
    pub const TXN: &str = "TXN";
    pub const BLOCK: &str = "BLOCK";
    pub const STATE: &str = "STATE";
    pub const SET: &str = "SET";
    pub const SEED: &str = "SEED";
}

pub mod cape {
    pub const EADDR: &str = "EADDR";
    pub const ERC20: &str = "ERC20";
    pub const CMTMNT_CAPE_TRNSTN: &str = "CMTMNT_CAPE_TRNSTN";
}

pub mod cap {
    pub const INTERNAL_ASSET_CODE: &str = "INTERNAL_ASSET_CODE";
    pub const ASSET_SEED: &str = "ASSET_SEED";
    pub const ASSET_CODE: &str = "ASSET_CODE";
    pub const ASSET_DEF: &str = "ASSET_DEF";
    pub const BLIND: &str = "BLIND";
    pub const NUL: &str = "NUL";
    pub const REC: &str = "REC";
    pub const CRED: &str = "CRED";
    pub const ID: &str = "ID";
    pub const AUDMEMO: &str = "AUDMEMO";
    pub const RECMEMO: &str = "RECMEMO";
    pub const TXVERKEY: &str = "TXVERKEY";
    pub const USERPUBKEY: &str = "USERPUBKEY";
    pub const USERKEY: &str = "USERKEY";
    pub const CREDPUBKEY: &str = "CREDPUBKEY";
    pub const CREDKEY: &str = "CREDKEY";
    pub const AUDPUBKEY: &str = "AUDPUBKEY";
    pub const AUDKEY: &str = "AUDKEY";
    pub const FREEZEPUBKEY: &str = "FREEZEPUBKEY";
    pub const FREEZEKEY: &str = "FREEZEKEY";
    pub const NULKEY: &str = "NULKEY";
}

pub mod phaselock {
    pub const PEER_ID: &str = "PEER_ID";
}
