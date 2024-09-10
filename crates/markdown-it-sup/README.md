<div align="center">
  <img src="https://img.shields.io/crates/l/markdown-it-sup?style=for-the-badge" alt="License" />
  <img src="https://img.shields.io/crates/v/markdown-it-sup?style=for-the-badge" alt="Latest version" />
  <img src="https://img.shields.io/crates/dv/markdown-it-sup?style=for-the-badge" alt="Downloads for latest version" />
</div>
# markdown-it-sup.rs

A [`markdown-it`](https://crates.io/crates/markdown-it) plugin to process superscript.

To load the plugin:

```rust
let mut parser = markdown_it::MarkdownIt::new();
markdown_it::plugins::cmark::add(&mut parser);

markdown_it_sup::add(&mut parser);

let html = parser.parse("Markdown^TM^").xrender();
assert_eq!(html, String::from("<p>Markdown<sup>TM</sup></p>\n"));
```
