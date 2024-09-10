<div align="center">
  <img src="https://img.shields.io/crates/l/markdown-it-ruby?style=for-the-badge" alt="License" />
  <img src="https://img.shields.io/crates/v/markdown-it-ruby?style=for-the-badge" alt="Latest version" />
  <img src="https://img.shields.io/crates/dv/markdown-it-ruby?style=for-the-badge" alt="Downloads for latest version" />
</div>

# markdown-it-ruby.rs

A [`markdown-it`](https://crates.io/crates/markdown-it) plugin to process [ruby text](https://en.wikipedia.org/wiki/Ruby_character).

To load the plugin:

```rust
let mut parser = markdown_it::MarkdownIt::new();
markdown_it::plugins::cmark::add(&mut parser);

markdown_it_ruby::add(&mut parser);

let html = parser.parse("{漢|Kan}{字|ji}").xrender();
assert_eq!(html, String::from("<p><ruby>漢<rp>(</rp><rt>Kan</rt><rp>)</rp></ruby><ruby>字<rp>(</rp><rt>ji</rt><rp>)</rp></ruby></p>\n"));
```
