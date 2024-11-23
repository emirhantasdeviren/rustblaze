use bytes::Buf;
use clap::Parser;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

type BoxError = Box<dyn ::std::error::Error + Send + Sync + 'static>;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    id: String,

    #[arg(short, long)]
    secret: String,

    #[arg(short)]
    bucket_name: String,

    #[arg(short)]
    file_path: String,
}

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let args = Args::parse();
    let client = rustblaze::Client::new(args.id, args.secret);
    let bucket = match client.bucket(args.bucket_name).await? {
        Some(b) => b,
        None => return Err("bucket does not exits".into()),
    };

    bucket
        .upload_file(args.file_path.clone(), args.file_path.clone())
        .await?;

    Ok(())
}
