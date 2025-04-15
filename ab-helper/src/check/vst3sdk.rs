use super::prelude::*;

#[derive(Clap)]
/// Find the latest Steinberg VST3 SDK
pub struct Vst3Sdk;

pub fn run(_: Vst3Sdk, c: Agent) -> VersionResult {
	let resp = c.head("https://www.steinberg.net/vst3sdk").call()?;
	let url = resp.get_url();

	let version = url
		.strip_prefix("https://download.steinberg.net/sdk_downloads/vst-sdk_")
		.and_then(|s| s.strip_suffix(".zip"))
		.ok_or(NotFound)?
		.replace("_build-", "-");

	version_result(version, url)
}
