use anyhow::{
	bail,
	Result,
};
use globset::{
	GlobBuilder,
	GlobSet,
};
use regex::Regex;

use crate::template::Template;

const VERSION_RE: &str = r"[vV]?(\d+(?:\.\d+(?:\.\d+)?)?[a-zA-Z0-9_\+\-\.]*)";
const EXT_RE: &str = r"\.(?:(?:tar\.|t)(?:gz|bz|bz2|xz|lz|lz4|lz2|lzma|lzma2|zst|zstd))";

pub enum Filter {
	Any,
	Regex(Regex),
	Glob {
		accept: Option<GlobSet>,
		deny: Option<GlobSet>,
	},
	/// Strings in the form <prefix>-<version><ext>
	File {
		name: String,
	},
}

impl Filter {
	pub fn glob<S: AsRef<str>>(s: S) -> Result<Self> {
		let s = s.as_ref();
		let g = |s| {
			GlobBuilder::new(s)
				.literal_separator(false)
				.backslash_escape(true)
				.empty_alternates(true)
				.case_insensitive(false)
				.build()
		};

		let mut deny = GlobSet::builder();
		let mut accept = GlobSet::builder();

		for pat in s.split('|') {
			if let Some(pat) = pat.strip_prefix('!') {
				deny.add(g(pat)?);
			} else {
				accept.add(g(pat)?);
			}
		}

		let accept = accept.build();
		let deny = deny.build();
		if accept.is_err() && deny.is_err() {
			bail!("invalid glob pattern: {s}");
		}
		Ok(Self::Glob {
			deny: deny.ok().filter(|g| !g.is_empty()),
			accept: accept.ok().filter(|g| !g.is_empty()),
		})
	}

	pub fn regex<S: Into<String> + AsRef<str>>(s: S) -> Result<Self> {
		use std::borrow::Cow::*;
		let s = Template::new(s, "<", ">").expand(|s| match s {
			"version" => Borrowed(VERSION_RE),
			"ext" => Borrowed(EXT_RE),
			_ => Owned(format!("<{s}>")),
		});

		Regex::new(&s).map(Self::Regex).map_err(|e| e.into())
	}

	pub fn file<S: Into<String>>(prefix: S) -> Self {
		Self::File {
			name: prefix.into(),
		}
	}

	pub fn is_match(&self, s: &str) -> bool {
		match self {
			Self::Any => true,
			Self::Glob { accept, deny } => match (accept, deny) {
				(Some(a), None) => a.is_match(s),
				(None, Some(d)) => !d.is_match(s),
				(Some(a), Some(d)) => !d.is_match(s) && a.is_match(s),
				(None, None) => true,
			},
			Self::Regex(r) => r.is_match(s),
			Self::File { name } => super::extract_version(s, name).is_some(),
		}
	}
}
