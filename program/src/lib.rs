use bonfida_utils::declare_id_with_central_state;

#[doc(hidden)]
pub mod entrypoint;
#[doc(hidden)]
pub mod error;
/// Program instructions and their CPI-compatible bindings
pub mod instruction;
/// Describes the different data structres that the program uses to encode state
pub mod state;

#[doc(hidden)]
pub(crate) mod processor;

#[allow(missing_docs)]
pub mod cpi;

pub mod utils;

declare_id_with_central_state!("6c4faDgogWngP5dFsj1AKFRDtn9zbsae3sgZPuaPfgLK");

#[cfg(not(feature = "no-entrypoint"))]
solana_security_txt::security_txt! {
    name: env!("CARGO_PKG_NAME"),
    project_url: "https://sns.id",
    contacts: "email:contact@sns.id,link:https://twitter.com/sns",
    policy: "https://immunefi.com/bounty/sns",
    preferred_languages: "en",
    auditors: "Halborn"
}
