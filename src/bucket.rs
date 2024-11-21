mod list;

pub use list::ListBucketsBuilder;
pub(crate) use list::*;

use crate::Client;

use std::time::SystemTime;

#[derive(Clone, Debug)]
pub struct Bucket {
    client: Client,
    id: String,
    upload_url: Option<UploadUrl>,
}

#[derive(Clone, Debug)]
struct UploadUrl {
    url: String,
    token: String,
    generated_at: SystemTime,
}

impl Bucket {
    fn from_list_buckets_buckets(client: Client, bucket: ListBucketsBuckets) -> Self {
        Self {
            client,
            id: bucket.bucket_id,
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
