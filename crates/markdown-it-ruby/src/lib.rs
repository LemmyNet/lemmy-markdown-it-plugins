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

use std::char;

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
        let mut char_indices = state.src[state.pos..state.pos_max].char_indices();
        if char_indices.next()?.1 != Self::MARKER {
            return None;
        }

        let mut prev_char_escaped = false;

        let base_end_pos = char_indices.find_map(|(i, c)| {
            if c == '\\' {
                prev_char_escaped = true;
                return None;
            }

            let index = (c == '|' && !prev_char_escaped).then_some(i);
            prev_char_escaped = false;

            index
        })? + state.pos;

        let base_text = &state.src[state.pos + 1..base_end_pos];

        let ruby_end_pos = char_indices.skip(2).find_map(|(i, c)| {
            if c == '\\' {
                prev_char_escaped = true;
                return None;
            }

            let index = (c == '}' && !prev_char_escaped).then_some(i);
            prev_char_escaped = false;

            index
        })? + state.pos;

        let ruby_text = &state.src[base_end_pos + 1..ruby_end_pos];

        Some((
            Node::new(Ruby {
                base_text: base_text.trim().into(),
                ruby_text: ruby_text.trim().into(),
            }),
            (ruby_end_pos - state.pos) + 1,
        ))
    }
}

pub fn add(md: &mut MarkdownIt) {
    md.inline.add_rule::<RubyScanner>();
}

#[cfg(test)]
mod test {
    use super::add;
    use markdown_it::{
        plugins::{cmark, extra},
        MarkdownIt,
    };
    use rstest::rstest;
    use std::sync::LazyLock;

    static MARKDOWN_PARSER: LazyLock<MarkdownIt> = LazyLock::new(|| {
        let mut parser = MarkdownIt::new();
        cmark::add(&mut parser);
        extra::add(&mut parser);
        add(&mut parser);

        parser
    });

    #[rstest]
    #[case("{漢|Kan}{字|ji}", "<p><ruby>漢<rp>(</rp><rt>Kan</rt><rp>)</rp></ruby><ruby>字<rp>(</rp><rt>ji</rt><rp>)</rp></ruby></p>\n")]
    #[case(
        "\\{foo|bar}{baz|qux}",
        "<p>{foo|bar}<ruby>baz<rp>(</rp><rt>qux</rt><rp>)</rp></ruby></p>\n"
    )]
    #[case(
        "{foo|bar}{baz\\|qux}",
        "<p><ruby>foo<rp>(</rp><rt>bar</rt><rp>)</rp></ruby>{baz|qux}</p>\n"
    )]
    fn test(#[case] md_str: &str, #[case] expected: &str) {
        let result = MARKDOWN_PARSER.parse(md_str).xrender();

        assert_eq!(result, String::from(expected));
    }
}
