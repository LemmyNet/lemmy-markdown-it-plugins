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

use itertools::Itertools;
use markdown_it::{
    parser::inline::{InlineRule, InlineState},
    MarkdownIt, Node, NodeValue, Renderer,
};

#[derive(Debug)]
pub struct Ruby {
    pub base_text: String,
    pub ruby_text: String,
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

fn find_text_end_index<const BREAK_CHAR: char>(
    char_indices: &mut impl Iterator<Item = (usize, char)>,
) -> Option<usize> {
    let mut prev_char_escaped = false;

    char_indices.find_map(|(i, c)| {
        if c == '\\' {
            prev_char_escaped = true;
            return None;
        }

        let index = (c == BREAK_CHAR && !prev_char_escaped).then_some(i);
        prev_char_escaped = false;

        index
    })
}

fn prepare_text(text: &str) -> String {
    // Intersperse guarantees there are still spaces between visible text words.
    #[allow(unstable_name_collisions)]
    text.split_whitespace()
        .intersperse(" ") // TODO: Use intersperse function from std once it makes it to a stable version: https://github.com/rust-lang/rust/issues/79524
        .collect::<String>()
        .replace('\\', "")
}

struct RubyScanner;

impl InlineRule for RubyScanner {
    const MARKER: char = '{';

    fn run(state: &mut InlineState) -> Option<(Node, usize)> {
        let mut char_indices = state.src[state.pos..state.pos_max].char_indices();
        if char_indices.next()?.1 != Self::MARKER {
            return None;
        }

        let base_end_pos = find_text_end_index::<'|'>(&mut char_indices)? + state.pos;
        let base_text = &state.src[state.pos + 1..base_end_pos];

        let end_pos = find_text_end_index::<'}'>(&mut char_indices.skip(2))? + state.pos;
        let ruby_text = &state.src[base_end_pos + 1..end_pos];

        Some((
            Node::new(Ruby {
                base_text: prepare_text(base_text),
                ruby_text: prepare_text(ruby_text),
            }),
            (end_pos - state.pos) + 1,
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
    #[case(
        "{foo\\|bar\\}{baz|qux}",
        "<p><ruby>foo|bar}{baz<rp>(</rp><rt>qux</rt><rp>)</rp></ruby></p>\n"
    )]
    #[case(
        "{foo\\|bar}\\{baz|qux}",
        "<p><ruby>foo|bar}{baz<rp>(</rp><rt>qux</rt><rp>)</rp></ruby></p>\n"
    )]
    #[case(
        "Some stuff before      {foo      doo|bar            hello}  mid {baz|qux} after      words",
        "<p>Some stuff before      <ruby>foo doo<rp>(</rp><rt>bar hello</rt><rp>)</rp></ruby>  mid <ruby>baz<rp>(</rp><rt>qux</rt><rp>)</rp></ruby> after      words</p>\n"
    )]
    fn test(#[case] md_str: &str, #[case] expected: &str) {
        let result = MARKDOWN_PARSER.parse(md_str).xrender();

        assert_eq!(result, String::from(expected));
    }
}
