use std::{
	env,
	fmt::{
		self,
	},
	thread,
	time::{
		Duration,
		Instant,
	},
};

use serde::Deserialize;

use super::prelude::*;

/// Query the Github API for the latest version of a repo
///
/// Set the $AB_GITHUB_TOKEN environment variable to a personal access token to reduce chances of getting rate limited and to reduce waiting time
#[derive(Clap)]
pub struct Gh {
	/// Repository: owner/repo
	#[arg(value_parser = parse_repo_ident)]
	repo: RepoIdent,
	/// The projects name as it appears on a release artifact
	#[arg(group = "filter")]
	name_in_release: Option<String>,
	/// Select release artifacts matching a regular expression; the strings <version> and <ext> will be expanded to a built-in regex pattern
	#[arg(short, long, group = "filter")]
	regex: Option<String>,
	/// Select release artifacts matching a glob pattern
	#[arg(short, long, group = "filter")]
	glob: Option<String>,
}

#[derive(Deserialize)]
struct GhResponse {
	tag_name: String,
	tarball_url: String,
	#[serde(default)]
	assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
	name: String,
	browser_download_url: String,
}

#[derive(Clone)]
struct RepoIdent {
	owner: String,
	repo: String,
}

impl fmt::Display for RepoIdent {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}/{}", self.owner, self.repo)
	}
}

fn parse_repo_ident(s: &str) -> Result<RepoIdent> {
	let (owner, repo) = s
		.split_once('/')
		.filter(|(a, b)| !a.is_empty() && !b.is_empty() && !b.contains('/'))
		.ok_or_else(|| anyhow!("value must be in the form OWNER/REPO"))?;
	Ok(RepoIdent {
		owner: owner.to_owned(),
		repo: repo.to_owned(),
	})
}

pub fn run(x: Gh, c: Agent) -> VersionResult {
	let filter = x
		.regex
		.as_deref()
		.map(Filter::regex)
		.or_else(|| x.glob.as_deref().map(Filter::glob))
		.transpose()?
		.or_else(|| x.name_in_release.map(Filter::file));

	let token = env::var("AB_GITHUB_TOKEN");
	let url = format!("https://api.github.com/repos/{}/releases/latest", x.repo);
	let mut req = c.get(&url).set("X-GitHub-Api-Version", "2022-11-28");
	if let Ok(token) = &token {
		req = req.set("Authorization", &format!("Bearer {token}"));
	}

	let t = Instant::now();
	let resp: GhResponse = serde_json::from_str(&req.call()?.into_string()?)?;
	let t = Instant::now().duration_since(t);

	// Rate limiting is less strict with personal access tokens so we can sleep less.
	let throttle = Duration::from_millis(if token.is_ok() { 490 } else { 990 });
	if t < throttle {
		thread::sleep(throttle - t);
	}

	let Some(filter) = filter else {
		return version_result(resp.tag_name, Url::parse(&resp.tarball_url)?);
	};

	for a in resp.assets {
		if filter.is_match(&a.name) {
			return version_result(resp.tag_name, a.browser_download_url);
		}
	}

	Err(NotFound.into())
}
