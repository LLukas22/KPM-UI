use std::io::Read;
use std::process::{Command, Output, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use super::KpmClient;

impl KpmClient {
    pub(super) fn successful_command(
        &self,
        arguments: &[&str],
        progress: &mut dyn FnMut(&str),
    ) -> Result<String, String> {
        let output = self.run(arguments, progress)?;
        let combined = combined_output(&output);
        if output.status.success() {
            Ok(strip_ansi(&combined))
        } else {
            Err(output_error(&output))
        }
    }

    pub(super) fn run(
        &self,
        arguments: &[&str],
        progress: &mut dyn FnMut(&str),
    ) -> Result<Output, String> {
        let mut child = Command::new(&self.executable)
            .args(arguments)
            .env("TERM", "dumb")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| format!("could not run KPM: {error}"))?;

        let stdout = child.stdout.take().expect("piped stdout is available");
        let stderr = child.stderr.take().expect("piped stderr is available");
        let (sender, receiver) = mpsc::channel();
        read_output(stdout, true, sender.clone());
        read_output(stderr, false, sender);

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let mut last_progress = String::new();
        loop {
            match receiver.recv_timeout(Duration::from_millis(400)) {
                Ok((is_stdout, chunk)) => {
                    let output = if is_stdout { &mut stdout } else { &mut stderr };
                    output.extend_from_slice(&chunk);
                    if let Some(message) = latest_output_line(output) {
                        if message != last_progress {
                            progress(&message);
                            last_progress = message;
                        }
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => progress(""),
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }

        let status = child
            .wait()
            .map_err(|error| format!("could not wait for KPM: {error}"))?;
        Ok(Output {
            status,
            stdout,
            stderr,
        })
    }
}

pub(super) fn summarize_output(output: &str) -> String {
    let clean = strip_ansi(output);
    clean
        .lines()
        .map(str::trim)
        .rfind(|line| !line.is_empty())
        .unwrap_or("Operation completed")
        .to_string()
}

pub(super) fn progress_percent(message: &str) -> Option<f64> {
    let bytes = message.as_bytes();
    for (end, byte) in bytes.iter().enumerate() {
        if *byte != b'%' {
            continue;
        }
        let start = bytes[..end]
            .iter()
            .rposition(|byte| !byte.is_ascii_digit() && *byte != b'.')
            .map_or(0, |position| position + 1);
        if let Some(percent) = message
            .get(start..end)
            .and_then(|value| value.parse::<f64>().ok())
        {
            if (0.0..=100.0).contains(&percent) {
                return Some(percent);
            }
        }
    }
    None
}

pub(super) fn combined_output(output: &Output) -> String {
    format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    )
}

pub(super) fn output_error(output: &Output) -> String {
    let clean_output = strip_ansi(&combined_output(output));
    let detail = clean_output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("\n");
    if detail.is_empty() {
        format!("KPM exited with {}", output.status)
    } else {
        detail
    }
}

pub(super) fn strip_ansi(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut characters = input.chars().peekable();
    while let Some(character) = characters.next() {
        if character == '\u{1b}' {
            if characters.peek() == Some(&'[') {
                characters.next();
                for part in characters.by_ref() {
                    if ('@'..='~').contains(&part) {
                        break;
                    }
                }
            }
            continue;
        }
        output.push(character);
    }
    output
}

fn read_output<R: Read + Send + 'static>(
    mut output: R,
    is_stdout: bool,
    sender: mpsc::Sender<(bool, Vec<u8>)>,
) {
    thread::spawn(move || {
        let mut buffer = [0; 1024];
        while let Ok(read) = output.read(&mut buffer) {
            if read == 0 {
                break;
            }
            if sender.send((is_stdout, buffer[..read].to_vec())).is_err() {
                break;
            }
        }
    });
}

fn latest_output_line(output: &[u8]) -> Option<String> {
    let clean = strip_ansi(&String::from_utf8_lossy(output));
    clean
        .split(['\r', '\n'])
        .map(str::trim)
        .rfind(|line| !line.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ansi_stripping_preserves_unicode() {
        assert_eq!(strip_ansi("\u{1b}[31mVéra 日本語\u{1b}[0m"), "Véra 日本語");
    }

    #[test]
    fn reads_progress_percentage() {
        assert_eq!(progress_percent("Downloading package (42.5%)"), Some(42.5));
        assert_eq!(progress_percent("Installing package"), None);
        assert_eq!(progress_percent("Invalid 120%"), None);
    }
}
