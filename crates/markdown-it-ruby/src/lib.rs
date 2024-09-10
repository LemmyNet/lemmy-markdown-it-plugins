//! A [`markdown-it`](https://crates.io/crates/markdown-it) plugin to process [ruby text](https://en.wikipedia.org/wiki/Ruby_character).
//!
//! To load the plugin:
//!
//! ```rust
//! # use markdown_it;
//! # use markdown_it_ruby;
//! let mut parser = markdown_it::MarkdownIt::new();
//! markdown_it::plugins::cmark::add(&mut parser);
//!
//! markdown_it_ruby::add(&mut parser);
//!
//! let html = parser.parse("{漢|Kan}{字|ji}").xrender();
//! assert_eq!(html, String::from("<p><ruby>漢<rp>(</rp><rt>Kan</rt><rp>)</rp></ruby><ruby>字<rp>(</rp><rt>ji</rt><rp>)</rp></ruby></p>\n"));
//! ```

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

impl InlineRule for RubyScanner {
    const MARKER: char = '{';

    fn run(state: &mut InlineState) -> Option<(Node, usize)> {
        let end_pos = state.src[state.pos..state.pos_max]
            .char_indices()
            .find_map(|(i, c)| (c == '}').then_some(i))?
            + state.pos;
        let (base_text, ruby_text) = state.src[state.pos + 1..end_pos].split_once('|')?;

        Some((
            Node::new(Ruby {
                base_text: base_text.trim().into(),
                ruby_text: ruby_text.trim().into(),
            }),
            (end_pos - state.pos) + 1,
        ))
    }
}

pub fn add(md: &mut MarkdownIt) {
    md.inline.add_rule::<RubyScanner>();
}
