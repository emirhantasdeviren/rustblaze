use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use super::File;
use crate::bucket::UploadFileResponse;
use crate::{Client, Result};

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NextFileName(String);

impl AsMut<str> for NextFileName {
    fn as_mut(&mut self) -> &mut str {
        self.0.as_mut()
    }
}

impl AsRef<[u8]> for NextFileName {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl AsRef<str> for NextFileName {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl Deref for NextFileName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl DerefMut for NextFileName {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.deref_mut()
    }
}

impl ::std::fmt::Display for NextFileName {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<String> for NextFileName {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for NextFileName {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListFileNamesRequest {
    bucket_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    start_file_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_file_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    delimeter: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListFileNamesResponse {
    pub(super) files: Vec<UploadFileResponse>,
    pub(super) next_file_name: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ListFileNamesBuilder {
    inner: Client,
    bucket_id: String,
    start_file_name: Option<String>,
    max_file_count: Option<usize>,
    prefix: Option<String>,
    delimeter: Option<String>,
}

impl ListFileNamesBuilder {
    pub(crate) fn new<T: AsRef<str>>(client: Client, bucket_id: T) -> Self {
        Self {
            inner: client,
            bucket_id: bucket_id.as_ref().to_string(),
            start_file_name: Default::default(),
            max_file_count: Default::default(),
            prefix: Default::default(),
            delimeter: Default::default(),
        }
    }

    pub fn start_file_name<T: AsRef<str>>(&mut self, start_file_name: T) -> &mut Self {
        self.start_file_name = Some(start_file_name.as_ref().to_string());
        self
    }

    pub fn max_file_count(&mut self, max_file_count: usize) -> &mut Self {
        self.max_file_count = Some(max_file_count);
        self
    }

    pub fn prefix<T: AsRef<str>>(&mut self, prefix: T) -> &mut Self {
        self.prefix = Some(prefix.as_ref().to_string());
        self
    }

    pub fn delimeter<T: AsRef<str>>(&mut self, delimeter: T) -> &mut Self {
        self.delimeter = Some(delimeter.as_ref().to_string());
        self
    }

    pub async fn send(&mut self) -> Result<(Vec<File>, Option<NextFileName>)> {
        let req = ListFileNamesRequest {
            bucket_id: self.bucket_id.clone(),
            start_file_name: self.start_file_name.clone(),
            max_file_count: self.max_file_count.clone(),
            prefix: self.prefix.clone(),
            delimeter: self.delimeter.clone(),
        };

        let res = self.inner._list_file_names(req).await?;
        let next_file_name = res.next_file_name.map(|s| NextFileName(s));

        Ok((
            res.files.into_iter().map(From::from).collect(),
            next_file_name,
        ))
    }
}
