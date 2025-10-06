use crate::types::results::{ResultWithError, U8Result};
use async_stream::try_stream;
use futures::Stream;
use regex::Regex;
use std::{path::PathBuf, pin::Pin, process::Stdio};
use tokio::{
    fs::{self},
    io::AsyncReadExt,
    process::Command,
};
use tokio_util::sync::CancellationToken;

pub fn get_folder_name(path: PathBuf) -> Option<String> {
    if let Some(last) = path.components().next_back() {
        let last_str = last.as_os_str().to_string_lossy().into_owned();
        return Some(last_str);
    }
    None
}

/// Move app from one directory to another
pub async fn move_folder_with_progress(
    from: &str,
    to: &str,
    cancel_token: CancellationToken,
) -> Pin<Box<impl Stream<Item = ResultWithError<U8Result>>>> {
    log::info!("Moving folder from: {} to: {}", from, to);

    let mut from = String::from(from);
    if !from.ends_with("/") {
        from.push('/');
    }
    let to = String::from(to);

    let stream = try_stream! {
        // ensure the new folder exists
        fs::create_dir_all(&to).await?;

        // move the files to the new location
        let mut command = Command::new("rsync")
            .arg("-a")
            .arg("--info=progress2")
            .arg(&from)
            .arg(&to)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdout = command.stdout.as_mut().unwrap();
        let progress_regex = Regex::new(r"\s(\d+)%\s").unwrap();

        let mut buffer = [0; 1];
        let mut line = Vec::new();
        let mut last_progress = 0;

        while stdout.read(&mut buffer).await? > 0 {
            if cancel_token.is_cancelled() {
                log::info!("Cancelling rsync process...");
                command.kill().await.ok();
                break;
            }
            if buffer[0] == b'\r' || buffer[0] == b'\n' {
                let progress = String::from_utf8_lossy(&line);

                if let Some(Ok(progress)) = progress_regex
                    .captures(&progress)
                    .and_then(|cap| cap.get(1))
                    .map(|progress| progress.as_str().parse::<u8>())
                {
                    if progress != last_progress {
                        last_progress = progress;
                        yield Ok(progress);
                    }
                }

                line.clear(); // Clear the buffer after printing
                continue;
            }

            // Collect the bytes of the line
            line.push(buffer[0]);
        }

        let output = command.wait_with_output().await?;
        let has_err = !output.status.success();

        if cancel_token.is_cancelled() || has_err {
            fs::remove_dir_all(&to).await?;
            let stdout = std::str::from_utf8(&output.stdout)?;
            let stderr = std::str::from_utf8(&output.stderr)?;
            log::error!("Error moving app {}, status:{}, stdout:{}", stderr, output.status, stdout);
            yield Err("App move cancelled".into());
        } else {
            fs::remove_dir_all(&from).await?;
        }
    };

    Box::pin(stream)
}
