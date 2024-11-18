use std::time::SystemTime;

#[derive(Clone, Debug)]
pub struct Bucket {
    client: super::Client,
    id: String,
    upload_url: Option<UploadUrl>,
}

#[derive(Clone, Debug)]
struct UploadUrl {
    url: String,
    token: String,
    generated_at: SystemTime,
}
