//! Container log streaming functionality
//!
//! Provides async streaming of container logs using the Docker API.

use bollard::container::LogsOptions;
use bollard::Docker;
use futures::stream::BoxStream;
use futures::StreamExt;
use std::sync::Arc;

use crate::error::Result;

/// Streams logs from Docker containers
pub struct LogStreamer {
    docker_client: Arc<Docker>,
}

impl LogStreamer {
    /// Create a new log streamer
    pub fn new(docker_client: Arc<Docker>) -> Self {
        Self { docker_client }
    }

    /// Stream logs from a container
    ///
    /// # Arguments
    /// * `container_id` - The container ID or name
    /// * `tail` - Number of lines to initially fetch (e.g., 100 for last 100 lines)
    ///
    /// # Returns
    /// A stream of log lines as Strings
    pub async fn stream_logs(
        &self,
        container_id: &str,
        tail: usize,
    ) -> Result<BoxStream<'static, String>> {
        let options = LogsOptions::<String> {
            follow: true,
            stdout: true,
            stderr: true,
            tail: tail.to_string(),
            timestamps: false,
            ..Default::default()
        };

        let stream = self.docker_client.logs(container_id, Some(options));

        // Map the log output stream to strings, filtering and cleaning
        let cleaned_stream = stream
            .filter_map(|result| async move {
                match result {
                    Ok(log_output) => {
                        let msg = log_output.to_string();
                        // Strip ANSI color codes and other escape sequences
                        let cleaned = strip_ansi_codes(&msg);
                        if !cleaned.trim().is_empty() {
                            Some(cleaned)
                        } else {
                            None
                        }
                    }
                    Err(_) => None, // Ignore errors, just skip
                }
            })
            .boxed();

        Ok(cleaned_stream)
    }

    /// Fetch recent logs without streaming (one-time fetch)
    ///
    /// # Arguments
    /// * `container_id` - The container ID or name
    /// * `tail` - Number of lines to fetch
    ///
    /// # Returns
    /// A vector of log lines
    pub async fn fetch_recent_logs(&self, container_id: &str, tail: usize) -> Result<Vec<String>> {
        let options = LogsOptions::<String> {
            follow: false,
            stdout: true,
            stderr: true,
            tail: tail.to_string(),
            timestamps: false,
            ..Default::default()
        };

        let mut stream = self.docker_client.logs(container_id, Some(options));
        let mut logs = Vec::new();

        while let Some(result) = stream.next().await {
            match result {
                Ok(log_output) => {
                    let msg = log_output.to_string();
                    let cleaned = strip_ansi_codes(&msg);
                    if !cleaned.trim().is_empty() {
                        logs.push(cleaned);
                    }
                }
                Err(_) => continue,
            }
        }

        Ok(logs)
    }
}

/// Strip ANSI color codes and escape sequences from a string
fn strip_ansi_codes(s: &str) -> String {
    // Simple regex-free approach: remove sequences starting with ESC
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            // ESC character
            // Skip the escape sequence
            if chars.peek() == Some(&'[') {
                chars.next(); // consume '['
                // Skip until we hit a letter (the command character)
                while let Some(&next_ch) = chars.peek() {
                    chars.next();
                    if next_ch.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
        } else {
            result.push(ch);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_ansi_codes() {
        assert_eq!(strip_ansi_codes("hello"), "hello");
        assert_eq!(
            strip_ansi_codes("\x1b[31mRed text\x1b[0m"),
            "Red text"
        );
        assert_eq!(
            strip_ansi_codes("\x1b[1;32mBold green\x1b[0m normal"),
            "Bold green normal"
        );
        assert_eq!(
            strip_ansi_codes("Line 1\nLine 2"),
            "Line 1\nLine 2"
        );
    }
}
