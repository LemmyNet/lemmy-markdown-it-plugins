<div align="center">
  <img src="https://img.shields.io/crates/l/markdown-it-sub?style=for-the-badge" alt="License" />
  <img src="https://img.shields.io/crates/v/markdown-it-sub?style=for-the-badge" alt="Latest version" />
  <img src="https://img.shields.io/crates/dv/markdown-it-sub?style=for-the-badge" alt="Downloads for latest version" />
</div>
# markdown-it-sub.rs

A [`markdown-it`](https://crates.io/crates/markdown-it) plugin to process subscript.

To load the plugin:

```rust
let mut parser = markdown_it::MarkdownIt::new();
markdown_it::plugins::cmark::add(&mut parser);

markdown_it_sub::add(&mut parser);

let html = parser.parse("log~2~(a)").xrender();
assert_eq!(html, "<p>log<sub>2</sub>(a)</p>\n");
```
