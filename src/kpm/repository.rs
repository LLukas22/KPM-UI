use std::collections::BTreeMap;

use crate::Repository;

use super::command::{combined_output, output_error, strip_ansi};
use super::KpmClient;

impl KpmClient {
    pub fn update(&self, progress: &mut dyn FnMut(&str)) -> Result<String, String> {
        self.successful_command(&["update"], progress)
    }

    pub fn repositories(&self, progress: &mut dyn FnMut(&str)) -> Result<Vec<Repository>, String> {
        let output = self.run(&["list-repo"], progress)?;
        if !output.status.success() {
            return Err(output_error(&output));
        }
        Ok(parse_repository_output(&combined_output(&output)))
    }

    pub fn add_repository(
        &self,
        url: &str,
        progress: &mut dyn FnMut(&str),
    ) -> Result<String, String> {
        repository_change("add-repo", url, |arguments| {
            self.successful_command(arguments, progress)
        })
    }

    pub fn remove_repository(
        &self,
        id: &str,
        progress: &mut dyn FnMut(&str),
    ) -> Result<String, String> {
        if id == Self::DEFAULT_REPOSITORY_ID {
            return Err("the default repository cannot be removed".to_string());
        }
        repository_change("remove-repo", id, |arguments| {
            self.successful_command(arguments, progress)
        })
    }
}

fn repository_change<F>(command: &str, value: &str, mut execute: F) -> Result<String, String>
where
    F: FnMut(&[&str]) -> Result<String, String>,
{
    let changed = execute(&[command, value])?;
    let updated = execute(&["update"])?;
    Ok(format!("{changed}\n{updated}"))
}

fn parse_repository_output(output: &str) -> Vec<Repository> {
    let clean = strip_ansi(output);
    let mut repositories = BTreeMap::new();
    for line in clean.lines() {
        let Some(line) = line.trim().strip_prefix("- ") else {
            continue;
        };
        let Some((id, rest)) = line.split_once(" - ") else {
            continue;
        };
        let Some((name, url)) = rest.rsplit_once(" (") else {
            continue;
        };
        let Some(url) = url.strip_suffix(')') else {
            continue;
        };
        if id.is_empty() || name.is_empty() || url.is_empty() {
            continue;
        }
        repositories
            .entry(id.to_string())
            .or_insert_with(|| Repository {
                id: id.to_string(),
                name: name.to_string(),
                url: url.to_string(),
            });
    }
    let mut repositories: Vec<_> = repositories.into_values().collect();
    repositories.sort_by(|left, right| {
        left.name
            .to_lowercase()
            .cmp(&right.name.to_lowercase())
            .then_with(|| left.id.cmp(&right.id))
    });
    repositories
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adding_repository_updates_the_index() {
        let mut calls: Vec<Vec<String>> = Vec::new();
        let output = repository_change(
            "add-repo",
            "https://example.com/manifest.json",
            |arguments| {
                calls.push(arguments.iter().map(|value| value.to_string()).collect());
                Ok(arguments[0].to_string())
            },
        )
        .unwrap();

        assert_eq!(
            calls,
            vec![
                vec![
                    "add-repo".to_string(),
                    "https://example.com/manifest.json".to_string()
                ],
                vec!["update".to_string()]
            ]
        );
        assert_eq!(output, "add-repo\nupdate");
    }

    #[test]
    fn parses_repository_list() {
        let output = "Repositories:\n\
            \u{1b}[32m  - kindlemodding - Official KMC Repo (https://repo.kindlemodding.org/manifest.v2.json)\u{1b}[0m\n\
              - community - Community Packages (https://example.com/manifest.json)\n";
        let repositories = parse_repository_output(output);

        assert_eq!(repositories.len(), 2);
        assert_eq!(repositories[0].id, "community");
        assert_eq!(repositories[1].name, "Official KMC Repo");
    }

    #[test]
    fn refuses_to_remove_default_repository() {
        let client = KpmClient::default();
        let mut ignore = |_: &str| {};
        assert_eq!(
            client.remove_repository(KpmClient::DEFAULT_REPOSITORY_ID, &mut ignore),
            Err("the default repository cannot be removed".to_string())
        );
    }
}
