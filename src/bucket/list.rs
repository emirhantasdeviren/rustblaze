use serde::{Deserialize, Serialize};

use crate::{Bucket, Client, Result};

#[derive(Debug, Clone)]
pub struct ListBucketsBuilder {
    inner: Client,
    bucket_id: Option<String>,
    bucket_name: Option<String>,
    _bucket_types: Option<()>,
}

impl ListBucketsBuilder {
    pub(crate) fn new(client: Client) -> Self {
        Self {
            inner: client,
            bucket_id: Default::default(),
            bucket_name: Default::default(),
            _bucket_types: Default::default(),
        }
    }

    pub fn bucket_id<T: AsRef<str>>(&mut self, id: T) -> &mut Self {
        self.bucket_id = Some(id.as_ref().to_string());
        self
    }

    pub fn bucket_name<T: AsRef<str>>(&mut self, name: T) -> &mut Self {
        self.bucket_name = Some(name.as_ref().to_string());
        self
    }

    pub async fn send(&mut self) -> Result<Vec<Bucket>> {
        let req = ListBucketsRequest {
            bucket_id: self.bucket_id.clone(),
            bucket_name: self.bucket_name.clone(),
            ..Default::default()
        };

        self.inner
            ._list_buckets(req)
            .await
            .map(|res| Bucket::from_list_buckets_response(self.inner.clone(), res))
    }
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListBucketsRequest {
    pub account_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bucket_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bucket_name: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct ListBucketsResponse {
    pub buckets: Vec<ListBucketsBuckets>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListBucketsBuckets {
    pub account_id: String,
    pub bucket_id: String,
    pub bucket_name: String,
}
