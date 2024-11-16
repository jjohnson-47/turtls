use crate::{cipher_suites::CipherList, extensions::Extensions};

#[repr(C)]
pub struct Config {
    /// The timeout in milliseconds to use for record layer reads during the handshake.
    ///
    /// Default value: `10000`
    pub timeout_millis: u64,
    /// The extensions to use.
    pub extensions: Extensions,
    /// The cipher suites to use.
    pub cipher_suites: CipherList,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            timeout_millis: 10_000,
            extensions: Extensions::default(),
            cipher_suites: CipherList::default(),
        }
    }
}
