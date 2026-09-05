use std::fs;
use std::path::{Path, PathBuf};

use md5::{Digest, Md5};
use tokio::process::Command;
use tokio::time::{Duration, timeout};
use uuid::Uuid;

use crate::export::ExportError;

const CONVERT_TIMEOUT_SECS: u64 = 300;

/// Convert an EPUB to another format via Calibre's `ebook-convert`.
///
/// If `calibre_container` is non-empty, the command is run inside a Docker
/// container (`docker exec <container> ebook-convert ...`). Otherwise it is
/// assumed `ebook-convert` is installed on the host system PATH.
///
/// The command is wrapped in a 300-second timeout. On failure it is retried
/// once before returning an error.
///
/// Returns `(output_path, md5_hex)`.
pub async fn convert_epub(
    epub_path: &Path,
    output_format: &str,
    calibre_container: &str,
    tmp_dir: &Path,
) -> Result<(PathBuf, String), ExportError> {
    // ---- work directory ---------------------------------------------------
    let uuid = Uuid::new_v4();
    let work_dir = tmp_dir.join(uuid.to_string());
    fs::create_dir_all(&work_dir)?;

    let output_filename = format!("output.{output_format}");
    let output_path = work_dir.join(&output_filename);

    // ---- first attempt ----------------------------------------------------
    let first = run_conversion(epub_path, &output_path, calibre_container).await;

    match first {
        Ok(()) => {}
        Err(e) => {
            // ---- retry once -----------------------------------------------
            tracing::warn!("First conversion attempt failed: {e}. Retrying once ...");
            run_conversion(epub_path, &output_path, calibre_container)
                .await
                .map_err(|retry_err| {
                    ExportError::CalibreError(format!(
                        "Calibre conversion failed after retry: {retry_err}"
                    ))
                })?;
        }
    }

    // ---- verify output exists ---------------------------------------------
    if !output_path.exists() {
        return Err(ExportError::CalibreError(format!(
            "Output file was not created: {}",
            output_path.display()
        )));
    }

    // ---- MD5 hash ---------------------------------------------------------
    let output_data = fs::read(&output_path)?;
    let md5_hex = Md5::digest(&output_data)
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    Ok((output_path, md5_hex))
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Run a single `ebook-convert` invocation (direct or inside Docker) with a
/// 300-second timeout.
async fn run_conversion(
    epub_path: &Path,
    output_path: &Path,
    calibre_container: &str,
) -> Result<(), ExportError> {
    let timeout_dur = Duration::from_secs(CONVERT_TIMEOUT_SECS);

    if calibre_container.is_empty() {
        run_direct(epub_path, output_path, timeout_dur).await
    } else {
        run_docker(calibre_container, epub_path, output_path, timeout_dur).await
    }
}

/// Run `ebook-convert <input> <output>` directly on the host.
async fn run_direct(
    epub_path: &Path,
    output_path: &Path,
    timeout_dur: Duration,
) -> Result<(), ExportError> {
    let result = timeout(timeout_dur, async {
        Command::new("ebook-convert")
            .arg(epub_path)
            .arg(output_path)
            .output()
            .await
    })
    .await;

    handle_command_result(result, "ebook-convert")
}

/// Run `docker exec <container> ebook-convert <input> <output>`.
async fn run_docker(
    container: &str,
    epub_path: &Path,
    output_path: &Path,
    timeout_dur: Duration,
) -> Result<(), ExportError> {
    let result = timeout(timeout_dur, async {
        Command::new("docker")
            .arg("exec")
            .arg(container)
            .arg("ebook-convert")
            .arg(epub_path)
            .arg(output_path)
            .output()
            .await
    })
    .await;

    handle_command_result(result, &format!("docker exec {container} ebook-convert"))
}

/// Inspect the result of a `Command::output()` call wrapped in a timeout.
fn handle_command_result(
    result: Result<Result<std::process::Output, std::io::Error>, tokio::time::error::Elapsed>,
    command_label: &str,
) -> Result<(), ExportError> {
    match result {
        Ok(Ok(output)) => {
            if output.status.success() {
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(ExportError::CalibreError(format!(
                    "{command_label} failed: {stderr}"
                )))
            }
        }
        Ok(Err(e)) => Err(ExportError::CalibreError(format!(
            "{command_label} process error: {e}"
        ))),
        Err(_) => Err(ExportError::CalibreError(format!(
            "{command_label} timed out after {CONVERT_TIMEOUT_SECS} seconds"
        ))),
    }
}
