use std::fs;

use zed_extension_api::{self as zed, settings::LspSettings, LanguageServerId, Result};

use crate::language_servers::{nuget::NuGetClient, util};

const PACKAGE_ID: &str = "csharp-ls";
const SERVER_DLL: &str = "CSharpLanguageServer.dll";
const BINARY_PATH_SETTING: &str = "lsp.csharp-ls.binary.path";

fn fix_hint() -> String {
    util::binary_path_override_hint(BINARY_PATH_SETTING, "a local server binary")
}

fn dotnet_missing_error(sdk_requirement: Option<&str>) -> String {
    util::append_sdk_requirement(
        "Could not find the `dotnet` executable on PATH (csharp-ls runs via `dotnet exec`). \
         Fix: install the .NET SDK 10+ and reopen the project, or set `lsp.csharp-ls.binary.path` \
         to a standalone `csharp-ls` binary."
            .to_string(),
        sdk_requirement,
    )
}

pub struct CsharpLs {
    cached_dll_path: Option<String>,
    nuget: NuGetClient,
}

impl CsharpLs {
    pub const LANGUAGE_SERVER_ID: &'static str = "csharp-ls";
    const BINARY_NAME: &'static str = "csharp-ls";

    pub fn new() -> Self {
        Self {
            cached_dll_path: None,
            nuget: NuGetClient::new(BINARY_PATH_SETTING),
        }
    }

    pub fn language_server_cmd(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let binary_settings = LspSettings::for_worktree(Self::LANGUAGE_SERVER_ID, worktree)
            .ok()
            .and_then(|lsp_settings| lsp_settings.binary);
        let binary_args = binary_settings.as_ref().and_then(|b| b.arguments.clone());

        if let Some(path) = binary_settings.and_then(|b| b.path) {
            return Ok(zed::Command {
                command: path,
                args: binary_args.unwrap_or_default(),
                env: Default::default(),
            });
        }

        if let Some(path) = worktree.which(Self::BINARY_NAME) {
            return Ok(zed::Command {
                command: path,
                args: binary_args.unwrap_or_default(),
                env: Default::default(),
            });
        }

        if let Some(ref dll_path) = self.cached_dll_path {
            if fs::metadata(dll_path).is_ok_and(|s| s.is_file()) {
                return Self::dotnet_exec(worktree, dll_path, binary_args);
            }
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let sdk_requirement = util::global_json_sdk_requirement(worktree);
        let version = self.nuget.get_latest_version(PACKAGE_ID)?;
        let version_dir = format!("{}-{}", Self::LANGUAGE_SERVER_ID, version);

        if Self::find_dll(&version_dir, sdk_requirement.as_deref()).is_err() {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            self.nuget
                .download_and_extract(PACKAGE_ID, &version, &version_dir)
                .map_err(|e| util::append_sdk_requirement(e, sdk_requirement.as_deref()))?;

            util::remove_outdated_versions(Self::LANGUAGE_SERVER_ID, &version_dir)?;
        }

        let dll_path = Self::find_dll(&version_dir, sdk_requirement.as_deref())?;
        let command = Self::dotnet_exec(worktree, &dll_path, binary_args)?;
        self.cached_dll_path = Some(dll_path);
        Ok(command)
    }

    fn dotnet_exec(
        worktree: &zed::Worktree,
        dll_path: &str,
        user_args: Option<Vec<String>>,
    ) -> Result<zed::Command> {
        let dotnet = worktree.which("dotnet").ok_or_else(|| {
            dotnet_missing_error(util::global_json_sdk_requirement(worktree).as_deref())
        })?;
        let mut args = vec!["exec".to_string(), dll_path.to_string()];
        if let Some(user) = user_args {
            args.extend(user);
        }
        Ok(zed::Command {
            command: dotnet,
            args,
            env: Default::default(),
        })
    }

    fn find_dll(version_dir: &str, sdk_requirement: Option<&str>) -> Result<String> {
        let tools_dir = format!("{version_dir}/tools");

        let layout = |detail: String| {
            util::package_layout_error(&detail, version_dir, &fix_hint(), sdk_requirement)
        };

        let tfm = fs::read_dir(&tools_dir)
            .map_err(|e| layout(format!("failed to read tools directory '{tools_dir}': {e}")))?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                if entry.file_type().ok()?.is_dir() {
                    entry.file_name().into_string().ok()
                } else {
                    None
                }
            })
            .next()
            .ok_or_else(|| layout(format!("no TFM directory found inside '{tools_dir}'")))?;

        let dll_path = format!("{tools_dir}/{tfm}/any/{SERVER_DLL}");

        if fs::metadata(&dll_path).is_ok_and(|s| s.is_file()) {
            util::absolute_path(&dll_path)
        } else {
            Err(layout(format!("missing entry DLL at '{dll_path}'")))
        }
    }

    pub fn configuration_options(
        worktree: &zed::Worktree,
    ) -> Result<Option<zed::serde_json::Value>> {
        let settings = LspSettings::for_worktree(Self::LANGUAGE_SERVER_ID, worktree)
            .ok()
            .and_then(|lsp_settings| lsp_settings.settings);
        Ok(settings.map(|s| zed::serde_json::json!({ "csharp": s })))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dotnet_missing_error_names_remediations_without_sdk_requirement() {
        let message = dotnet_missing_error(None);
        assert!(message.starts_with(
            "Could not find the `dotnet` executable on PATH (csharp-ls runs via `dotnet exec`)."
        ));
        assert!(message.contains("Fix: install the .NET SDK 10+ and reopen the project"));
        assert!(
            message.contains("set `lsp.csharp-ls.binary.path` to a standalone `csharp-ls` binary")
        );
        assert!(!message.contains("global.json"));
    }

    #[test]
    fn dotnet_missing_error_appends_sdk_requirement_when_present() {
        let message = dotnet_missing_error(Some("9.0.100"));
        assert!(message.ends_with(
            "global.json requires SDK 9.0.100; ensure an SDK satisfying it is installed."
        ));
    }

    #[test]
    fn fix_hint_names_the_csharp_ls_setting() {
        assert_eq!(
            fix_hint(),
            "set `lsp.csharp-ls.binary.path` to a local server binary or an already-cached version directory"
        );
    }

    #[test]
    fn find_dll_layout_errors_embed_version_dir_and_setting() {
        // Mirrors the layout closure inside find_dll: the version directory
        // embeds the resolved version and the binary-path escape must appear.
        let version_dir = format!("{}-1.2.3", CsharpLs::LANGUAGE_SERVER_ID);
        let detail = format!("missing entry DLL at '{version_dir}/tools/net8.0/any/{SERVER_DLL}'");
        let message =
            util::package_layout_error(&detail, &version_dir, &fix_hint(), Some("8.0.100"));
        assert!(message.contains("unexpected layout"));
        assert!(message.contains("csharp-ls-1.2.3"));
        assert!(message.contains("lsp.csharp-ls.binary.path"));
        assert!(message.contains("global.json requires SDK 8.0.100"));
    }
}
