use std::cmp::Ordering;

use zed_extension_api::{self as zed, http_client, serde_json, Result};

use crate::language_servers::util;

const ROSLYN_NUGET_FEED_INDEX: &str = "https://api.nuget.org/v3/index.json";

pub struct NuGetClient {
    package_base_address: Option<String>,
    binary_path_setting: &'static str,
}

impl NuGetClient {
    pub fn new(binary_path_setting: &'static str) -> Self {
        NuGetClient {
            package_base_address: None,
            binary_path_setting,
        }
    }

    fn fix_hint(&self) -> String {
        util::binary_path_override_hint(self.binary_path_setting, "a local server binary")
    }

    fn feed_error(&self, detail: &str) -> String {
        util::feed_error("the NuGet feed", detail, &self.fix_hint())
    }

    fn ensure_package_base_address(&mut self) -> Result<String> {
        if let Some(ref base) = self.package_base_address {
            return Ok(base.clone());
        }

        let response = http_client::fetch(
            &http_client::HttpRequest::builder()
                .method(http_client::HttpMethod::Get)
                .url(ROSLYN_NUGET_FEED_INDEX)
                .redirect_policy(http_client::RedirectPolicy::FollowAll)
                .build()
                .map_err(|e| {
                    self.feed_error(&format!(
                        "the service index request to '{ROSLYN_NUGET_FEED_INDEX}' could not be built: {e}"
                    ))
                })?,
        )
        .map_err(|e| {
            self.feed_error(&format!(
                "the service index at '{ROSLYN_NUGET_FEED_INDEX}' could not be fetched: {e}"
            ))
        })?;

        let index: serde_json::Value = serde_json::from_slice(&response.body)
            .map_err(|e| self.feed_error(&format!("the service index could not be parsed: {e}")))?;

        let base_url = index["resources"]
            .as_array()
            .ok_or_else(|| self.feed_error("the service index contains no 'resources' array"))?
            .iter()
            .find(|r| {
                r["@type"]
                    .as_str()
                    .is_some_and(|t| t == "PackageBaseAddress/3.0.0")
            })
            .and_then(|r| r["@id"].as_str())
            .ok_or_else(|| {
                self.feed_error(
                    "the service index does not list a 'PackageBaseAddress/3.0.0' resource",
                )
            })?
            .trim_end_matches('/')
            .to_string();

        self.package_base_address = Some(base_url.clone());
        Ok(base_url)
    }

    pub fn get_latest_version(&mut self, package_id: &str) -> Result<String> {
        let base = self.ensure_package_base_address()?;
        let lower_id = package_id.to_lowercase();

        let url = format!("{base}/{lower_id}/index.json");
        let response = http_client::fetch(
            &http_client::HttpRequest::builder()
                .method(http_client::HttpMethod::Get)
                .url(&url)
                .redirect_policy(http_client::RedirectPolicy::FollowAll)
                .build()
                .map_err(|e| {
                    self.feed_error(&format!(
                        "the version index request for '{package_id}' could not be built: {e}"
                    ))
                })?,
        )
        .map_err(|e| {
            self.feed_error(&format!(
                "the version index for '{package_id}' could not be fetched: {e}"
            ))
        })?;

        let body: serde_json::Value = serde_json::from_slice(&response.body).map_err(|e| {
            self.feed_error(&format!(
                "the version index for '{package_id}' could not be parsed: {e}"
            ))
        })?;

        let versions = body["versions"].as_array().ok_or_else(|| {
            self.feed_error(&format!(
                "the version index for '{package_id}' contains no 'versions' array"
            ))
        })?;

        versions
            .iter()
            .filter_map(|v| v.as_str())
            .filter_map(NuGetVersion::parse)
            .max()
            .map(|v| v.raw)
            .ok_or_else(|| {
                self.feed_error(&format!(
                    "'{package_id}' has no parseable published versions"
                ))
            })
    }

    pub fn download_and_extract(
        &mut self,
        package_id: &str,
        version: &str,
        dest_dir: &str,
    ) -> Result<()> {
        let base = self.ensure_package_base_address()?;
        let lower_id = package_id.to_lowercase();
        let lower_version = version.to_lowercase();

        let url = format!("{base}/{lower_id}/{lower_version}/{lower_id}.{lower_version}.nupkg");

        zed::download_file(&url, dest_dir, zed::DownloadedFileType::Zip).map_err(|e| {
            util::download_error(
                &format!("NuGet package '{package_id}' v{version}"),
                &e,
                dest_dir,
                &self.fix_hint(),
            )
        })
    }
}

#[derive(Debug, Clone)]
struct NuGetVersion {
    major: u64,
    minor: u64,
    patch: u64,
    revision: u64,
    prerelease: Option<String>,
    raw: String,
}

impl PartialEq for NuGetVersion {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for NuGetVersion {}

impl NuGetVersion {
    fn parse(input: &str) -> Option<Self> {
        let (core, prerelease) = match input.split_once('-') {
            Some((c, p)) => (c, Some(p.to_string())),
            None => (input, None),
        };

        let segments: Vec<u64> = core
            .split('.')
            .map(|s| s.parse::<u64>().ok())
            .collect::<Option<Vec<_>>>()?;

        let (major, minor, patch, revision) = match segments[..] {
            [major] => (major, 0, 0, 0),
            [major, minor] => (major, minor, 0, 0),
            [major, minor, patch] => (major, minor, patch, 0),
            [major, minor, patch, revision] => (major, minor, patch, revision),
            _ => return None,
        };

        Some(NuGetVersion {
            major,
            minor,
            patch,
            revision,
            prerelease,
            raw: input.to_string(),
        })
    }
}

fn cmp_prerelease_token(a: &str, b: &str) -> Ordering {
    match (a.parse::<u64>(), b.parse::<u64>()) {
        (Ok(na), Ok(nb)) => na.cmp(&nb),
        (Ok(_), Err(_)) => Ordering::Less,
        (Err(_), Ok(_)) => Ordering::Greater,
        (Err(_), Err(_)) => a.to_ascii_lowercase().cmp(&b.to_ascii_lowercase()),
    }
}

impl Ord for NuGetVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        self.major
            .cmp(&other.major)
            .then(self.minor.cmp(&other.minor))
            .then(self.patch.cmp(&other.patch))
            .then(self.revision.cmp(&other.revision))
            .then_with(|| match (&self.prerelease, &other.prerelease) {
                (None, None) => Ordering::Equal,
                (None, Some(_)) => Ordering::Greater,
                (Some(_), None) => Ordering::Less,
                (Some(a), Some(b)) => {
                    let mut a_parts = a.split('.');
                    let mut b_parts = b.split('.');
                    loop {
                        match (a_parts.next(), b_parts.next()) {
                            (Some(at), Some(bt)) => {
                                let ord = cmp_prerelease_token(at, bt);
                                if ord != Ordering::Equal {
                                    return ord;
                                }
                            }
                            (None, Some(_)) => return Ordering::Less,
                            (Some(_), None) => return Ordering::Greater,
                            (None, None) => return Ordering::Equal,
                        }
                    }
                }
            })
    }
}

impl PartialOrd for NuGetVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> NuGetVersion {
        NuGetVersion::parse(input).expect("version should parse")
    }

    #[test]
    fn parses_two_to_four_segments() {
        assert_eq!(parse("1").raw, "1");
        assert_eq!(parse("1.2").raw, "1.2");
        assert_eq!(parse("1.2.3").raw, "1.2.3");
        assert_eq!(parse("1.2.3.4").raw, "1.2.3.4");
        assert_eq!(parse("1.2.3-rc.1").raw, "1.2.3-rc.1");
        assert_eq!(parse("1.2.3.4-preview").raw, "1.2.3.4-preview");
    }

    #[test]
    fn rejects_unparseable_versions() {
        assert!(NuGetVersion::parse("").is_none());
        assert!(NuGetVersion::parse("abc").is_none());
        assert!(NuGetVersion::parse("1.x.3").is_none());
        assert!(NuGetVersion::parse("1.2.3.4.5").is_none());
    }

    #[test]
    fn orders_by_core_segments() {
        assert!(parse("1.2.3") < parse("1.2.4"));
        assert!(parse("1.2.9") < parse("1.3.0"));
        assert!(parse("1.9.9.9") < parse("2.0.0"));
        assert!(parse("1.2.3.9") < parse("1.2.4"));
        assert!(parse("1.2.3.4") < parse("1.2.3.5"));
    }

    #[test]
    fn prerelease_orders_below_release() {
        assert!(parse("1.0.0-rc.1") < parse("1.0.0"));
        assert!(parse("1.0.0-beta") < parse("1.0.0-rc.1"));
    }

    #[test]
    fn prerelease_tokens_compare_numerically_then_alphabetically() {
        assert!(parse("1.0.0-beta.1") < parse("1.0.0-beta.2"));
        assert!(parse("1.0.0-alpha") < parse("1.0.0-beta"));
        assert!(parse("1.0.0-1") < parse("1.0.0-alpha"));
        assert!(parse("1.0.0-RC1") == parse("1.0.0-rc1"));
    }

    #[test]
    fn shorter_prerelease_orders_below_longer_equal_prefix() {
        assert!(parse("1.0.0-beta") < parse("1.0.0-beta.1"));
        assert!(parse("1.0.0-beta.1") < parse("1.0.0-beta.1.1"));
    }

    #[test]
    fn max_picks_highest_version_ignoring_none() {
        let versions = [
            "5.0.0-preview.1",
            "4.2.1",
            "5.0.0",
            "not-a-version",
            "5.0.0-rc.2",
        ];
        let highest = versions
            .iter()
            .filter_map(|v| NuGetVersion::parse(v))
            .max()
            .map(|v| v.raw);
        assert_eq!(highest.as_deref(), Some("5.0.0"));
    }

    #[test]
    fn equality_is_total_order_equality() {
        assert_eq!(parse("1.2.3"), parse("1.2.3.0"));
        assert_ne!(parse("1.2.3"), parse("1.2.3-rc.1"));
    }
}
