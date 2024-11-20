use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    status: u16,
    code: String,
    message: String,
}

pub struct Error {
    kind: ErrorKind,
}

impl Error {
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
}

impl TryFrom<ErrorResponse> for Error {
    type Error = UnknownErrorCode;

    fn try_from(res: ErrorResponse) -> Result<Self, Self::Error> {
        let kind = ErrorKind::try_from(res)?;

        Ok(Self { kind })
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
}

#[derive(Debug, Clone)]
struct UnknownErrorCode(String);

impl ::std::fmt::Display for UnknownErrorCode {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        "unknown error code".fmt(f)
    }
}

impl ::std::error::Error for UnknownErrorCode {}

impl TryFrom<ErrorResponse> for ErrorKind {
    type Error = UnknownErrorCode;

    fn try_from(res: ErrorResponse) -> Result<Self, Self::Error> {
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
