use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub(crate) struct Account {
    inner: Arc<Inner>,
}

impl Account {
    pub fn new(id: String, secret: String) -> Self {
        Self {
            inner: Arc::new(Inner::new(id, secret)),
        }
    }

    pub fn application_key(&self) -> ApplicationKey {
        self.inner.app_key.clone()
    }

    pub fn authorized(&self) -> Option<Authorized> {
        let guard = self.inner.authorized.lock().unwrap();
        (*guard).as_ref().cloned()
    }

    pub fn set_authorized(&self, authorized: Authorized) {
        let mut guard = self.inner.authorized.lock().unwrap();
        *guard = Some(authorized);
    }
}

#[derive(Debug)]
struct Inner {
    app_key: ApplicationKey,
    authorized: Mutex<Option<Authorized>>,
}

impl Inner {
    fn new(id: String, secret: String) -> Self {
        Self {
            app_key: ApplicationKey::new(id, secret),
            authorized: Mutex::new(None),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ApplicationKey {
    pub id: String,
    pub secret: String,
}

impl ApplicationKey {
    fn new(id: String, secret: String) -> Self {
        Self { id, secret }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Authorized {
    pub id: String,
    pub storage_api_info: StorageApiInfo,
    pub token: String,
}

#[derive(Clone, Debug)]
pub(crate) struct StorageApiInfo {
    pub url: String,
    pub download_url: String,
}
