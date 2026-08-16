mod command;
mod package;
mod repository;

use std::env;
use std::path::PathBuf;

use command::{progress_percent, summarize_output};

const DEFAULT_KPM_PATH: &str = "/var/local/kmc/bin/kpm";
const DEFAULT_PACKAGES_PATH: &str = "/mnt/us/kmc/kpm/packages";
const DEFAULT_DATABASE_PATH: &str = "/mnt/us/kmc/kpm/kpm.db";

/// Access to a KPM executable and its package state.
///
/// The executable must be built to use the same package and database paths
/// supplied to the client.
#[derive(Clone, Debug)]
pub struct KpmClient {
    executable: PathBuf,
    packages_dir: PathBuf,
    database: PathBuf,
}

impl KpmClient {
    pub const DEFAULT_REPOSITORY_ID: &'static str = "kindlemodding";

    pub fn new(
        executable: impl Into<PathBuf>,
        packages_dir: impl Into<PathBuf>,
        database: impl Into<PathBuf>,
    ) -> Self {
        Self {
            executable: executable.into(),
            packages_dir: packages_dir.into(),
            database: database.into(),
        }
    }

    /// Creates a client from the `KPM_UI_*` overrides used by the application.
    pub fn from_env() -> Self {
        let defaults = Self::default();
        Self {
            executable: env::var_os("KPM_UI_KPM")
                .map(PathBuf::from)
                .unwrap_or(defaults.executable),
            packages_dir: env::var_os("KPM_UI_PACKAGES_DIR")
                .map(PathBuf::from)
                .unwrap_or(defaults.packages_dir),
            database: env::var_os("KPM_UI_DB")
                .map(PathBuf::from)
                .unwrap_or(defaults.database),
        }
    }

    pub fn summarize_output(output: &str) -> String {
        summarize_output(output)
    }

    pub fn progress_percent(message: &str) -> Option<f64> {
        progress_percent(message)
    }
}

impl Default for KpmClient {
    fn default() -> Self {
        Self::new(
            DEFAULT_KPM_PATH,
            DEFAULT_PACKAGES_PATH,
            DEFAULT_DATABASE_PATH,
        )
    }
}
