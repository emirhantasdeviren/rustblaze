use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub(crate) struct Account {
    inner: Arc<Inner>,
}

#[derive(Debug)]
struct Inner {
    app_key: ApplicationKey,
    authorized: Mutex<Option<Authorized>>,
}

#[derive(Clone, Debug)]
struct ApplicationKey {
    id: String,
    secret: String,
}

#[derive(Clone, Debug)]
struct Authorized {
    id: String,
    storage_api_info: StorageApiInfo,
    token: String,
}

#[derive(Clone, Debug)]
struct StorageApiInfo {
    url: String,
    download_url: String,
}
