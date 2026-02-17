# aspect-rs Documentation

This directory contains the comprehensive mdBook documentation for aspect-rs.

## Building the Book

Install mdbook (if needed):

```bash
cargo install mdbook
```

Build the book:

```bash
cd docs/book
mdbook build
```

The output will be in `docs/book/book/`.

## Serving Locally

Serve the book with live-reload:

```bash
mdbook serve
```

Then visit http://localhost:3000

## Viewing Online

The book is published at: [https://yijunyu.github.io/aspect-rs/](https://yijunyu.github.io/aspect-rs/) (or your GitHub Pages URL)

## Structure

- `src/` - Markdown source files
- `book.toml` - Configuration
- `SUMMARY.md` - Table of contents
- `theme/` - Custom styling (optional)

## Contributing

To update the documentation:

1. Edit markdown files in `src/`
2. Run `mdbook build` to verify
3. Commit and push

The book will automatically rebuild on GitHub Pages.

## Content Overview

The book covers:

- **Chapters 1-3**: Getting Started (Motivation, Background, Installation)
- **Chapters 4-5**: User Guide (Core Concepts, Usage Patterns)
- **Chapters 6-7**: Technical Reference (Architecture, Implementation)
- **Chapters 8-9**: Real-World (Case Studies, Benchmarks)
- **Chapter 10**: Advanced Topics (Phase 3 Automatic Weaving)
- **Chapter 11**: Community (Roadmap, Contributing)
- **Appendices**: Glossary, API Reference, Troubleshooting

Total: **73 HTML pages** of comprehensive documentation!
