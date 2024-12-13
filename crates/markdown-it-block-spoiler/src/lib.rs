//! A [`markdown-it`](https://crates.io/crates/markdown-it) plugin to process block spoliers.
//!
//! To load the plugin:
//!
//! ```rust
//! # use markdown_it;
//! # use markdown_it_block_spoiler;
//! let mut parser = markdown_it::MarkdownIt::new();
//! markdown_it::plugins::cmark::add(&mut parser);
//!
//! markdown_it_block_spoiler::add(&mut parser);
//!
//! let html = parser.parse("::: spoiler _click to see more_\nhow spicy!\n:::\n").xrender();
//! assert_eq!(html, String::from("<details><summary>_click to see more_</summary>how spicy!\n</details>\n"));
//! ```

use itertools::Itertools;
use markdown_it::{
    parser::{
        block::{BlockRule, BlockState},
        inline::InlineRoot,
    },
    MarkdownIt, Node, NodeValue,
};

#[derive(Debug)]
pub struct BlockSpoiler {
    pub visible_text: String,
}

impl NodeValue for BlockSpoiler {
    fn render(&self, node: &Node, fmt: &mut dyn markdown_it::Renderer) {
        fmt.cr();
        fmt.open("details", &node.attrs);
        fmt.open("summary", &[]);
        fmt.text(&self.visible_text);
        fmt.close("summary");
        fmt.contents(&node.children);
        fmt.close("details");
        fmt.cr();
    }
}

struct BlockSpoilerScanner;

impl BlockRule for BlockSpoilerScanner {
    fn run(state: &mut BlockState) -> Option<(Node, usize)> {
        // Using split_whitespace and skip here because number of spaces from ":::" to "spoiler" and "spoiler" to visible text is arbitrary,
        // and current implementation in lemmy-ui strips out extra whitespace between words in visible text.
        let mut first_line_words = state.get_line(state.line).split_whitespace().peekable();

        // The order these iterator methods are called in is essential
        if !(first_line_words.next()? == ":::"
            && first_line_words.next()? == "spoiler"
            && first_line_words.peek().is_some())
        {
            return None;
        }

        let spoiler_content_start_index = state.line + 1;
        let spoiler_content_end_index = (spoiler_content_start_index..state.line_max)
            .find(|&i| state.get_line(i).trim_end() == ":::")?;

        let (spoiler_content, mapping) = state.get_lines(
            spoiler_content_start_index,
            spoiler_content_end_index,
            state.blk_indent,
            true,
        );
        let mut node = Node::new(BlockSpoiler {
            // Intersperse guarantees there are still spaces between visible text words.
            #[allow(unstable_name_collisions)]
            visible_text: first_line_words
                .intersperse(" ") // TODO: Use intersperse function from std once it makes it to a stable version: https://github.com/rust-lang/rust/issues/79524
                .collect(),
        });
        node.children
            .push(Node::new(InlineRoot::new(spoiler_content, mapping)));

        Some((node, (spoiler_content_end_index - state.line) + 1))
    }
}

/// Adds the block spoiler plugin to the parser.
pub fn add(md: &mut MarkdownIt) {
    md.block.add_rule::<BlockSpoilerScanner>();
}

#[cfg(test)]
mod tests {
    use crate::add;
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
    #[case(
        "::: spoiler click to see more\nbut I never finished",
        "<p>::: spoiler click to see more\nbut I never finished</p>\n"
    )]
    #[case(
        "::: spoiler\nnever added the lead in\n:::",
        "<p>::: spoiler\nnever added the lead in\n:::</p>\n"
    )]
    #[case(
        "::: spoiler click to see more\nhow spicy!\n:::",
        "<details><summary>click to see more</summary>how spicy!\n</details>\n"
    )]
    #[case(
        "::: spoiler click to see more\nhow spicy!\n:::\n",
        "<details><summary>click to see more</summary>how spicy!\n</details>\n"
    )]
    #[case(
        "::: spoiler _click to see more_\nhow spicy!\n:::\n",
        "<details><summary>_click to see more_</summary>how spicy!\n</details>\n"
    )]
    #[case("::: spoiler click to see more\n**how spicy!**\n*i have many lines*\n:::\n",
        "<details><summary>click to see more</summary><strong>how spicy!</strong>\n<em>i have many lines</em>\n</details>\n")]
    #[case("hey you\npsst, wanna hear a secret?\n::: spoiler lean in and i'll tell you\n**you are breathtaking!**\n:::\nwhatcha think about that?",
        "<p>hey you\npsst, wanna hear a secret?</p>\n<details><summary>lean in and i'll tell you</summary><strong>you are breathtaking!</strong>\n</details>\n<p>whatcha think about that?</p>\n")]
    #[case("- did you know that\n::: spoiler the call was\n***coming from inside the house!***\n:::\n - crazy, right?",
        "<ul>\n<li>did you know that</li>\n</ul>\n<details><summary>the call was</summary><em><strong>coming from inside the house!</strong></em>\n</details>\n<ul>\n<li>crazy, right?</li>\n</ul>\n")]
    fn test(#[case] md_str: &str, #[case] expected: &str) {
        let result = MARKDOWN_PARSER.parse(md_str).xrender();

        assert_eq!(result, String::from(expected));
    }
}
