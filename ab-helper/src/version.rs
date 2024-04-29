use core::{
	fmt,
	str::FromStr,
};

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Version {
	pub major: u32,
	pub minor: u32,
	pub patch: u32,
	pub extra: Extra,
}

// The order of the variants is significant for ordering!
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Extra {
	Tag(String),
	Alpha(u32),
	Beta(u32),
	Rc(u32),
	None,
	Rev(u32),
}

impl FromStr for Version {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, ()> {
		Self::parse(s).ok_or(())
	}
}

impl From<Version> for String {
	fn from(v: Version) -> Self {
		v.to_string()
	}
}

impl Version {
	pub const MIN: Self = Self {
		major: 0,
		minor: 0,
		patch: 0,
		extra: Extra::Tag(String::new()),
	};
	pub const ZERO: Self = Self {
		major: 0,
		minor: 0,
		patch: 0,
		extra: Extra::None,
	};

	pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
		Self {
			major,
			minor,
			patch,
			extra: Extra::None,
		}
	}

	pub fn parse(s: &str) -> Option<Self> {
		if s.contains(|c: char| c.is_ascii_whitespace()) {
			return None;
		}
		let s = s.strip_prefix(['v', 'V']).unwrap_or(s);
		let (major, s) = parse_u32(s)?;

		if s.is_empty() {
			return Some(Self {
				major,
				..Self::ZERO
			});
		}
		let s = s.strip_prefix('.')?;

		let Some((minor, s)) = parse_u32(s) else {
			// Parse the rest as an extra
			return Some(Self {
				major,
				minor: 0,
				patch: 0,
				extra: parse_extra(s),
			});
		};

		if s.is_empty() {
			return Some(Self {
				major,
				minor,
				..Self::ZERO
			});
		}

		let (patch, extra) = match s.strip_prefix('.') {
			Some("") => return None,
			Some(s) => parse_u32(s)
				.map(|(patch, s)| (patch, s.strip_prefix(['.', '_', '-', '+']).unwrap_or(s)))
				.unwrap_or((0, s)),
			None => (0, s.strip_prefix(['-', '+', '_']).unwrap_or(s)),
		};

		Some(Self {
			major,
			minor,
			patch,
			extra: parse_extra(extra),
		})
	}

	pub const fn to_basic(&self) -> Self {
		Self {
			major: self.major,
			minor: self.minor,
			patch: self.patch,
			extra: Extra::None,
		}
	}

	pub fn is_pre(&self) -> bool {
		!matches!(self.extra, Extra::None | Extra::Rev(_))
	}
}

fn parse_u32(s: &str) -> Option<(u32, &str)> {
	let i = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
	s[..i].parse::<u32>().ok().map(|n| (n, &s[i..]))
}

fn parse_rev(s: &str) -> Option<u32> {
	const REV_PREFIXES: &[&str] = &["release", "rel", "revision", "rev", "r", "build", "b"];

	for p in REV_PREFIXES {
		if p.len() < s.len() && s[..p.len()].eq_ignore_ascii_case(p) {
			let s = &s[p.len()..];
			let s = s.strip_prefix(['.', '_', '+', '-']).unwrap_or(s);
			return s.parse::<u32>().ok();
		}
	}

	s.parse::<u32>().ok()
}

fn parse_extra(s: &str) -> Extra {
	if s.is_empty() {
		return Extra::None;
	}

	for (prefix, kind) in [
		("alpha", Extra::Alpha as fn(_) -> _),
		("beta", Extra::Beta),
		("rc", Extra::Rc),
		("pre", Extra::Rc),
	] {
		if s.len() >= prefix.len() && s[..prefix.len()].eq_ignore_ascii_case(prefix) {
			let s = &s[prefix.len()..];
			if s.is_empty() {
				return kind(0);
			}
			let s = s.strip_prefix(['.', '_', '-', '+']).unwrap_or(s);
			match s.parse::<u32>() {
				Ok(n) => return kind(n),
				Err(_) => break,
			}
		}
	}

	parse_rev(s)
		.map(Extra::Rev)
		.unwrap_or_else(|| Extra::Tag(s.to_owned()))
}

impl fmt::Display for Version {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let Self {
			major,
			minor,
			patch,
			..
		} = self;
		match &self.extra {
			Extra::None => write!(f, "{major}.{minor}.{patch}"),
			Extra::Tag(t) if t.is_empty() => write!(f, "{major}.{minor}.{patch}"),
			Extra::Tag(tag) => write!(f, "{major}.{minor}.{patch}-{tag}"),
			Extra::Rev(rev) => write!(f, "{major}.{minor}.{patch}-r+{rev}"),
			Extra::Alpha(alpha) => write!(f, "{major}.{minor}.{patch}-alpha{alpha}"),
			Extra::Beta(beta) => write!(f, "{major}.{minor}.{patch}-beta{beta}"),
			Extra::Rc(rc) => write!(f, "{major}.{minor}.{patch}-rc{rc}"),
		}
	}
}

#[cfg(test)]
mod tests {
	use Extra::*;

	use super::*;

	macro_rules! v {
		[$major:literal, $minor:literal, $patch:literal $(, extra = $extra:expr)?] => {{
			let mut v = Version::new($major, $minor, $patch);
			$(
				v.extra = $extra;
			)?
			v
		}};
		[$major:literal, $minor:literal $(, extra = $extra:expr)?] => (v!($major, $minor, 0 $(, extra = $extra)?));
		[$major:literal $(, extra = $extra:expr)?] => (v!($major, 0 $(, extra = $extra)?));
	}

	#[test]
	fn version_parse_and_format() {
		let tests = [
			("1", "1.0.0", v!(1)),
			("1.2", "1.2.0", v!(1, 2)),
			("1.2.3", "1.2.3", v!(1, 2, 3)),
			("v1", "1.0.0", v!(1)),
			("1.2-foo", "1.2.0-foo", v!(1, 2, extra = Tag("foo".into()))),
			(
				"1.2.3_foo",
				"1.2.3-foo",
				v!(1, 2, 3, extra = Tag("foo".into())),
			),
			("1.0.alpha", "1.0.0-alpha0", v!(1, extra = Alpha(0))),
			("1.2.3_beta.2", "1.2.3-beta2", v!(1, 2, 3, extra = Beta(2))),
			("1.2.3rc4", "1.2.3-rc4", v!(1, 2, 3, extra = Rc(4))),
			("1.2.3_pre+4", "1.2.3-rc4", v!(1, 2, 3, extra = Rc(4))),
			("1.0-r+3", "1.0.0-r+3", v!(1, extra = Rev(3))),
			("1.2.3rev.4", "1.2.3-r+4", v!(1, 2, 3, extra = Rev(4))),
			("1.2.3_build-4", "1.2.3-r+4", v!(1, 2, 3, extra = Rev(4))),
		];

		for (input, formatted, version) in tests {
			let got = Version::parse(input).unwrap();
			assert_eq!(version, got, "\n parse mismatch; input was {input}");
			assert_eq!(formatted, &got.to_string());
		}
	}

	#[test]
	fn version_ord() {
		let versions = [
			"0",
			"0.0",
			"0.0.0",
			"0.0.1",
			"0.1",
			"0.1.0",
			"0.1.1",
			"0.2.0",
			"1.0-asdf",
			"1.0-bcd",
			"1.0.alpha",
			"1.0.0-alpha1",
			"1.0alpha.2",
			"1.0beta",
			"1.0.0beta3",
			"1.0.0pre.0",
			"1.0.0rc+5",
			"1",
			"1.0",
			"1.0.0",
			"1.0.0r+0",
			"1.0release.2",
			"1.0.0+rev.3",
			"1.1",
			"2",
		];

		for (a, b) in versions.iter().zip(&versions[1..]) {
			let va = Version::parse(a).unwrap();
			let vb = Version::parse(b).unwrap();
			assert!(va <= vb, "\nordering error: {a:?} <= {b:?} == false",);
		}
	}
}
