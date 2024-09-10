<div align="center">
  <img src="https://img.shields.io/crates/l/markdown-it-block-spoiler?style=for-the-badge" alt="License" />
  <img src="https://img.shields.io/crates/v/markdown-it-block-spoiler?style=for-the-badge" alt="Latest version" />
  <img src="https://img.shields.io/crates/dv/markdown-it-block-spoiler?style=for-the-badge" alt="Downloads for latest version" />
</div>
# markdown-it-spoiler.rs

A [`markdown-it`](https://crates.io/crates/markdown-it) plugin to process block spoliers.

To load the plugin:

```rust
let mut parser = markdown_it::MarkdownIt::new();
markdown_it::plugins::cmark::add(parser);

markdown_it_block_spoiler::add(&mut parser);

let html = parser.parse(r"::: spolier Click me to reveal my secrets
Arcane knowledge beyond human comprehension.
Can you handle this information without losing your mind?
:::").xrender();
```

If you are using this plugin in the browser, you can use the "browser" feature to have this library use built-in browser libraries to keep the bundle size down.

``` toml
markdown-it-spoiler = { version = "1", default-features = false, features = ["browser"] }
```
