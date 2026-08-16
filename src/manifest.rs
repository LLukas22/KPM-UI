use serde::Deserialize;

use crate::Package;

#[derive(Deserialize)]
struct PackageManifest {
    id: String,
    name: Option<String>,
    author: Option<String>,
    description: Option<String>,
    version: Option<[u64; 3]>,
}

pub fn parse_package(input: &str) -> Result<Package, String> {
    let manifest: PackageManifest = serde_json::from_str(input)
        .map_err(|error| format!("invalid package manifest: {error}"))?;
    Ok(Package {
        name: manifest.name.unwrap_or_else(|| manifest.id.clone()),
        id: manifest.id,
        author: manifest.author.unwrap_or_default(),
        description: manifest.description.unwrap_or_default(),
        version: manifest.version,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_kpm_manifest() {
        let package = parse_package(
            r#"{
                "manifest_version": 2,
                "id": "reader_tools",
                "name": "Reader Tools",
                "author": "A. Developer",
                "description": "Tools, notes, and \"extras\".",
                "version": [1, 4, 2],
                "dependencies": [],
                "supported_platforms": null
            }"#,
        )
        .unwrap();

        assert_eq!(package.id, "reader_tools");
        assert_eq!(package.description, "Tools, notes, and \"extras\".");
        assert_eq!(package.version, Some([1, 4, 2]));
    }

    #[test]
    fn defaults_optional_metadata() {
        let package = parse_package(r#"{"id":"minimal"}"#).unwrap();
        assert_eq!(package.name, "minimal");
        assert_eq!(package.author, "");
        assert_eq!(package.description, "");
        assert_eq!(package.version, None);
    }

    #[test]
    fn rejects_non_numeric_version() {
        let result = parse_package(r#"{"id":"bad","version":[1,"2",3]}"#);
        assert!(result.is_err());
    }

    #[test]
    fn rejects_invalid_json() {
        assert!(parse_package(r#"{"id":"bad",}"#).is_err());
    }
}
