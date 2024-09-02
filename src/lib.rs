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

#[cfg(test)]
mod test {
    use super::add;
    use markdown_it::{
        plugins::{cmark, extra},
        MarkdownIt,
    };
    use std::sync::LazyLock;

    static MARKDOWN_PARSER: LazyLock<MarkdownIt> = LazyLock::new(|| {
        let mut parser = MarkdownIt::new();
        cmark::add(&mut parser);
        extra::add(&mut parser);
        add(&mut parser);

        parser
    });

    #[test]
    fn foo() {
        let result = MARKDOWN_PARSER.parse("~foo~").xrender();

        assert_eq!(result, String::from("<p><sub>foo</sub></p>\n"));
    }
}
