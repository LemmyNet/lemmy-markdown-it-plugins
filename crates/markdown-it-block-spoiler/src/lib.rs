//! A [`markdown-it`](https://crates.io/crates/markdown-it) plugin to process block spoilers.
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

use std::mem;

use itertools::Itertools;
use markdown_it::{
    parser::block::{BlockRule, BlockState},
    MarkdownIt, Node, NodeValue, Renderer,
};

const MARKER_CHAR: char = ':';

#[derive(Debug)]
pub struct BlockSpoiler {
    pub visible_text: String,
}

impl NodeValue for BlockSpoiler {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        fmt.cr();
        fmt.open("details", &node.attrs);
        fmt.cr();

        fmt.open("summary", &[]);
        fmt.cr();
        fmt.text(&self.visible_text);
        fmt.cr();
        fmt.close("summary");
        fmt.cr();
        fmt.contents(&node.children);
        fmt.cr();
        fmt.close("details");
        fmt.cr();
    }
}

struct BlockSpoilerScanner;

impl BlockSpoilerScanner {
    fn get_header(state: &mut BlockState) -> Option<(usize, String)> {
        if state.line_indent(state.line) >= state.md.max_indent {
            return None;
        }

        // Using split_whitespace and skip here because number of spaces from
        // the marker to "spoiler" and "spoiler" to visible text is arbitrary,
        // and the current implementation used by lemmy-ui
        // strips out extra whitespace between words in visible text.
        let mut first_line_words = state.get_line(state.line).split_whitespace().peekable();

        let marker_len = first_line_words
            .next_if(|word| word.chars().all(|c| c == MARKER_CHAR))?
            .len();

        if !(marker_len >= 3
            // These iterator method calls need to be in the order they're in
            // so that "spoiler" gets consumed
            && first_line_words.next()? == "spoiler"
            && first_line_words.peek().is_some())
        {
            return None;
        }

        // Intersperse guarantees there are still spaces between visible text words.
        #[expect(unstable_name_collisions)]
        let visible_text = first_line_words
            .intersperse(" ") // TODO: Use intersperse function from std once it makes it to a stable version: https://github.com/rust-lang/rust/issues/79524
            .collect();

        Some((marker_len, visible_text))
    }
}

impl BlockRule for BlockSpoilerScanner {
    fn check(state: &mut BlockState) -> Option<()> {
        Self::get_header(state).map(|_| ())
    }

    fn run(state: &mut BlockState) -> Option<(Node, usize)> {
        let (_, visible_text) = Self::get_header(state)?;

        let spoiler_content_start_line = state.line + 1;

        println!("-------------------");
        println!("Tegst: {visible_text}");
        // TODO: Handle case where spoiler block is closed by parent spoiler block instead of marker
        let mut spoiler_content_end_line = (spoiler_content_start_line..state.line_max)
            .find(|&i| {
                let line = state.get_line(i).trim_end();
                println!("{i} -- {line}");
                line == ":::"
            })
            .or_else(|| state.node.is::<BlockSpoiler>().then_some(state.line_max))?;
        println!(
            "End Line: {spoiler_content_end_line}, Start Line: {}, Line Max: {}, Content Start: {}",
            state.line, state.line_max, spoiler_content_start_line
        );

        let old_indent = state.blk_indent;
        state.blk_indent = 0;

        // TODO: Explain what's going on here with comments.
        let old_node = mem::replace(&mut state.node, Node::new(BlockSpoiler { visible_text }));
        let old_line_max = state.line_max;
        state.line = spoiler_content_start_line;
        state.line_max = spoiler_content_end_line;
        state.md.block.tokenize(state);
        spoiler_content_end_line = state.line;
        state.line = spoiler_content_start_line;
        state.line_max = old_line_max;

        state.blk_indent = old_indent;

        let node = std::mem::replace(&mut state.node, old_node);
        Some((node, (spoiler_content_end_line - state.line) + 1))
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
        "<details>\n<summary>\nclick to see more\n</summary>\n<p>how spicy!</p>\n</details>\n"
    )]
    #[case(
        "::: spoiler click to see more\nhow spicy!\n:::\n",
        "<details>\n<summary>\nclick to see more\n</summary>\n<p>how spicy!</p>\n</details>\n"
    )]
    #[case(
        "::: spoiler _click to see more_\nhow spicy!\n:::\n",
        "<details>\n<summary>\n_click to see more_\n</summary>\n<p>how spicy!</p>\n</details>\n"
    )]
    #[case("::: spoiler click to see more\n**how spicy!**\n*i have many lines*\n:::\n",
        "<details>\n<summary>\nclick to see more\n</summary>\n<p><strong>how spicy!</strong>\n<em>i have many lines</em></p>\n</details>\n")]
    #[case("hey you\npsst, wanna hear a secret?\n::: spoiler lean in and i'll tell you\n**you are breathtaking!**\n:::\nwhatcha think about that?",
        "<p>hey you\npsst, wanna hear a secret?</p>\n<details>\n<summary>\nlean in and i'll tell you\n</summary>\n<p><strong>you are breathtaking!</strong></p>\n</details>\n<p>whatcha think about that?</p>\n")]
    #[case("- did you know that\n::: spoiler the call was\n***coming from inside the house!***\n:::\n - crazy, right?",
        "<ul>\n<li>did you know that</li>\n</ul>\n<details>\n<summary>\nthe call was\n</summary>\n<p><em><strong>coming from inside the house!</strong></em></p>\n</details>\n<ul>\n<li>crazy, right?</li>\n</ul>\n")]
    #[case("\n::: spoiler 1\n\n\n::: spoiler 2\n::: spoiler 3\n::: spoiler 4\n::: spoiler 5\n::: spoiler 6\n::: spoiler 7\n::: spoiler 8\n\n:::\n\n\nThis could probably be used to make a choose your own adventure game, provided your client can handle it.\n\n",
    "<details><summary>1</summary>\n<details><summary>2</summary>\n<details><summary>3</summary>\n<details><summary>4</summary>\n<details><summary>5</summary>\n<details><summary>6</summary>\n<details><summary>7</summary>\n<details><summary>8</summary>\n</details>\n</details>\n</details>\n</details>\n</details>\n</details>\n</details>\n</details>\n<p>This could probably be used to make a choose your own adventure game, provided your client can handle it.</p>\n")]
    fn test(#[case] md_str: &str, #[case] expected: &str) {
        let result = MARKDOWN_PARSER.parse(md_str).xrender();

        assert_eq!(result, String::from(expected));
    }
}
