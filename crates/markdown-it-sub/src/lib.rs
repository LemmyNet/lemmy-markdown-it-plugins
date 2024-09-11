//! A [`markdown-it`](https://crates.io/crates/markdown-it) plugin to process subscript.
//!
//! To load the plugin:
//!
//! ```rust
//! # use markdown_it;
//! # use markdown_it_sub;
//! let mut parser = markdown_it::MarkdownIt::new();
//! markdown_it::plugins::cmark::add(&mut parser);
//!
//! markdown_it_sub::add(&mut parser);
//!
//! let html = parser.parse("log~2~(a)").xrender();
//! assert_eq!(html, "<p>log<sub>2</sub>(a)</p>\n");
//! ```

use markdown_it::{generics::inline::emph_pair, MarkdownIt, Node, NodeValue, Renderer};

#[derive(Debug)]
struct Sub;

impl NodeValue for Sub {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        fmt.open("sub", &node.attrs);
        fmt.contents(&node.children);
        fmt.close("sub");
    }
}

/// Adds the subscript plugin to the parser.
pub fn add(md: &mut MarkdownIt) {
    emph_pair::add_with::<'~', 1, true>(md, || Node::new(Sub));
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
    #[case("~foo~", "<p><sub>foo</sub></p>\n")]
    #[case("foo~", "<p>foo~</p>\n")]
    #[case("~foo", "<p>~foo</p>\n")]
    #[case("foo~bar~", "<p>foo<sub>bar</sub></p>\n")]
    #[case("~~foo~bar~~~", "<p><s>foo<sub>bar</sub></s></p>\n")]
    #[case("\\~foo~", "<p>~foo~</p>\n")]
    #[case("~foo\\~", "<p>~foo~</p>\n")]
    fn test(#[case] md_str: &str, #[case] expected: &str) {
        let result = MARKDOWN_PARSER.parse(md_str).xrender();

        assert_eq!(result, String::from(expected));
    }
}
