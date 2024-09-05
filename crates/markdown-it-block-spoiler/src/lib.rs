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
    fn render(&self, node: &markdown_it::Node, fmt: &mut dyn markdown_it::Renderer) {
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
    const REGEX_STR: &str = r"^::: +spolier +\S+$";

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
    fn run(state: &mut BlockState) -> Option<(markdown_it::Node, usize)> {
        const SPOILER_GATE_END: &str = ":::";

        // Using trim_end since get_line already trims the start
        let first_line = state.get_line(state.line).trim_end();

        if !is_valid_block_start(first_line) {
            return None;
        }

        let spoiler_content_start_index = state.line + 1;
        let mut spoiler_depth: usize = 0;
        let spoiler_content_end_index = (spoiler_content_start_index..state.line_offsets.len())
            .map(|i| spoiler_content_start_index + i)
            .find(
                |&i| match (state.get_line(i).trim_end(), &mut spoiler_depth) {
                    (line, depth @ 1..) if line == SPOILER_GATE_END => {
                        *depth -= 1;
                        false
                    }
                    (line, depth) if is_valid_block_start(line) => {
                        *depth += 1;
                        false
                    }
                    (line, _) => line == SPOILER_GATE_END,
                },
            )?
            - 1;

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
            visible_text: first_line
                .split_whitespace()
                .skip(2)
                .intersperse(" ") // TODO: Use intersperse function from std once it makes it to a stable version: https://github.com/rust-lang/rust/issues/79524
                .collect(),
        });
        node.children
            .push(Node::new(InlineRoot::new(spoiler_content, mapping)));

        Some((
            node,
            spoiler_content_end_index - spoiler_content_start_index,
        ))
    }
}

pub fn add(md: &mut MarkdownIt) {
    md.block.add_rule::<BlockSpoilerScanner>();
}

// TODO: Write tests
