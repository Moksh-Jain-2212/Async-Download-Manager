mod cli;
mod download;
mod manager;
mod progress;
mod storage;

use download::download_file;
use anyhow::Error;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::Arc;
use tokio::sync::Semaphore;

#[derive(Debug)]
pub enum DownloadStatus {
    Queued,
    Downloading,
    Completed,
    Failed,
}

#[derive(Debug)]
pub struct Download {
    pub id: u64,
    pub url: String,
    pub filename: String,
    pub status: DownloadStatus,
}

impl Download {
    pub fn new(id: u64, filename: String, url: String) -> Download {
        Download {
            id,
            url,
            filename,
            status: DownloadStatus::Queued,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("Async Download Manager");

    let client = reqwest::Client::new();

    let Downloads = vec![
        Download::new(
            1,
            String::from("1Mb.dat"),
            String::from("https://bom.proof.ovh.net/files/1Mb.dat")
        ),
        Download::new(
            2,
            String::from("test.bin"),
            String::from("https://testfile.to/dl/10mb")
        ),
        Download::new(
            3,
            String::from("test-5mb.bin"),
            String::from("https://cdn.truefilesize.com/test/test-5mb.bin")
        )
    ];

    let max_concurrent = 2;

    let semaphore = Arc::new(
        Semaphore::new(max_concurrent)
    );

    let mut handles = Vec::new();

    let multi_progress = MultiProgress::new();

    for mut download in Downloads {
        let client = client.clone();
        let semaphore = semaphore.clone();

        let progress = multi_progress.add(
            ProgressBar::new(0)
        );

        progress.set_style(
            ProgressStyle::with_template(
                "[{msg}] [{bar:40}] {bytes}/{total_bytes} {bytes_per_sec} ETA:{eta}"
            )?
        );

        let handle = tokio::spawn(async move {

            download.status = DownloadStatus::Queued;

            progress.set_message(format!(
                "{} - {} - Queued",
                download.id,
                download.filename
            ));

            let permit = semaphore
                .acquire_owned()
                .await
                .unwrap();
            
            download.status = DownloadStatus::Downloading;

            progress.set_message(format!(
                "{} - {} - Downloading",
                download.id,
                download.filename
            ));

            let result = download_file(
                &client,
                &download.url,
                &download.filename,
                progress.clone(),
            )
            .await;

            match result {
                Ok(_) => {
                    download.status = DownloadStatus::Completed;

                    progress.finish_with_message(format!(
                        "{} - {} - Completed",
                        download.id,
                        download.filename
                    ));
                }

                Err(error) => {
                    download.status = DownloadStatus::Failed;

                    progress.abandon_with_message(format!(
                        "{} - {} - Failed: {}",
                        download.id,
                        download.filename,
                        error
                    ));
                }
            }

            drop(permit);

            download
        });

        handles.push(handle);
    }

    let mut finished_downloads = Vec::new();

    for handle in handles {
        match handle.await {
            Ok(download) => {
                finished_downloads.push(download);
            }

            Err(error) => {
                eprintln!("Task crashed: {}", error);
            }
        }
    }

    println!("\nDownload Summary:");

    for download in finished_downloads {
        println!(
            "[{}] {} - {:?}",
            download.id,
            download.filename,
            download.status
        );
    }


    Ok(())
}