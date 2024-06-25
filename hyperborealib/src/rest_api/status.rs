#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResponseStatus {
    /// Success - 100
    Success,

    /// Server error - 200
    ServerError,

    /// Protocol error - 300
    InvalidRequestStructure,

    /// Protocol error - 301
    RequestValidationFailed,

    /// Protocol error - 310
    ClientLookupTimeout,

    /// Protocol error - 311
    ClientNotFound,

    /// Protocol error - 320
    ClientNotConnected,

    /// Protocol error - 321
    ClientInboxFull,

    /// Protocol error - 322
    MessageTooLarge
}

impl ResponseStatus {
    pub fn from_code(code: u64) -> Option<Self> {
        let status = match code {
            // Success
            100 => Self::Success,

            // Server error
            200 => Self::ServerError,

            // Protocol error
            300 => Self::InvalidRequestStructure,
            301 => Self::RequestValidationFailed,

            // Protocol error - lookup error
            310 => Self::ClientLookupTimeout,
            311 => Self::ClientNotFound,

            // Protocol error - inbox error
            320 => Self::ClientNotConnected,
            321 => Self::ClientInboxFull,
            322 => Self::MessageTooLarge,

            _ => return None
        };

        Some(status)
    }

    pub fn to_code(&self) -> u64 {
        match self {
            // Success
            Self::Success => 100,

            // Server error
            Self::ServerError => 200,

            // Protocol error
            Self::InvalidRequestStructure => 300,
            Self::RequestValidationFailed => 301,

            // Protocol error - lookup error
            Self::ClientLookupTimeout => 310,
            Self::ClientNotFound      => 311,

            // Protocol error - inbox error
            Self::ClientNotConnected => 320,
            Self::ClientInboxFull    => 321,
            Self::MessageTooLarge    => 322
        }
    }

    #[inline]
    pub fn is_success(&self) -> bool {
        (100..200).contains(&self.to_code())
    }
}
