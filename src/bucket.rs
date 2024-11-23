mod list;

pub use self::list::ListBucketsBuilder;
pub(crate) use self::list::*;

use serde::Deserialize;
use tokio::io::AsyncRead;

use crate::file::{File, ListFileNamesBuilder};
use crate::{Client, Result};

use std::path::Path;
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

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UploadFileResponse {
    pub(crate) account_id: String,
    pub(crate) bucket_id: String,
    pub(crate) content_length: usize,
    pub(crate) content_sha1: Option<String>,
    pub(crate) content_md5: Option<String>,
    pub(crate) content_type: Option<String>,
    pub(crate) file_id: String,
    pub(crate) file_name: String,
    pub(crate) upload_timestamp: i64,
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

    pub fn list_files(&self) -> ListFileNamesBuilder {
        ListFileNamesBuilder::new(self.client.clone(), &self.id)
    }

    pub async fn upload_file<P: AsRef<Path>>(&self, path: P, name: String) -> Result<File> {
        let _root_span = tracing::trace_span!("upload_file").entered();
        let url_span = tracing::trace_span!("get_url").entered();
        tracing::trace!("getting upload url");
        let start = SystemTime::now();
        let upload_url = self.get_or_try_get_upload_url().await?;
        let elapsed = start.elapsed().unwrap();
        tracing::trace!("successfully got upload url, took {:?}", elapsed);
        url_span.exit();

        let _inner_span = tracing::trace_span!("inner");
        let res = self
            .client
            .upload_file(upload_url.url, upload_url.token, path, name)
            .await?;
        let file = File {
            id: res.file_id,
            name: res.file_name,
            size: res.content_length,
            upload_timestamp: res.upload_timestamp,
        };

        Ok(file)
    }

    pub async fn upload_file_from_reader<R, S>(&self, reader: R, name: S) -> Result<File>
    where
        R: AsyncRead + Unpin,
        S: AsRef<str>,
    {
        let _root_span = tracing::trace_span!("upload_file").entered();
        let url_span = tracing::trace_span!("get_url").entered();
        tracing::trace!("getting upload url");
        let start = SystemTime::now();
        let upload_url = self.get_or_try_get_upload_url().await?;
        let elapsed = start.elapsed().unwrap();
        tracing::trace!("successfully got upload url, took {:?}", elapsed);
        url_span.exit();

        let inner_span = tracing::trace_span!("inner").entered();
        let res = self
            .client
            .upload_file_from_reader(
                upload_url.url,
                upload_url.token,
                reader,
                name.as_ref().to_owned(),
            )
            .await?;
        inner_span.exit();


        Ok(res.into())
    }
}
