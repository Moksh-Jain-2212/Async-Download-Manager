use anyhow::Result;
use futures_util::StreamExt;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use indicatif::ProgressBar;

pub async fn download_file(
    client: &reqwest::Client,
    url: &str,
    filename: &str,
    progress: ProgressBar,
) -> Result<()> {

    let response = client
        .get(url)
        .send()
        .await?
        .error_for_status()?;

    let mut file = File::create(filename).await?;

    let total_size = response
        .content_length()
        .unwrap_or(0);

    let mut stream = response.bytes_stream();

    progress.set_length(total_size);

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;

        file.write_all(&chunk).await?;

        progress.inc(chunk.len() as u64);
    }

    file.flush().await?;

    Ok(())
}