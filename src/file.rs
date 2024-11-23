mod list;

pub use list::{ListFileNamesBuilder, NextFileName};

pub(crate) use list::*;

use crate::bucket::UploadFileResponse;

#[derive(Clone, Debug)]
pub struct File {
    pub id: String,
    pub name: String,
    pub size: usize,
    pub upload_timestamp: i64,
}

impl From<UploadFileResponse> for File {
    fn from(res: UploadFileResponse) -> Self {
        Self {
            id: res.file_id,
            name: res.file_name,
            size: res.content_length,
            upload_timestamp: res.upload_timestamp,
        }
    }
}
