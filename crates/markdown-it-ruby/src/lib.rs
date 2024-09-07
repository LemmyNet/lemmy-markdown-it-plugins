use cfg_if::cfg_if;
use std::sync::LazyLock;

use markdown_it::{
    parser::inline::{InlineRule, InlineState},
    MarkdownIt, Node, NodeValue, Renderer,
};

#[derive(Debug)]
pub struct Ruby {
    base_text: String,
    ruby_text: String,
}

impl NodeValue for Ruby {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        fmt.open("ruby", &node.attrs);
        fmt.text(self.base_text.trim());

        fmt.open("rp", &[]);
        fmt.text("(");
        fmt.close("rp");

        fmt.open("rt", &[]);
        fmt.text(self.ruby_text.trim());
        fmt.close("rt");

        fmt.open("rp", &[]);
        fmt.text(")");
        fmt.close("rp");

        fmt.close("ruby");
    }
}

struct RubyScanner;

fn get_texts(search_str: &str) -> Option<(String, String)> {
    const REGEX_PATTERN: &str = r"\{( *[\S]+ *)+\|( *[\S]+ *)+\}";

    cfg_if! {
        if #[cfg(feature = "browser")] {
            use js_sys::RegExp;
            static RUBY_REGEX = LazyLock<RegExp> = LazyLock::new(|| RegExp::new(REGEX_PATTERN, "u"));

            let capture_groups = RUBY_REGEX.exec(search_str)?;
            Some((capture_groups.get(1).as_string()?, capture_groups.get(2).as_string()?))
        } else {
            use regex::Regex;
            static RUBY_REGEX: LazyLock<Regex> =
                LazyLock::new(|| Regex::new(REGEX_PATTERN).expect("Invalid ruby regex"));

            let capture_groups = RUBY_REGEX.captures(search_str)?;
            Some((capture_groups.get(0)?.as_str().into(), capture_groups.get(1)?.as_str().into()))
        }
    }
}

impl InlineRule for RubyScanner {
    const MARKER: char = '{';

    fn run(state: &mut InlineState) -> Option<(Node, usize)> {
        let input = &state.src[state.pos..state.pos_max];
        let (base_text, ruby_text) = get_texts(input)?;

        Some((
            Node::new(Ruby {
                base_text,
                ruby_text,
            }),
            state.pos_max - state.pos,
        ))
    }
}

pub fn add(md: &mut MarkdownIt) {
    md.inline.add_rule::<RubyScanner>();
}
