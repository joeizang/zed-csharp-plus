use std::fs;
use zed_extension_api::{self as zed, settings::LspSettings, LanguageServerId, Result};

use crate::language_servers::util;

const BINARY_PATH_SETTING: &str = "lsp.omnisharp.binary.path";
const GITHUB_REPO: &str = "OmniSharp/omnisharp-roslyn";

fn fix_hint() -> String {
    util::binary_path_override_hint(BINARY_PATH_SETTING, "a local server binary")
}

fn asset_error(asset_name: &str, release_version: &str) -> String {
    format!(
        "No release asset named '{asset_name}' was found in {GITHUB_REPO} release {release_version}; \
         your platform may not be supported by this release. Fix: {}.",
        fix_hint()
    )
}

pub struct Omnisharp {
    cached_binary_path: Option<String>,
}

pub struct OmnisharpBinary {
    pub path: String,
    pub args: Option<Vec<String>>,
}

impl Omnisharp {
    pub const LANGUAGE_SERVER_ID: &'static str = "omnisharp";

    pub fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    pub fn language_server_binary(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<OmnisharpBinary> {
        let binary_settings = LspSettings::for_worktree("omnisharp", worktree)
            .ok()
            .and_then(|lsp_settings| lsp_settings.binary);
        let binary_args = binary_settings
            .as_ref()
            .and_then(|binary_settings| binary_settings.arguments.clone());

        if let Some(path) = binary_settings.and_then(|binary_settings| binary_settings.path) {
            return Ok(OmnisharpBinary {
                path,
                args: binary_args,
            });
        }

        if let Some(path) = worktree.which("OmniSharp") {
            return Ok(OmnisharpBinary {
                path,
                args: binary_args,
            });
        }

        if let Some(path) = &self.cached_binary_path {
            if fs::metadata(path).is_ok_and(|stat| stat.is_file()) {
                return Ok(OmnisharpBinary {
                    path: path.clone(),
                    args: binary_args,
                });
            }
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );
        let release = zed::latest_github_release(
            GITHUB_REPO,
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )
        .map_err(|e| {
            util::feed_error(
                &format!("the GitHub release list for '{GITHUB_REPO}'"),
                &e,
                &fix_hint(),
            )
        })?;

        let (platform, arch) = zed::current_platform();
        let asset_name = format!(
            "omnisharp-{os}-{arch}-net6.0.{extension}",
            os = match platform {
                zed::Os::Mac => "osx",
                zed::Os::Linux => "linux",
                zed::Os::Windows => "win",
            },
            arch = match arch {
                zed::Architecture::Aarch64 => "arm64",
                zed::Architecture::X86 => "x86",
                zed::Architecture::X8664 => "x64",
            },
            extension = match platform {
                zed::Os::Mac | zed::Os::Linux => "tar.gz",
                zed::Os::Windows => "zip",
            }
        );

        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| asset_error(&asset_name, &release.version))?;

        let version_dir = format!("{}-{}", Self::LANGUAGE_SERVER_ID, release.version);
        let binary_path = match platform {
            zed::Os::Windows => format!("{version_dir}/OmniSharp.exe"),
            _ => format!("{version_dir}/OmniSharp"),
        };

        if !fs::metadata(&binary_path).is_ok_and(|stat| stat.is_file()) {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            zed::download_file(
                &asset.download_url,
                &version_dir,
                match platform {
                    zed::Os::Mac | zed::Os::Linux => zed::DownloadedFileType::GzipTar,
                    zed::Os::Windows => zed::DownloadedFileType::Zip,
                },
            )
            .map_err(|e| {
                util::download_error(
                    &format!(
                        "the {GITHUB_REPO} release asset '{asset_name}' v{}",
                        release.version
                    ),
                    &e,
                    &version_dir,
                    &fix_hint(),
                )
            })?;

            util::remove_outdated_versions(Self::LANGUAGE_SERVER_ID, &version_dir)?;
        }

        self.cached_binary_path = Some(binary_path.clone());
        Ok(OmnisharpBinary {
            path: binary_path,
            args: binary_args,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fix_hint_names_the_omnisharp_setting() {
        assert_eq!(
            fix_hint(),
            "set `lsp.omnisharp.binary.path` to a local server binary or an already-cached version directory"
        );
    }

    #[test]
    fn asset_error_names_asset_release_version_and_escape() {
        let message = asset_error("omnisharp-osx-arm64-net6.0.tar.gz", "v1.2.3");
        assert!(message.starts_with(
            "No release asset named 'omnisharp-osx-arm64-net6.0.tar.gz' was found in OmniSharp/omnisharp-roslyn release v1.2.3"
        ));
        assert!(message.contains("your platform may not be supported"));
        assert!(message.contains("Fix: set `lsp.omnisharp.binary.path`"));
    }

    #[test]
    fn download_errors_name_asset_version_and_cache_directory() {
        let message = util::download_error(
            "the OmniSharp/omnisharp-roslyn release asset 'omnisharp-win-x64-net6.0.zip' v1.2.3",
            "connection reset",
            "omnisharp-1.2.3",
            &fix_hint(),
        );
        assert!(
            message.starts_with("Failed to download the OmniSharp/omnisharp-roslyn release asset")
        );
        assert!(message.contains("v1.2.3"));
        assert!(message.contains("connection reset"));
        assert!(message.contains("delete the cached directory 'omnisharp-1.2.3'"));
        assert!(message.contains("lsp.omnisharp.binary.path"));
    }
}
