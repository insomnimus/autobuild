#[derive(Copy, Clone, Debug)]
enum Token {
	Lit(usize, usize),
	Var(usize, usize),
}

#[derive(Debug)]
pub struct Template {
	template: String,
	tokens: Vec<Token>,
}

impl Template {
	pub fn new<S>(template: S, open: &str, close: &str) -> Self
	where
		S: Into<String> + AsRef<str>,
	{
		assert!(!open.is_empty(), "the opening symbol can't be emty");
		assert!(!close.is_empty(), "the closing symbol can't be emty");

		let s = template.as_ref();
		let mut tokens = Vec::with_capacity(8);
		let mut i = 0;

		while i < s.len() {
			let remaining = &s[i..];
			match remaining.find(open) {
				None => {
					tokens.push(Token::Lit(i, s.len()));
					break;
				}
				Some(0) => {
					// let end = remaining.find('}').unwrap_or(remaining.len()) + i;
					let end = match remaining[open.len()..]
						.find(close)
						.map(|n| n + open.len() + close.len() - 1)
					{
						Some(n) => n + i,
						None => {
							tokens.push(Token::Lit(i, s.len()));
							break;
						}
					};

					tokens.push(Token::Var(i + open.len(), end));
					i = end + 1;
				}
				Some(n) => {
					tokens.push(Token::Lit(i, i + n));
					i += n;
				}
			}
		}

		Self {
			template: template.into(),
			tokens,
		}
	}

	pub fn expand<'a, S, F>(&'a self, mut f: F) -> String
	where
		F: FnMut(&'a str) -> S,
		S: AsRef<str>,
	{
		self.tokens.iter().copied().fold(
			String::with_capacity(self.template.len() * 2),
			|mut buf, t| {
				match t {
					Token::Lit(start, end) => buf += &self.template[start..end],
					Token::Var(start, end) => buf += f(&self.template[start..end]).as_ref(),
				}
				buf
			},
		)
	}
}
