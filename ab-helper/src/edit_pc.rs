use std::{
	collections::{
		btree_map::Entry,
		BTreeMap,
		BTreeSet,
	},
	fmt::Write as FmtWrite,
	fs,
	path::PathBuf,
};

use anyhow::{
	anyhow,
	bail,
	Result,
};
use clap::Parser as Clap;
use log::{
	info,
	warn,
};

#[derive(Clap)]
/// Edit a pkg-config .pc file
pub struct EditPc {
	/// Path to the .pc file
	#[arg(short, long)]
	path: PathBuf,
	/// Print the modified file to stdout instead of saving to disk
	#[arg(long)]
	print: bool,
	/// List of actions to perform
	#[arg(value_parser = parse_action)]
	actions: Vec<Action>,
}

#[derive(Debug, Clone)]
enum Action {
	SetVar(String, String),
	StringField {
		name: &'static str,
		value: String,
		if_missing: bool,
	},
	ListField {
		name: &'static str,
		separator: Separator,
		op: ListOp,
		value: Vec<String>,
	},
}

#[derive(Debug, Copy, Clone)]
enum ListOp {
	Append,
	AppendLeft,
	Remove,
	Set,
	SetIfMissing,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Separator {
	Space,
	Comma,
}

fn parse_list_field_name(s: &str) -> Option<(&'static str, Separator, &str)> {
	["Cflags.private", "Cflags", "Libs.private", "Libs"]
		.into_iter()
		.find_map(|field| s.strip_prefix(field).map(|s| (field, Separator::Space, s)))
		.or_else(|| {
			["Conflicts", "Requires.private", "Requires"]
				.into_iter()
				.find_map(|field| s.strip_prefix(field).map(|s| (field, Separator::Comma, s)))
		})
}

fn parse_string_field_name(s: &str) -> Option<(&'static str, &str)> {
	["Description", "Name", "URL", "Version"]
		.into_iter()
		.find_map(|field| s.strip_prefix(field).map(|s| (field, s)))
}

fn parse_var_name(s: &str) -> Option<(&str, &str)> {
	if !s.starts_with(|c: char| c.is_alphabetic() || c == '_') {
		return None;
	}

	let i = s
		.find(|c: char| c != '_' && !c.is_alphanumeric())
		.unwrap_or(s.len());
	Some((&s[..i], &s[i..]))
}

fn parse_action(s: &str) -> Result<Action> {
	// Is it a variable operation?
	if let Some(s) = s.strip_prefix('$') {
		let (name, s) = parse_var_name(s).ok_or_else(|| anyhow!("illegal variable name"))?;
		let value = s
			.trim_start()
			.strip_prefix('=')
			.ok_or_else(|| anyhow!("variable name must be followed by ="))?;
		return Ok(Action::SetVar(name.to_owned(), value.trim().to_owned()));
	}

	// It's an operation on a field.
	if let Some((field, s)) = parse_string_field_name(s) {
		let s = s.trim_start();
		let (if_missing, s) = if let Some(s) = s.strip_prefix('=') {
			(false, s)
		} else if let Some(s) = s.strip_prefix("?=") {
			(true, s)
		} else {
			bail!("expected = or ?= after {field}");
		};
		return Ok(Action::StringField {
			name: field,
			value: s.trim().to_owned(),
			if_missing,
		});
	}

	if let Some((field, separator, s)) = parse_list_field_name(s) {
		let s = s.trim_start();
		let (op, s) = [
			("=", ListOp::Set),
			("?=", ListOp::SetIfMissing),
			("+=", ListOp::Append),
			("<+=", ListOp::AppendLeft),
			("-=", ListOp::Remove),
		]
		.into_iter()
		.find_map(|(op_str, op)| s.strip_prefix(op_str).map(|s| (op, s)))
		.ok_or_else(|| anyhow!("expected one of =, ?=, +=, <+= or -= after {field}"))?;
		let value = match separator {
			Separator::Space => s.split_whitespace().map(str::to_owned).collect(),
			Separator::Comma => s.trim().split(',').map(|x| x.trim().to_owned()).collect(),
		};

		return Ok(Action::ListField {
			name: field,
			separator,
			op,
			value,
		});
	}

	Err(anyhow!("unknown field name; if you meant to specify a variable, precede the variable name with a dollar sign ($)"))
}

#[derive(Debug, Clone)]
enum Pc<'a> {
	Var(&'a str, &'a str),
	StringField(&'static str, &'a str),
	ListField(&'static str, Separator, Vec<&'a str>),
	Other(&'a str),
}

fn parse_pc(contents: &str) -> Result<Vec<Pc<'_>>> {
	let mut defines = BTreeSet::new();
	let mut pc = Vec::with_capacity(64);

	for s in contents.lines().map(str::trim) {
		if let Some((field, separator, rest)) = parse_list_field_name(s) {
			let rest = rest.trim_start();
			match rest.strip_prefix(':') {
				Some(rest) if defines.insert(field) => {
					let value = match separator {
						Separator::Space => rest.split_whitespace().collect(),
						Separator::Comma => rest.trim().split(',').map(|x| x.trim()).collect(),
					};
					pc.push(Pc::ListField(field, separator, value));
				}
				None => pc.push(Pc::Other(s)),
				Some(_) => bail!("the field {field} is defined multiple times"),
			}

			continue;
		}

		if let Some((field, rest)) = parse_string_field_name(s) {
			match rest.strip_prefix(':') {
				Some(val) if defines.insert(field) => pc.push(Pc::StringField(field, val.trim())),
				None => pc.push(Pc::Other(s)),
				_ => bail!("the field {field} is defined multiple times"),
			}

			continue;
		}

		if let Some((var, rest)) = parse_var_name(s) {
			match rest.strip_prefix('=') {
				Some(val) => pc.push(Pc::Var(var, val.trim_start())),
				None => pc.push(Pc::Other(s)),
			}

			continue;
		}

		pc.push(Pc::Other(s))
	}

	Ok(pc)
}

pub fn run(args: EditPc) -> Result<()> {
	if args.actions.is_empty() {
		info!("no action given, exiting early");
		return Ok(());
	}

	let contents = fs::read_to_string(&args.path)
		.map_err(|e| anyhow!("failure reading {}: {}", args.path.display(), e))?;

	let pc = parse_pc(&contents)
		.map_err(|e| anyhow!("failure parsing {}: {}", args.path.display(), e))?;

	let mut edited = edit_pc(pc, args.actions);

	if args.print {
		println!("{}", edited.trim());
	} else {
		if let Some(i) = edited.rfind(|c: char| !c.is_whitespace()) {
			edited.truncate(i + 1);
		}
		if edited.trim_start() == contents.trim() {
			info!("no changes");
		} else {
			edited.push('\n');

			fs::write(&args.path, edited.trim_start().as_bytes())
				.map_err(|e| anyhow!("failure saving changes: {e}"))?;
		}
	}

	Ok(())
}

fn edit_pc(pc: Vec<Pc>, actions: Vec<Action>) -> String {
	// This side-steps a limitation with the borrow checker.
	// Without it, it complains that `actions` does not live long enough.
	let mut pc = pc;
	let mut lookup = BTreeMap::new();

	for (i, x) in pc.iter().enumerate() {
		match x {
			Pc::StringField(name, ..) | Pc::ListField(name, ..) => {
				lookup.insert(*name, i);
			}
			_ => (),
		}
	}

	let mut new_vars = Vec::new();
	let mut new_vars_lookup = BTreeMap::new();

	for a in &actions {
		match a {
			Action::SetVar(name, value) => {
				let mut found = false;
				for x in &mut pc {
					match x {
						Pc::Var(x_name, old_val) if x_name == name => {
							found = true;
							*old_val = value.as_str();
						}
						_ => (),
					}
				}

				if !found {
					match new_vars_lookup.entry(name) {
						Entry::Vacant(x) => {
							x.insert(new_vars.len());
							new_vars.push((name, value));
						}
						Entry::Occupied(x) => {
							new_vars[*x.get()].1 = value;
						}
					}
				}
			}

			Action::StringField {
				name,
				value,
				if_missing,
			} => match lookup.get(name) {
				None => pc.push(Pc::StringField(name, value.as_str())),
				Some(_) if *if_missing => (),
				Some(&i) => pc[i] = Pc::StringField(name, value.as_str()),
			},

			Action::ListField {
				name,
				op,
				value,
				separator,
			} => {
				match lookup.get(name) {
					None => match op {
						ListOp::Remove => (),
						ListOp::Append
						| ListOp::AppendLeft
						| ListOp::Set
						| ListOp::SetIfMissing => {
							lookup.insert(name, pc.len());
							pc.push(Pc::ListField(
								name,
								*separator,
								value.iter().map(String::as_str).collect(),
							));
						}
					},
					Some(&i) => {
						let Pc::ListField(_, _, old_value) = &mut pc[i] else {
							unreachable!();
						};
						match op {
							ListOp::SetIfMissing => (),
							ListOp::Set => {
								old_value.clear();
								old_value.extend(value.iter().map(String::as_str));
							}
							ListOp::Append => {
								old_value.extend(value.iter().map(String::as_str));
							}
							ListOp::AppendLeft => {
								old_value.reserve(value.len());
								if old_value.is_empty() {
									old_value.extend(value.iter().map(String::as_str));
								} else if !value.is_empty() {
									// Append dummy values, then shift by replacing those dummy values and making up space to the left.
									let idx = old_value.len();
									old_value.extend((0..value.len()).map(|_| ""));
									for i in (0..idx).rev() {
										old_value[i + value.len()] = old_value[i];
									}
									for (i, x) in value.iter().enumerate() {
										old_value[i] = x.as_str();
									}
								}
							}
							ListOp::Remove => {
								old_value.retain(|&s| !value.iter().any(|v| s == v));
							}
						}
					}
				}
			}
		}
	}

	let mut buf = String::with_capacity(2048);
	for (name, value) in new_vars {
		let _ = writeln!(buf, "{name}={value}");
		warn!("the variable {name} is not present in the input; adding");
	}

	for x in &pc {
		let _ = match x {
			Pc::Other(s) => writeln!(buf, "{s}"),
			Pc::StringField(name, value) => writeln!(buf, "{name}: {value}"),
			Pc::Var(name, value) => writeln!(buf, "{name}={value}"),
			Pc::ListField(name, separator, value) => {
				let sep = match separator {
					Separator::Space => " ",
					Separator::Comma => ", ",
				};
				let _ = write!(buf, "{name}: ");
				let mut added = BTreeSet::new();
				for val in value {
					if added.insert(val) {
						if added.len() > 1 {
							buf += sep;
						}
						buf += val;
					}
				}
				buf.push('\n');
				Ok(())
			}
		};
	}

	buf
}
