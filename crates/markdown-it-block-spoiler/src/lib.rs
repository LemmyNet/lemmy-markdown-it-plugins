use cfg_if::cfg_if;
use itertools::Itertools;
use markdown_it::{
    parser::{
        block::{BlockRule, BlockState},
        inline::InlineRoot,
    },
    MarkdownIt, Node, NodeValue,
};
use std::sync::LazyLock;

#[derive(Debug)]
pub struct BlockSpoiler {
    visible_text: String,
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

fn is_valid_block_start(line: &str) -> bool {
    const REGEX_STR: &str = r"^::: +spoiler +(?:\S+ *)+$";

    cfg_if! {
        if #[cfg(feature = "browser")] {
            use js_sys::RegExp;
            static SPOILER_START_REGEX: LazyLock<RegExp> = LazyLock::new(|| RegExp::new(REGEX_STR, "u"));

            SPOILER_START_REGEX.test(line)
        } else {
            use regex::Regex;
            static SPOLIER_START_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(REGEX_STR).expect("Invalid regex str!"));

            SPOLIER_START_REGEX.is_match(line)
        }
    }
}

impl BlockRule for BlockSpoilerScanner {
    fn run(state: &mut BlockState) -> Option<(Node, usize)> {
        // Using trim_end since get_line already trims the start
        let first_line = state.get_line(state.line).trim_end();

        if !is_valid_block_start(first_line) {
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
            // Using split_whitespace and skip here because number of spaces from ":::" to "spoiler" and "spoiler" to visible text is arbitrary,
            // and current implementation in lemmy-ui strips out extra whitespace between words in visible text.
            // Intersperse guarantees there are still spaces between visible text words.
            #[allow(unstable_name_collisions)]
            visible_text: first_line
                .split_whitespace()
                .skip(2)
                .intersperse(" ") // TODO: Use intersperse function from std once it makes it to a stable version: https://github.com/rust-lang/rust/issues/79524
                .collect(),
        });
        node.children
            .push(Node::new(InlineRoot::new(spoiler_content, mapping)));

        Some((node, (spoiler_content_end_index - state.line) + 1))
    }
}

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
