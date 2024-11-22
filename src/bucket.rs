mod list;

pub use list::ListBucketsBuilder;
pub(crate) use list::*;
use serde::Deserialize;

use crate::{Client, Result};

use std::sync::{Arc, Mutex};
use std::time::SystemTime;

#[derive(Clone, Debug)]
pub struct Bucket {
    client: Client,
    id: String,
    name: String,
    upload_url: UploadUrl,
}

#[derive(Clone, Default, Debug)]
struct UploadUrl {
    inner: Arc<Mutex<Option<UploadUrlInner>>>,
}

impl UploadUrl {
    fn get(&self) -> Option<UploadUrlInner> {
        let guard = self.inner.lock().unwrap();
        (*guard).clone()
    }

    fn set(&self, inner: UploadUrlInner) -> UploadUrlInner {
        let mut guard = self.inner.lock().unwrap();
        *guard = Some(inner.clone());
        inner
    }
}

#[derive(Clone, Debug)]
struct UploadUrlInner {
    url: String,
    token: String,
    generated_at: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetUploadUrlResponse {
    pub(crate) upload_url: String,
    pub(crate) authorization_token: String,
}

impl Bucket {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    async fn get_or_try_get_upload_url(&self) -> Result<UploadUrlInner> {
        let now: i64 = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis()
            .try_into()
            .unwrap();
        match self.upload_url.get() {
            Some(inner) if now - inner.generated_at < 86400000 => Ok(inner),
            _ => self.get_upload_url().await,
        }
    }

    async fn get_upload_url(&self) -> Result<UploadUrlInner> {
        let now: i64 = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis()
            .try_into()
            .unwrap();
        let res = self.client.get_upload_url(self.id.clone()).await?;
        let upload_url = UploadUrlInner {
            url: res.upload_url,
            token: res.authorization_token,
            generated_at: now,
        };
        self.upload_url.set(upload_url.clone());

        Ok(upload_url)
    }

    fn from_list_buckets_buckets(client: Client, bucket: ListBucketsBuckets) -> Self {
        Self {
            client,
            id: bucket.bucket_id,
            name: bucket.bucket_name,
            upload_url: Default::default(),
        }
    }

    fn from_list_buckets_response(client: Client, res: ListBucketsResponse) -> Vec<Self> {
        res.buckets
            .into_iter()
            .map(|b| Self::from_list_buckets_buckets(client.clone(), b))
            .collect()
    }
}
