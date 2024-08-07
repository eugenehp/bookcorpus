use std::cmp::min;
use std::path::PathBuf;
use archiver_rs::{ Archive as _, ArchiverError, Compressed as _ };
use futures::TryStreamExt as _;
use indicatif::{ ProgressBar, ProgressStyle };
use reqwest::Client;
use tokio::fs::File;
use tokio::io::AsyncWriteExt as _;

use crate::cache::Cache;
use crate::error::ApiError;

const URL: &str =
    "https://storage.googleapis.com/huggingface-nlp/datasets/bookcorpus/bookcorpus.tar.bz2";

#[derive(Clone, Debug)]
pub struct Api {
    url: String,
    cache: Cache,
    client: Client,
    // max_files: usize,
    // chunk_size: usize,
    // parallel_failures: usize,
    // max_retries: usize,
    progress: bool,
}

impl Default for Api {
    fn default() -> Self {
        Self {
            url: URL.to_string(),
            cache: Default::default(),
            client: Default::default(),
            // max_files: num_cpus::get(),
            // chunk_size: 10_000_000,
            // parallel_failures: 0,
            // max_retries: 0,
            progress: true,
        }
    }
}

impl Api {
    pub fn new() -> Self {
        Api::default()
    }

    pub fn get_filename(&self) -> String {
        let url = self.url.clone();
        let chunks = url.split("/").collect::<Vec<&str>>();
        let filename = chunks.last().unwrap();
        filename.to_string()
    }

    pub fn get_filename_path(&self) -> PathBuf {
        self.cache.blob_path(&self.get_filename())
    }

    // TODO: migrate to tokio version, once this is merged https://github.com/zip-rs/zip2/pull/121
    // TODO: add progress bar
    pub fn unzip(&self) -> Result<Vec<String>, ArchiverError> {
        let zip_path = self.get_filename_path();
        println!("Unzip for {}", zip_path.to_string_lossy());

        let mut bz = archiver_rs::Bzip2::open(zip_path.as_path()).unwrap();
        let target_path = zip_path.to_string_lossy().replace(".bz2", "");
        let target_path = PathBuf::from(target_path);

        if !target_path.exists() {
            bz.decompress(target_path.as_path()).unwrap();
        }

        let mut tar = archiver_rs::Tar::open(target_path.as_path()).unwrap();
        tar.extract(self.cache.path()).unwrap();

        let mut tar = archiver_rs::Tar::open(target_path.as_path()).unwrap();
        tar.files()
    }

    pub async fn download(&self) -> Result<PathBuf, ApiError> {
        let blob_path = self.get_filename_path();

        if !blob_path.exists() {
            let tmp_filename = self.download_tempfile().await.unwrap();
            tokio::fs::rename(&tmp_filename, &blob_path).await.unwrap();
        }

        Ok(blob_path)
    }

    pub async fn download_tempfile(&self) -> Result<PathBuf, ApiError> {
        let filename = self.cache.temp_path();

        let res = self.client.get(self.url.clone()).send().await?;
        let total_size = res.content_length().unwrap();
        let is_progress = self.progress;

        let pb = match is_progress {
            true => {
                let pb = ProgressBar::new(total_size);
                pb.set_style(
                    ProgressStyle::default_bar()
                        .template(
                            "{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.green/gray}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})"
                        )
                        .unwrap()
                        .progress_chars("■ ")
                );
                pb.set_message(format!("Downloading {}", self.url));
                Some(pb)
            }
            _ => None,
        };

        let filename_str = filename.to_string_lossy().to_string();
        let mut file = File::create(filename.clone()).await
            .or(Err(format!("Failed to create file '{}'", filename_str)))
            .unwrap();
        let mut downloaded: u64 = 0;
        let mut stream = res.bytes_stream();

        while let Some(chunk) = stream.try_next().await.unwrap() {
            file.write_all(&chunk).await?;
            let new = min(downloaded + (chunk.len() as u64), total_size);
            downloaded = new;
            if is_progress {
                pb.clone().unwrap().set_position(new);
            }
        }

        if is_progress {
            pb.unwrap().finish_with_message(format!("Downloaded {} to {}", self.url, filename_str));
        }

        Ok(filename)
    }
}
