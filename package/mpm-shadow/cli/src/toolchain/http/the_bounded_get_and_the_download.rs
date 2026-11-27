//! What:
//! The two bounded reads of the real client, inside its runtime:
//! `get_async` collects the body of one GET into memory,
//! refusing more than the caller's byte bound and failing when the body stalls,
//! and `download_async` streams the body into a file,
//! hashing as it writes,
//! and returns the SHA-256 as lowercase hex.
//! Why:
//! A registry lookup and a release archive have different bounds (a few megabytes against a couple of gigabytes),
//! and both must stop on a bound rather than fill the disk or the memory:
//! the idle timeout is what makes a stalled chunk a failure instead of a wait with no end.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // http/the_bounded_get_and_the_download.ts
//! async getAsync(url: string, headers: [string, string][], maxBytes: number): Promise<HttpResponse>;
//! async downloadAsync(url: string, target: string, maxBytes: number): Promise<string>;
//! ```

/// Imports the body helpers from `http_body_util`.
use http_body_util::BodyExt;
/// Imports what this module uses from `sha2`.
use sha2::{
    Digest,
    Sha256,
};
/// Imports `File` from `std::fs`.
use std::fs::File;
/// Imports `Write` from `std::io`.
use std::io::Write;
/// Imports `Path` from `std::path`.
use std::path::Path;
/// Imports `Duration` from `std::time`.
use std::time::Duration;

/// Imports `hex_of_digest` from the toolchain's hash module, one folder above this one.
use super::super::hash::hex_of_digest;
/// Imports the two idle and total bounds, the error, the response and the client from `super`.
use super::{
    DOWNLOAD_IDLE_TIMEOUT_SECONDS,
    DOWNLOAD_TOTAL_TIMEOUT_SECONDS,
    HttpError,
    HttpResponse,
    HyperClient,
};

/// The methods of `HyperClient` that read a bounded body,
/// collecting it or writing it to a file.
impl HyperClient {
    /// The bounded GET,
    /// inside the runtime.
    pub(super) async fn get_async(
        &self,
        url: &str,
        headers: &[(String, String)],
        max_bytes: usize,
    ) -> Result<HttpResponse, HttpError> {
        let response = self.send(url, headers).await?;
        let status: u16 = response.status().as_u16();
        let mut body = response.into_body();
        let mut collected: Vec<u8> = Vec::new();
        loop {
            let next = match tokio::time::timeout(
                Duration::from_secs(DOWNLOAD_IDLE_TIMEOUT_SECONDS),
                body.frame(),
            )
            .await
            {
                Ok(frame) => frame,
                Err(_elapsed) => {
                    return Err(HttpError {
                        url: url.to_string(),
                        message: format!(
                            "the body stalled for {DOWNLOAD_IDLE_TIMEOUT_SECONDS} seconds"
                        ),
                    });
                }
            };
            let Some(received_frame) = next else {
                break;
            };
            let frame = match received_frame {
                Ok(received) => received,
                Err(error) => {
                    return Err(HttpError {
                        url: url.to_string(),
                        message: format!("reading the body failed: {error}"),
                    });
                }
            };
            if let Some(data) = frame.data_ref() {
                if collected.len() + data.len() > max_bytes {
                    return Err(HttpError {
                        url: url.to_string(),
                        message: format!(
                            "the response is larger than the {max_bytes} bytes this lookup accepts"
                        ),
                    });
                }
                collected.extend_from_slice(data);
            }
        }
        return Ok(HttpResponse {
            status,
            body: collected,
        });
    }

    /// The download,
    /// inside the runtime.
    pub(super) async fn download_async(
        &self,
        url: &str,
        target: &Path,
        max_bytes: u64,
    ) -> Result<String, HttpError> {
        let response = self.send(url, &[]).await?;
        let status: u16 = response.status().as_u16();
        if !(200..300).contains(&status) {
            return Err(HttpError {
                url: url.to_string(),
                message: format!("HTTP {status}"),
            });
        }
        if let Some(parent) = target.parent()
            && let Err(error) = std::fs::create_dir_all(parent)
        {
            return Err(HttpError {
                url: url.to_string(),
                message: format!("cannot create {}: {error}", parent.display()),
            });
        }
        let mut file: File = match File::create(target) {
            Ok(created) => created,
            Err(error) => {
                return Err(HttpError {
                    url: url.to_string(),
                    message: format!("cannot create {}: {error}", target.display()),
                });
            }
        };
        let mut hasher: Sha256 = Sha256::new();
        let mut written: u64 = 0;
        let mut body = response.into_body();
        let started = std::time::Instant::now();
        loop {
            if started.elapsed() > Duration::from_secs(DOWNLOAD_TOTAL_TIMEOUT_SECONDS) {
                return Err(HttpError {
                    url: url.to_string(),
                    message: format!(
                        "the download took longer than {DOWNLOAD_TOTAL_TIMEOUT_SECONDS} seconds"
                    ),
                });
            }
            let next = match tokio::time::timeout(
                Duration::from_secs(DOWNLOAD_IDLE_TIMEOUT_SECONDS),
                body.frame(),
            )
            .await
            {
                Ok(frame) => frame,
                Err(_elapsed) => {
                    return Err(HttpError {
                        url: url.to_string(),
                        message: format!(
                            "the download stalled for {DOWNLOAD_IDLE_TIMEOUT_SECONDS} seconds"
                        ),
                    });
                }
            };
            let Some(received_frame) = next else {
                break;
            };
            let frame = match received_frame {
                Ok(received) => received,
                Err(error) => {
                    return Err(HttpError {
                        url: url.to_string(),
                        message: format!("reading the body failed: {error}"),
                    });
                }
            };
            if let Some(data) = frame.data_ref() {
                written += data.len() as u64;
                if written > max_bytes {
                    return Err(HttpError {
                        url: url.to_string(),
                        message: format!(
                            "the file is larger than the {max_bytes} bytes this download accepts"
                        ),
                    });
                }
                hasher.update(data);
                if let Err(error) = file.write_all(data) {
                    return Err(HttpError {
                        url: url.to_string(),
                        message: format!("cannot write {}: {error}", target.display()),
                    });
                }
            }
        }
        return Ok(hex_of_digest(hasher.finalize().as_slice()));
    }
}
