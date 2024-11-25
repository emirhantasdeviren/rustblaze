use std::path::Path;

use serde::de::DeserializeOwned;
use serde::Deserialize;
use sha1::{Digest, Sha1};
use tokio::io::{AsyncRead, AsyncReadExt};

use crate::account::{Authorized, StorageApiInfo};
use crate::bucket::{
    GetUploadUrlResponse, ListBucketsBuilder, ListBucketsRequest, ListBucketsResponse,
    UploadFileResponse,
};
use crate::error::ErrorResponse;
use crate::file::{ListFileNamesRequest, ListFileNamesResponse};
use crate::{Account, Bucket, Result};

pub const BASE_URL: &str = "https://api.backblazeb2.com";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthorizeAccountResponse {
    account_id: String,
    api_info: AuthorizeAccountApiInfo,
    #[serde(rename(deserialize = "authorizationToken"))]
    token: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthorizeAccountApiInfo {
    storage_api: AuthorizeAccountApiInfoStorageApi,
}

#[derive(Deserialize)]
struct AuthorizeAccountApiInfoStorageApi {
    #[serde(rename(deserialize = "apiUrl"))]
    url: String,
    #[serde(rename(deserialize = "downloadUrl"))]
    download_url: String,
}

#[derive(Clone, Debug)]
pub struct Client {
    inner: reqwest::Client,
    account: Account,
}

impl Client {
    pub fn new(id: String, secret: String) -> Self {
        Self {
            inner: reqwest::Client::new(),
            account: Account::new(id, secret),
        }
    }

    async fn get_or_try_authorize(&self) -> Result<Authorized> {
        if let Some(authorized) = self.account.authorized() {
            Ok(authorized)
        } else {
            self.authorize_account().await
        }
    }

    async fn authorize_account(&self) -> Result<Authorized> {
        const PATH: &str = "/b2api/v3/b2_authorize_account";
        let url = format!("{}{}", BASE_URL, PATH);
        let key = self.account.application_key();
        let req = self.inner.get(url).basic_auth(key.id, Some(key.secret));

        let res = req.send().await?;

        let res = handle_b2_api_response::<AuthorizeAccountResponse>(res).await?;

        let authorized = Authorized {
            id: res.account_id,
            storage_api_info: StorageApiInfo {
                url: res.api_info.storage_api.url,
                download_url: res.api_info.storage_api.download_url,
            },
            token: res.token,
        };

        self.account.set_authorized(authorized.clone());

        Ok(authorized)
    }

    pub(crate) async fn get_upload_url(&self, bucket_id: String) -> Result<GetUploadUrlResponse> {
        const PATH: &str = "/b2api/v3/b2_get_upload_url";
        let authorized = self.get_or_try_authorize().await?;
        let url = format!(
            "{}{}?bucketId={}",
            authorized.storage_api_info.url, PATH, bucket_id
        );
        let req = self
            .inner
            .get(url)
            .header(reqwest::header::AUTHORIZATION, authorized.token);
        let res = req.send().await?;

        handle_b2_api_response(res).await
    }

    pub(crate) async fn upload_file_from_reader<R: AsyncRead + Unpin>(
        &self,
        upload_url: String,
        authorization_token: String,
        mut reader: R,
        name: String,
    ) -> Result<UploadFileResponse> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await.unwrap();

        let mut hasher = Sha1::new();
        hasher.update(&buf);
        let sum = format!("{:x}", hasher.finalize());

        let content_type = "b2/x-auto";
        let content_length = buf.len();

        let req = self
            .inner
            .post(upload_url)
            .header(reqwest::header::AUTHORIZATION, authorization_token)
            .header("X-Bz-File-Name", name)
            .header(reqwest::header::CONTENT_TYPE, content_type)
            .header(reqwest::header::CONTENT_LENGTH, content_length)
            .header("X-Bz-Content-Sha1", sum)
            .body(buf);

        let res = req.send().await?;

        handle_b2_api_response(res).await
    }

    pub(crate) async fn upload_file<P: AsRef<Path>>(
        &self,
        upload_url: String,
        authorization_token: String,
        path: P,
        name: String,
    ) -> Result<UploadFileResponse> {
        let file = tokio::fs::read(path).await.unwrap();
        let mut hasher = Sha1::new();
        hasher.update(&file);
        let sum = format!("{:x}", hasher.finalize());

        let content_type = "b2/x-auto";
        let content_length = file.len();

        let req = self
            .inner
            .post(upload_url)
            .header(reqwest::header::AUTHORIZATION, authorization_token)
            .header("X-Bz-File-Name", name)
            .header(reqwest::header::CONTENT_TYPE, content_type)
            .header(reqwest::header::CONTENT_LENGTH, content_length)
            .header("X-Bz-Content-Sha1", sum)
            .body(file);

        let res = req.send().await?;

        handle_b2_api_response(res).await
    }

    pub(crate) async fn _list_buckets(
        &self,
        mut req: ListBucketsRequest,
    ) -> Result<ListBucketsResponse> {
        const PATH: &str = "/b2api/v3/b2_list_buckets";

        let authorized = self.get_or_try_authorize().await?;

        req.account_id = authorized.id;

        let url = format!("{}{}", authorized.storage_api_info.url, PATH);
        let req = self
            .inner
            .post(url)
            .header(reqwest::header::AUTHORIZATION, authorized.token)
            .json(&req);

        let res = req.send().await?;

        handle_b2_api_response(res).await
    }

    pub(crate) async fn _list_file_names(
        &self,
        req: ListFileNamesRequest,
    ) -> Result<ListFileNamesResponse> {
        const PATH: &str = "/b2api/v3/b2_list_file_names";

        let authorized = self.get_or_try_authorize().await?;

        let url = format!("{}{}", authorized.storage_api_info.url, PATH);
        let req = self
            .inner
            .get(url)
            .header(reqwest::header::AUTHORIZATION, authorized.token)
            .query(&req);

        let res = req.send().await?;

        handle_b2_api_response(res).await
    }

    pub async fn list_buckets(&self) -> ListBucketsBuilder {
        ListBucketsBuilder::new(self.clone())
    }

    pub async fn bucket<T: AsRef<str>>(&self, bucket_name: T) -> Result<Option<Bucket>> {
        let buckets = ListBucketsBuilder::new(self.clone())
            .bucket_name(bucket_name.as_ref())
            .send()
            .await?;

        Ok(buckets
            .into_iter()
            .find(|b| b.name() == bucket_name.as_ref()))
    }
}

async fn handle_b2_api_response<T>(res: reqwest::Response) -> Result<T>
where
    T: DeserializeOwned,
{
    if res.status().is_client_error() || res.status().is_server_error() {
        let err_response = res.json::<ErrorResponse>().await?;

        return Err(err_response.into());
    }

    match res.json::<T>().await {
        Ok(res) => Ok(res),
        Err(err) => {
            tracing::error!("could not deserialize response body: {:?}", err);
            Err(err.into())
        }
    }
}
