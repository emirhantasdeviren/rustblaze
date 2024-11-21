use ::std::result::Result as StdResult;
use serde::Deserialize;

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    message: String,
}

impl Error {
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl ::std::fmt::Display for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        self.message.fmt(f)
    }
}

impl ::std::error::Error for Error {}

impl From<ErrorResponse> for Error {
    fn from(res: ErrorResponse) -> Self {
        let message = res.message.clone();
        let kind = match ErrorKind::try_from(res) {
            Ok(k) => k,
            Err(e) => {
                tracing::warn!(message = "encountered unknown error code", code = e.0);
                ErrorKind::Unknown
            }
        };

        Self { kind, message }
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        let kind = ErrorKind::from(err);
        let message = match kind {
            ErrorKind::Connect => "could not connect",
            ErrorKind::Timeout => "timed out",
            ErrorKind::Deserialize => "invalid or malformed response",
            _ => "unknown error related to communication",
        }
        .to_string();

        Self { kind, message }
    }
}

#[derive(Debug, Copy, Clone)]
#[non_exhaustive]
pub enum ErrorKind {
    BadAuthToken,
    ExpiredAuthToken,
    BadBucketId,
    BadRequest,
    Unauthorized,
    Unsupported,
    TransactionCapExceeded,
    Connect,
    Timeout,
    Deserialize,
    Unknown,
}

#[derive(Debug, Clone)]
pub(crate) struct UnknownErrorCode(String);

impl ::std::fmt::Display for UnknownErrorCode {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        "unknown error code".fmt(f)
    }
}

impl ::std::error::Error for UnknownErrorCode {}

impl TryFrom<ErrorResponse> for ErrorKind {
    type Error = UnknownErrorCode;

    fn try_from(res: ErrorResponse) -> StdResult<Self, Self::Error> {
        match res.code.as_str() {
            "bad_auth_token" => Ok(Self::BadAuthToken),
            "expired_auth_token" => Ok(Self::ExpiredAuthToken),
            "bad_bucket_id" => Ok(Self::BadBucketId),
            "bad_request" => Ok(Self::BadRequest),
            "unauthorized" => Ok(Self::Unauthorized),
            "unsupported" => Ok(Self::Unsupported),
            "transaction_cap_exceeded" => Ok(Self::TransactionCapExceeded),
            code => Err(UnknownErrorCode(code.to_string())),
        }
    }
}

impl From<reqwest::Error> for ErrorKind {
    fn from(err: reqwest::Error) -> Self {
        if err.is_connect() {
            return Self::Connect;
        }
        if err.is_timeout() {
            return Self::Timeout;
        }
        if err.is_decode() {
            return Self::Deserialize;
        }

        return Self::Unknown;
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct ErrorResponse {
    status: u16,
    code: String,
    message: String,
}
