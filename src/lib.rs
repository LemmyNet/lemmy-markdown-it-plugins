use std::sync::LazyLock;

use markdown_it::{Node, NodeValue, Renderer};
use regex::Regex;

static UNESCAPE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r##"\\([ \\!"#$%&'()*+,./:;<=>?@[\]^_`{|}~-])"##).expect("Invalid regex!")
});

#[derive(Debug)]
pub struct Sup;

impl NodeValue for Sup {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        fmt.open("sub", &node.attrs);
        fmt.contents(&node.children);
        fmt.close("sub");
    }
}
