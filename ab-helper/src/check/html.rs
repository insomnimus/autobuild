use kuchikiki::{
	iter::{
		Descendants,
		Elements,
		Select,
	},
	traits::TendrilSink,
	ElementData,
	NodeData,
	NodeDataRef,
	NodeRef,
};

pub fn parse_html(s: &str) -> NodeRef {
	kuchikiki::parse_html().one(s).document_node
}

type SelectIter = Select<Elements<Descendants>>;

pub trait NodeExt {
	fn query(&self, selector: &str) -> SelectIter;
	fn inner_text(&self) -> String;
	fn attr_filter(&self, name: &str) -> Option<String>;
	fn is(&self, tag: &str) -> bool;
	fn next_element(&self) -> Option<NodeRef>;
	fn as_element(&self) -> Option<&ElementData>;

	fn is_element(&self) -> bool {
		self.as_element().is_some()
	}

	fn is_class(&self, class: &str) -> bool {
		self.attr_filter("class").map_or(false, |s| {
			s.split_ascii_whitespace()
				.any(|s| s.eq_ignore_ascii_case(class))
		})
	}

	fn href(&self) -> Option<String> {
		self.attr_filter("href")
	}
}

impl<T> NodeExt for NodeDataRef<T> {
	fn query(&self, selector: &str) -> SelectIter {
		self.as_node().query(selector)
	}

	fn as_element(&self) -> Option<&ElementData> {
		self.as_node().as_element()
	}

	fn inner_text(&self) -> String {
		self.as_node().inner_text()
	}

	fn attr_filter(&self, name: &str) -> Option<String> {
		self.as_node().attr_filter(name)
	}

	fn is(&self, tag: &str) -> bool {
		self.as_node().is(tag)
	}

	fn next_element(&self) -> Option<NodeRef> {
		self.as_node().next_element()
	}
}

impl NodeExt for NodeRef {
	fn query(&self, selector: &str) -> SelectIter {
		self.select(selector).unwrap_or_else(|()| {
			eprintln!("internal css selector error: the selector {selector:?} is invalid");
			std::process::abort();
		})
	}

	fn as_element(&self) -> Option<&ElementData> {
		match self.data() {
			NodeData::Element(e) => Some(e),
			_ => None,
		}
	}

	fn inner_text(&self) -> String {
		self.text_contents()
	}

	fn attr_filter(&self, name: &str) -> Option<String> {
		self.as_element()
			.and_then(|e| e.attributes.borrow().get(name).map(str::to_owned))
	}

	fn is(&self, tag: &str) -> bool {
		self.as_element()
			.map_or(false, |e| e.name.local.eq_str_ignore_ascii_case(tag))
	}

	fn next_element(&self) -> Option<NodeRef> {
		self.following_siblings().find(|x| x.is_element())
	}
}
