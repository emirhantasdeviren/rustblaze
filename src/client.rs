use std::sync::{Arc, Mutex};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::error::ErrorResponse;
use crate::{Bucket, Result};

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

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ListBucketsRequest {
    account_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    bucket_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bucket_name: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct ListBucketsResponse {
    buckets: Vec<ListBucketsBuckets>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ListBucketsBuckets {
    account_id: String,
    bucket_id: String,
    bucket_name: String,
}

#[derive(Clone, Debug)]
pub struct Client {
    inner: reqwest::Client,
    app_key: ApplicationKey,
    authorized_account: Arc<Mutex<Option<Account>>>,
}

impl Client {
    pub fn new(app_key: ApplicationKey) -> Self {
        Self {
            inner: reqwest::Client::new(),
            app_key,
            authorized_account: Arc::new(Mutex::new(None)),
        }
    }

    async fn authorize_account(&self) -> Result<()> {
        const PATH: &str = "/b2api/v3/b2_authorize_account";
        let url = format!("{}{}", BASE_URL, PATH);
        let req = self
            .inner
            .get(url)
            .basic_auth(&self.app_key.id, Some(&self.app_key.secret));

        let res = req.send().await?;

        let res = handle_b2_api_response::<AuthorizeAccountResponse>(res).await?;

        let account = Account {
            id: res.account_id,
            storage_api_info: StorageApiInfo {
                url: res.api_info.storage_api.url,
                download_url: res.api_info.storage_api.download_url,
            },
            token: res.token,
        };

        *self.authorized_account.lock().unwrap() = Some(account);

        Ok(())
    }

    async fn _list_buckets(&self, mut req: ListBucketsRequest) -> Result<ListBucketsResponse> {
        const PATH: &str = "/b2api/v3/b2_list_buckets";

        let account = self
            .authorized_account
            .lock()
            .unwrap()
            .as_ref()
            .cloned()
            .unwrap();

        req.account_id = account.id;

        let url = format!("{}{}", account.storage_api_info.url, PATH);
        let req = self
            .inner
            .post(url)
            .header(reqwest::header::AUTHORIZATION, account.token)
            .json(&req);

        let res = req.send().await?;

        handle_b2_api_response(res).await
    }

    pub async fn bucket<T: AsRef<str>>(&self, bucket_name: T) -> Bucket {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub struct ApplicationKey {
    id: String,
    secret: String,
}

impl ApplicationKey {
    pub fn new(id: String, secret: String) -> Self {
        Self { id, secret }
    }
}

#[derive(Clone, Debug)]
struct StorageApiInfo {
    url: String,
    download_url: String,
}

#[derive(Clone, Debug)]
struct Account {
    id: String,
    storage_api_info: StorageApiInfo,
    token: String,
}

async fn handle_b2_api_response<T>(res: reqwest::Response) -> Result<T>
where
    T: DeserializeOwned,
{
    if res.status().is_client_error() || res.status().is_server_error() {
        let err_response = res.json::<ErrorResponse>().await?;

        return Err(err_response.into());
    }

    Ok(res.json::<T>().await?)
}
