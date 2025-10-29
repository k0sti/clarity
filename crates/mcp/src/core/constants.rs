//! ContextVM protocol constants

/// ContextVM messages (ephemeral events)
pub const CTXVM_MESSAGES_KIND: u16 = 25910;

/// Encrypted messages using NIP-59 Gift Wrap
pub const GIFT_WRAP_KIND: u16 = 1059;

/// Server announcement (addressable)
pub const SERVER_ANNOUNCEMENT_KIND: u16 = 11316;

/// Tools list (addressable)
pub const TOOLS_LIST_KIND: u16 = 11317;

/// Resources list (addressable)
pub const RESOURCES_LIST_KIND: u16 = 11318;

/// Resource templates list (addressable)
pub const RESOURCETEMPLATES_LIST_KIND: u16 = 11319;

/// Prompts list (addressable)
pub const PROMPTS_LIST_KIND: u16 = 11320;

/// Nostr tag constants
pub mod tags {
    /// Public key tag
    pub const PUBKEY: &str = "p";

    /// Event ID tag for correlation
    pub const EVENT_ID: &str = "e";

    /// Capability tag for pricing metadata
    pub const CAPABILITY: &str = "cap";

    /// Name tag for server announcements
    pub const NAME: &str = "name";

    /// Website tag for server announcements
    pub const WEBSITE: &str = "website";

    /// Picture tag for server announcements
    pub const PICTURE: &str = "picture";

    /// About tag for server announcements
    pub const ABOUT: &str = "about";

    /// Support encryption tag
    pub const SUPPORT_ENCRYPTION: &str = "support_encryption";
}

/// Maximum message size (1MB)
pub const MAX_MESSAGE_SIZE: usize = 1024 * 1024;

/// NIP-44 salt for HKDF
pub const NIP44_SALT: &str = "nip44-v2";

/// NIP-44 version byte
pub const NIP44_VERSION: u8 = 2;
