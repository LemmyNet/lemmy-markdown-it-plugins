use markdown_it::{generics::inline::emph_pair, MarkdownIt, Node, NodeValue, Renderer};

#[derive(Debug)]
pub struct Sup;

impl NodeValue for Sup {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        fmt.open("sub", &node.attrs);
        fmt.contents(&node.children);
        fmt.close("sub");
    }
}

pub fn add(md: &mut MarkdownIt) {
    emph_pair::add_with::<'~', 1, true>(md, || Node::new(Sup));
}
