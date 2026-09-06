use std::fs;

use zed_extension_api::{self as zed, serde_json, Result};

pub(super) fn absolute_path(path: &str) -> Result<String> {
    let cwd = std::env::current_dir().map_err(|e| {
        format!("Failed to resolve the extension's working directory ({e}). Fix: restart Zed and reopen the project.")
    })?;
    Ok(cwd.join(path).to_string_lossy().into_owned())
}

pub(super) fn remove_outdated_versions(
    language_server_id: &'static str,
    version_dir: &str,
) -> Result<()> {
    let entries = fs::read_dir(".").map_err(|e| {
        cleanup_error(
            language_server_id,
            &format!("failed to list the extension's working directory: {e}"),
        )
    })?;
    for entry in entries {
        let entry = entry.map_err(|e| {
            cleanup_error(
                language_server_id,
                &format!("failed to read a directory entry: {e}"),
            )
        })?;
        if entry.file_name().to_str().is_none_or(|file_name| {
            file_name.starts_with(language_server_id) && file_name != version_dir
        }) {
            fs::remove_dir_all(entry.path()).ok();
        }
    }
    Ok(())
}

fn cleanup_error(language_server_id: &str, detail: &str) -> String {
    format!(
        "Failed to clean up outdated '{language_server_id}-*' directories ({detail}). \
         Fix: close Zed, delete the outdated '{language_server_id}-*' directories in the \
         extension's working directory, and reopen the project."
    )
}

pub(super) fn binary_path_override_hint(
    binary_path_setting: &str,
    binary_description: &str,
) -> String {
    format!("set `{binary_path_setting}` to {binary_description} or an already-cached version directory")
}

pub(super) fn feed_error(source: &str, detail: &str, fix: &str) -> String {
    format!("Could not use {source} ({detail}); check network, proxy, or offline status. Fix: retry later, or {fix}.")
}

pub(super) fn download_error(what: &str, detail: &str, version_dir: &str, fix: &str) -> String {
    format!("Failed to download {what} ({detail}). Fix: delete the cached directory '{version_dir}' in the extension's working directory and retry, or {fix}.")
}

pub(super) fn layout_error(detail: &str, version_dir: &str, fix: &str) -> String {
    format!("The downloaded language server package has an unexpected layout ({detail}); it may be corrupt. Fix: delete the cached directory '{version_dir}' in the extension's working directory and retry, or {fix}.")
}

pub(super) fn package_layout_error(
    detail: &str,
    version_dir: &str,
    fix: &str,
    sdk_requirement: Option<&str>,
) -> String {
    append_sdk_requirement(layout_error(detail, version_dir, fix), sdk_requirement)
}

pub(super) fn sdk_requirement_from_global_json(global_json: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(global_json).ok()?;
    let version = value.get("sdk")?.get("version")?.as_str()?.trim();
    (!version.is_empty()).then(|| version.to_string())
}

pub(super) fn global_json_sdk_requirement(worktree: &zed::Worktree) -> Option<String> {
    let global_json = worktree.read_text_file("global.json").ok()?;
    sdk_requirement_from_global_json(&global_json)
}

pub(super) fn sdk_requirement_sentence(requirement: &str) -> String {
    format!("global.json requires SDK {requirement}; ensure an SDK satisfying it is installed.")
}

pub(super) fn append_sdk_requirement(message: String, requirement: Option<&str>) -> String {
    match requirement {
        Some(requirement) => format!("{message} {}", sdk_requirement_sentence(requirement)),
        None => message,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binary_path_override_hint_names_setting_and_binary() {
        let hint = binary_path_override_hint("lsp.roslyn.binary.path", "a local server binary");
        assert_eq!(
            hint,
            "set `lsp.roslyn.binary.path` to a local server binary or an already-cached version directory"
        );
    }

    #[test]
    fn feed_error_mentions_network_and_fix() {
        let message = feed_error(
            "the NuGet feed",
            "the service index could not be fetched: timeout",
            &binary_path_override_hint("lsp.roslyn.binary.path", "a local server binary"),
        );
        assert!(message.contains("Could not use the NuGet feed"));
        assert!(message.contains("the service index could not be fetched: timeout"));
        assert!(message.contains("network, proxy, or offline"));
        assert!(message.contains("Fix: retry later"));
        assert!(message.contains("lsp.roslyn.binary.path"));
        assert!(message.contains("already-cached version directory"));
    }

    #[test]
    fn download_error_names_cache_directory_and_fix() {
        let message = download_error(
            "NuGet package 'roslyn-language-server.osx-arm64' v5.0.0",
            "connection reset",
            "roslyn-5.0.0",
            &binary_path_override_hint("lsp.roslyn.binary.path", "a local server binary"),
        );
        assert!(message.contains(
            "Failed to download NuGet package 'roslyn-language-server.osx-arm64' v5.0.0"
        ));
        assert!(message.contains("connection reset"));
        assert!(message.contains("delete the cached directory 'roslyn-5.0.0'"));
        assert!(message.contains("lsp.roslyn.binary.path"));
    }

    #[test]
    fn layout_error_says_package_may_be_corrupt() {
        let message = layout_error(
            "no TFM directory found inside 'roslyn-5.0.0/tools'",
            "roslyn-5.0.0",
            &binary_path_override_hint("lsp.roslyn.binary.path", "a local server binary"),
        );
        assert!(message.contains("unexpected layout"));
        assert!(message.contains("it may be corrupt"));
        assert!(message.contains("no TFM directory found inside 'roslyn-5.0.0/tools'"));
        assert!(message.contains("delete the cached directory 'roslyn-5.0.0'"));
        assert!(message.contains("lsp.roslyn.binary.path"));
    }

    #[test]
    fn package_layout_error_appends_sdk_requirement_when_present() {
        let base = package_layout_error(
            "no TFM directory found inside 'roslyn-5.0.0/tools'",
            "roslyn-5.0.0",
            &binary_path_override_hint("lsp.roslyn.binary.path", "a local server binary"),
            None,
        );
        assert!(!base.contains("global.json"));
        let with_sdk = package_layout_error(
            "no TFM directory found inside 'roslyn-5.0.0/tools'",
            "roslyn-5.0.0",
            &binary_path_override_hint("lsp.roslyn.binary.path", "a local server binary"),
            Some("8.0.100"),
        );
        assert_eq!(
            with_sdk,
            format!("{base} global.json requires SDK 8.0.100; ensure an SDK satisfying it is installed.")
        );
    }

    #[test]
    fn global_json_valid_version_is_extracted() {
        let requirement = sdk_requirement_from_global_json(
            r#"{"sdk": {"version": "8.0.100", "rollForward": "latestFeature"}}"#,
        );
        assert_eq!(requirement.as_deref(), Some("8.0.100"));
    }

    #[test]
    fn global_json_version_is_trimmed() {
        let requirement =
            sdk_requirement_from_global_json(r#"{"sdk": {"version": "  9.0.100  "}}"#);
        assert_eq!(requirement.as_deref(), Some("9.0.100"));
    }

    #[test]
    fn global_json_with_extra_fields_is_tolerated() {
        let requirement = sdk_requirement_from_global_json(
            r#"{"projects": ["src/App"], "sdk": {"version": "8.0.100"}}"#,
        );
        assert_eq!(requirement.as_deref(), Some("8.0.100"));
    }

    #[test]
    fn global_json_missing_sdk_object_yields_none() {
        assert_eq!(sdk_requirement_from_global_json("{}"), None);
    }

    #[test]
    fn global_json_missing_version_field_yields_none() {
        assert_eq!(
            sdk_requirement_from_global_json(r#"{"sdk": {"rollForward": "latestMajor"}}"#),
            None
        );
    }

    #[test]
    fn global_json_non_string_version_yields_none() {
        assert_eq!(
            sdk_requirement_from_global_json(r#"{"sdk": {"version": 8}}"#),
            None
        );
    }

    #[test]
    fn global_json_empty_version_yields_none() {
        assert_eq!(
            sdk_requirement_from_global_json(r#"{"sdk": {"version": ""}}"#),
            None
        );
    }

    #[test]
    fn global_json_invalid_json_yields_none() {
        assert_eq!(sdk_requirement_from_global_json("not json at all"), None);
        assert_eq!(sdk_requirement_from_global_json("[1, 2, 3]"), None);
    }

    #[test]
    fn global_json_null_sdk_yields_none() {
        assert_eq!(sdk_requirement_from_global_json(r#"{"sdk": null}"#), None);
    }

    #[test]
    fn sdk_requirement_sentence_contains_version() {
        let sentence = sdk_requirement_sentence("8.0.100");
        assert_eq!(
            sentence,
            "global.json requires SDK 8.0.100; ensure an SDK satisfying it is installed."
        );
    }

    #[test]
    fn append_sdk_requirement_appends_only_when_present() {
        let base = "Some failure. Fix: do a thing.".to_string();
        assert_eq!(
            append_sdk_requirement(base.clone(), None),
            "Some failure. Fix: do a thing."
        );
        let appended = append_sdk_requirement(base, Some("8.0.100"));
        assert!(appended.starts_with("Some failure. Fix: do a thing. "));
        assert!(appended.contains("global.json requires SDK 8.0.100"));
    }
}
