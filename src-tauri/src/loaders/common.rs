use crate::models::InstanceInstallProgress;
use chrono::Local;
use futures_util::StreamExt;
use std::fs;
use std::io::Write;
use tauri::Emitter;

/// Stream a URL to a local file, emitting `install-progress` events as bytes
/// land. Used by every loader that needs to fetch a JAR.
pub async fn download_file(
    app: &tauri::AppHandle,
    url: &str,
    path: &std::path::Path,
    id: &str,
    known_size: Option<u64>,
    log_path: Option<&std::path::Path>,
) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .user_agent("AnvilCraft/1.0")
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| e.to_string())?;

    println!("[DEBUG] Downloading: {}", url);
    let mut log_file = log_path.and_then(|p| {
        fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(p)
            .ok()
    });

    if let Some(ref mut f) = log_file {
        let _ = writeln!(
            f,
            "[{}] Starting download: {}",
            Local::now().format("%H:%M:%S"),
            url
        );
    }

    let _ = app.emit(
        "install-progress",
        InstanceInstallProgress {
            id: id.to_string(),
            step: format!("Connecting: {}", url),
            progress: 0,
            total_size: known_size,
            downloaded: 0,
        },
    );

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?;

    // Intermediate event: the TCP/TLS handshake is done and the server
    // accepted the request. Without this, a slow first chunk would leave
    // the UI stuck on "Connecting: …" with no feedback.
    let total_size = response.content_length().or(known_size).filter(|&s| s > 0);
    let _ = app.emit(
        "install-progress",
        InstanceInstallProgress {
            id: id.to_string(),
            step: format!("Connected. Streaming {} bytes…", total_size.unwrap_or(0)),
            progress: 0,
            total_size,
            downloaded: 0,
        },
    );
    if let Some(ref mut f) = log_file {
        let _ = writeln!(
            f,
            "[{}] HTTP {} — {} bytes expected",
            Local::now().format("%H:%M:%S"),
            response.status(),
            total_size.unwrap_or(0)
        );
    }

    let mut file = fs::File::create(path).map_err(|e| e.to_string())?;
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();
    eprintln!("[download_file] Stream created, beginning to poll for chunks…");

    let mut last_emit = std::time::Instant::now();
    let mut last_progress = 0;
    let mut last_logged_mb = 0;
    let mut chunks_received: u64 = 0;

    // Read timeout: if no chunk arrives in 45s, abort with a clear error
    // instead of letting the body stream hang forever.
    let read_timeout = std::time::Duration::from_secs(45);

    loop {
        let item = match tokio::time::timeout(read_timeout, stream.next()).await {
            Ok(Some(item)) => item,
            Ok(None) => break, // stream ended cleanly
            Err(_) => {
                return Err(format!(
                    "Read timeout: no chunk received in {}s (downloaded {} bytes)",
                    read_timeout.as_secs(),
                    downloaded
                ));
            }
        };
        let chunk = item.map_err(|e| format!("Stream error: {}", e))?;
        file.write_all(&chunk).map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;
        chunks_received += 1;
        if chunks_received == 1 || chunks_received % 50 == 0 {
            eprintln!("[download_file] Chunk #{}: {} bytes (total {})", chunks_received, chunk.len(), downloaded);
        }

        let current_mb = downloaded / (1024 * 1024);
        let mut progress = 0;
        let mut is_finished = false;

        if let Some(size) = total_size {
            progress = (downloaded as f64 / size as f64 * 100.0) as u64;
            is_finished = downloaded == size;
        }

        if current_mb > last_logged_mb || is_finished {
            last_logged_mb = current_mb;
            if let Some(ref mut f) = log_file {
                let _ = writeln!(
                    f,
                    "[{}] Progress: {}% ({} bytes)",
                    Local::now().format("%H:%M:%S"),
                    progress,
                    downloaded
                );
            }
        }

        if progress > last_progress || last_emit.elapsed().as_millis() > 50 || is_finished {
            last_progress = progress;
            last_emit = std::time::Instant::now();

            let _ = app.emit(
                "install-progress",
                InstanceInstallProgress {
                    id: id.to_string(),
                    step: format!("Downloading..."),
                    progress,
                    total_size,
                    downloaded,
                },
            );
        }
    }

    if let Some(ref mut f) = log_file {
        let _ = writeln!(
            f,
            "[{}] Download finished.",
            Local::now().format("%H:%M:%S")
        );
    }

    let _ = app.emit(
        "install-progress",
        InstanceInstallProgress {
            id: id.to_string(),
            step: "Finalizing download...".into(),
            progress: 100,
            total_size: total_size,
            downloaded,
        },
    );

    Ok(())
}

/// Write `eula.txt` (or skip if `accept` is false). Shared by every loader.
pub fn write_eula_txt(path: std::path::PathBuf, accept: bool) -> Result<(), String> {
    if !accept {
        return Ok(());
    }
    let timestamp = Local::now().format("%a %b %d %H:%M:%S %Z %Y").to_string();
    let content = format!(
        "#By changing the setting below to TRUE you are indicating your agreement to our EULA (https://aka.ms/MinecraftEULA).\n#{}\neula=true",
        timestamp
    );

    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }

    fs::write(path, content).map_err(|e| e.to_string())
}
