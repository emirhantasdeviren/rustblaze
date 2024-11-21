mod list;

pub(crate) use list::*;

use crate::Client;

use std::time::SystemTime;

#[derive(Clone, Debug)]
pub struct Bucket {
    client: Client,
    id: String,
    name: String,
    upload_url: Option<UploadUrl>,
}

#[derive(Clone, Debug)]
struct UploadUrl {
    url: String,
    token: String,
    generated_at: SystemTime,
}

impl Bucket {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    fn from_list_buckets_buckets(client: Client, bucket: ListBucketsBuckets) -> Self {
        Self {
            client,
            id: bucket.bucket_id,
            name: bucket.bucket_name,
            upload_url: None,
        }
    }

    fn from_list_buckets_response(client: Client, res: ListBucketsResponse) -> Vec<Self> {
        res.buckets
            .into_iter()
            .map(|b| Self::from_list_buckets_buckets(client.clone(), b))
            .collect()
    }
}
