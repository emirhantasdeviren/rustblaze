mod account;
pub mod bucket;
mod client;
pub mod file;

pub(crate) mod error;

#[doc(inline)]
pub use bucket::Bucket;
pub use client::Client;
pub use error::{Error, Result};

pub(crate) use account::Account;
