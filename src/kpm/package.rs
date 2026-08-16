use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::manifest::parse_package;
use crate::Package;

use super::command::{combined_output, output_error, strip_ansi};
use super::KpmClient;

impl KpmClient {
    pub fn installed_packages(&self) -> Result<Vec<Package>, String> {
        read_installed_packages(&self.packages_dir)
    }

    pub fn search(
        &self,
        query: &str,
        progress: &mut dyn FnMut(&str),
    ) -> Result<Vec<Package>, String> {
        let output = self.run(&["search", query], progress)?;
        if !output.status.success() {
            return Err(output_error(&output));
        }
        let mut packages = parse_search_output(&combined_output(&output));
        if let Ok(versions) = self.available_versions() {
            for package in &mut packages {
                package.version = versions.get(&package.id).copied();
            }
        }
        Ok(packages)
    }

    pub fn install(&self, id: &str, progress: &mut dyn FnMut(&str)) -> Result<String, String> {
        self.successful_command(&["-y", "install", id], progress)
    }

    pub fn uninstall(&self, id: &str, progress: &mut dyn FnMut(&str)) -> Result<String, String> {
        self.successful_command(&["-y", "uninstall", id], progress)
    }

    pub fn upgrade(&self, progress: &mut dyn FnMut(&str)) -> Result<String, String> {
        self.successful_command(&["-y", "upgrade"], progress)
    }

    fn available_versions(&self) -> Result<BTreeMap<String, [u64; 3]>, String> {
        let output = Command::new("sqlite3")
            .arg("-separator")
            .arg("\t")
            .arg(&self.database)
            .arg(
                "SELECT id, version_major, version_minor, version_patch FROM artifacts \
                 ORDER BY id, version_major DESC, version_minor DESC, version_patch DESC;",
            )
            .output()
            .map_err(|error| format!("could not read KPM versions: {error}"))?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
        }
        Ok(parse_available_versions(&String::from_utf8_lossy(
            &output.stdout,
        )))
    }
}

fn read_installed_packages(root: &Path) -> Result<Vec<Package>, String> {
    if !root.exists() {
        return Ok(Vec::new());
    }

    let entries = fs::read_dir(root)
        .map_err(|error| format!("could not read {}: {error}", root.display()))?;
    let mut packages = Vec::new();
    let mut failures = Vec::new();
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                failures.push(error.to_string());
                continue;
            }
        };
        let manifest_path = entry.path().join("manifest.json");
        if !manifest_path.is_file() {
            continue;
        }
        match fs::read_to_string(&manifest_path)
            .map_err(|error| error.to_string())
            .and_then(|manifest| parse_package(&manifest))
        {
            Ok(package) => packages.push(package),
            Err(error) => failures.push(format!("{}: {error}", manifest_path.display())),
        }
    }
    sort_packages(&mut packages);
    if packages.is_empty() && !failures.is_empty() {
        Err(failures.join("\n"))
    } else {
        Ok(packages)
    }
}

fn parse_search_output(output: &str) -> Vec<Package> {
    let clean = strip_ansi(output);
    let mut packages = BTreeMap::new();
    for line in clean.lines() {
        let Some(line) = line.trim().strip_prefix("- ") else {
            continue;
        };
        let Some((id, rest)) = line.split_once(" (") else {
            continue;
        };
        let Some((name, description)) = rest.split_once("): ") else {
            continue;
        };
        if id.is_empty() || name.is_empty() {
            continue;
        }
        packages.entry(id.to_string()).or_insert_with(|| Package {
            id: id.to_string(),
            name: name.to_string(),
            author: String::new(),
            description: description.to_string(),
            version: None,
        });
    }
    let mut packages: Vec<_> = packages.into_values().collect();
    sort_packages(&mut packages);
    packages
}

fn parse_available_versions(output: &str) -> BTreeMap<String, [u64; 3]> {
    let mut versions = BTreeMap::new();
    for line in output.lines() {
        let mut fields = line.split('\t');
        let (Some(id), Some(major), Some(minor), Some(patch), None) = (
            fields.next(),
            fields.next().and_then(|value| value.parse().ok()),
            fields.next().and_then(|value| value.parse().ok()),
            fields.next().and_then(|value| value.parse().ok()),
            fields.next(),
        ) else {
            continue;
        };
        versions
            .entry(id.to_string())
            .or_insert([major, minor, patch]);
    }
    versions
}

fn sort_packages(packages: &mut [Package]) {
    packages.sort_by(|left, right| {
        left.name
            .to_lowercase()
            .cmp(&right.name.to_lowercase())
            .then_with(|| left.id.cmp(&right.id))
    });
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn parses_colored_search_output() {
        let output = "\u{1b}[36mFound 2 package(s) for :\u{1b}[0m\n\
            \u{1b}[32m  - koreader (KOReader): An e-book reader\u{1b}[0m\n\
              - helper_tools (Helper Tools): Device utilities\n";
        let packages = parse_search_output(output);

        assert_eq!(packages.len(), 2);
        assert_eq!(packages[0].id, "helper_tools");
        assert_eq!(packages[1].description, "An e-book reader");
    }

    #[test]
    fn reads_newest_available_versions() {
        let versions = parse_available_versions(
            "reader\t2\t1\t0\nreader\t2\t0\t0\ntools\t1\t4\t3\ninvalid\tx\t1\t0\n",
        );

        assert_eq!(versions.get("reader"), Some(&[2, 1, 0]));
        assert_eq!(versions.get("tools"), Some(&[1, 4, 3]));
        assert!(!versions.contains_key("invalid"));
    }

    #[test]
    fn reads_installed_manifests() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = env::temp_dir().join(format!("kpm-ui-test-{unique}"));
        let package_dir = root.join("sample");
        fs::create_dir_all(&package_dir).unwrap();
        fs::write(
            package_dir.join("manifest.json"),
            r#"{"id":"sample","name":"Sample","author":"Dev","description":"Test","version":[2,0,1]}"#,
        )
        .unwrap();

        let client = KpmClient::new("kpm", &root, "kpm.db");
        let packages = client.installed_packages().unwrap();
        fs::remove_dir_all(root).unwrap();

        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0].version_text(), "2.0.1");
    }
}
