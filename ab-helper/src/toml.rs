use std::{
	fmt,
	fs,
	io::{
		self,
		Read,
	},
	mem,
	path::{
		Path,
		PathBuf,
	},
	str::FromStr,
};

use anyhow::{
	anyhow,
	bail,
	Result,
};
use clap::Parser as Clap;
use toml_edit::{
	DocumentMut,
	Entry,
	Item,
	Key,
	Table,
	Value,
};

#[derive(Clap)]
/// View or edit TOML values
pub struct Toml {
	#[command(subcommand)]
	cmd: Cmd,
}

#[derive(Clap)]
enum Cmd {
	/// Get a value
	Get(Get),
	/// Set a value
	Set(Set),
}

#[derive(Clap)]
pub struct Get {
	/// Path to a .toml file
	#[arg(short, long)]
	path: PathBuf,

	/// Print in TOML syntax
	#[arg(short, long)]
	toml: bool,

	/// The TOML key
	#[arg(value_parser = TomlPath::parse)]
	key: TomlPath,
}

#[derive(Clap)]
pub struct Set {
	/// Path to a .toml file
	#[arg(short, long)]
	path: PathBuf,

	/// Write output to a file (by default, it's the same as the path specified with --path)
	#[arg(short, long)]
	out: Option<PathBuf>,

	/// Append to arrays instead of replacing them
	#[arg(short, long)]
	append_arrays: bool,

	/// Keys to set in full TOML form; e.g. `target.'cfg(windows)'.rustflags = ['-C', 'target-feature=+crt-static']`
	#[arg(value_parser = DocumentMut::from_str)]
	changes: Vec<DocumentMut>,
}

#[derive(Clone)]
struct TomlPath {
	path: Vec<Key>,
}

impl TomlPath {
	fn parse(s: &str) -> Result<Self, toml_edit::TomlError> {
		Key::parse(s).map(|path| Self { path })
	}

	fn get_from<'a>(&self, doc: &'a DocumentMut) -> Option<&'a Item> {
		let mut current = doc.as_item();
		for k in &self.path {
			current = current.get(k.get())?;
		}

		Some(current)
	}
}

impl fmt::Display for TomlPath {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		for (i, k) in self.path.iter().enumerate() {
			if i > 0 {
				write!(f, ".{k}")?;
			} else {
				k.fmt(f)?;
			}
		}

		Ok(())
	}
}

pub fn run(cmd: Toml) -> Result<()> {
	match cmd.cmd {
		Cmd::Get(x) => run_get(x),
		Cmd::Set(x) => run_set(x),
	}
}

fn read_toml(p: &Path) -> Result<DocumentMut> {
	let data = if p.as_os_str() == "-" {
		let mut buf = String::with_capacity(1024);
		io::stdin().lock().read_to_string(&mut buf)?;
		buf
	} else {
		fs::read_to_string(p)?
	};

	data.parse::<DocumentMut>()
		.map_err(|e| anyhow!("input is not valid TOML: {e}"))
}

fn format_item(x: Item) -> Item {
	match x {
		Item::Value(mut v) => {
			match &mut v {
				Value::Array(a) => a.fmt(),
				Value::InlineTable(t) => t.fmt(),
				_ => (),
			}
			Item::Value(v.decorated(" ", ""))
		}
		Item::Table(mut t) => {
			t.fmt();
			Item::Table(t)
		}
		_ => x,
	}
}

fn merge(main: &mut DocumentMut, mut to_merge: DocumentMut, append_arrays: bool) {
	fn merge_tables<I>(main: &mut Table, table_items: I, append_arrays: bool)
	where
		I: IntoIterator<Item = (toml_edit::InternalString, Item)>,
	{
		for (k, v) in table_items {
			let v = format_item(v);
			let k = Key::new(k); //.with_leaf_decor(Decor::new("", " "));
						// If `v` isn't an array or table (inline or normal), insert it and continue.
			let is_array = v.is_array();
			if !is_array && !matches!(v, Item::Table(..) | Item::Value(Value::InlineTable(..))) {
				main.insert_formatted(&k, v);
				continue;
			}

			let (mut table, is_inline) = match main.entry(&k) {
				Entry::Vacant(x) => {
					x.insert(v);
					continue;
				}
				Entry::Occupied(mut x) => match x.get_mut() {
					Item::Value(Value::Array(main_arr)) if is_array => {
						let Item::Value(Value::Array(a)) = v else {
							unreachable!()
						};
						if !append_arrays {
							main_arr.clear();
						}
						main_arr.extend(a);
						main_arr.fmt();
						continue;
					}
					Item::Table(t) if !is_array => (mem::take(t), false),
					Item::Value(Value::InlineTable(t)) if !is_array => {
						(mem::take(t).into_table(), true)
					}
					_ => {
						x.insert(v);
						continue;
					}
				},
			};

			match v {
				Item::Table(t) => {
					merge_tables(&mut table, t, append_arrays);
				}
				Item::Value(Value::InlineTable(t)) => {
					merge_tables(&mut table, t.into_table(), append_arrays);
				}
				_ => unreachable!(),
			}

			table.fmt();
			if is_inline {
				main.insert_formatted(
					&k,
					Item::Value(Value::InlineTable(table.into_inline_table())),
				);
			} else {
				main.insert_formatted(&k, Item::Table(table));
			}
		}
	}

	merge_tables(
		main.as_table_mut(),
		mem::take(to_merge.as_table_mut()),
		append_arrays,
	);
}

fn run_get(args: Get) -> Result<()> {
	let doc = read_toml(&args.path)?;
	let val = match args
		.key
		.get_from(&doc)
		.ok_or_else(|| anyhow!("the key {} does not exist in the TOML document", args.key))?
	{
		Item::Value(val) => val,
		Item::None => return Ok(()),
		Item::Table(_) | Item::ArrayOfTables(_) => {
			bail!("this command does not support printing tables")
		}
	};

	if args.toml {
		println!("{val}");
		return Ok(());
	}

	match val {
		Value::String(x) => println!("{}", x.value()),
		Value::Integer(x) => println!("{}", x.value()),
		Value::Float(x) => println!("{}", x.value()),
		Value::Boolean(x) => println!("{}", x.value()),
		Value::Datetime(x) => println!("{x}"),
		Value::Array(_) => bail!("printing arrays is not supported without the --toml option"),
		Value::InlineTable(_) => {
			bail!("printing inline tables is not supported without the --toml option")
		}
	}

	Ok(())
}

fn run_set(args: Set) -> Result<()> {
	let mut doc = read_toml(&args.path)?;
	for change in args.changes {
		merge(&mut doc, change, args.append_arrays);
	}

	let out = args.out.as_deref().unwrap_or(&*args.path);
	if out.as_os_str() == "-" {
		println!("{doc}");
	} else {
		fs::write(out, doc.to_string().as_bytes())?;
	}

	Ok(())
}
