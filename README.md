# p4-lsp

![Tests](https://github.com/ace-design/p4-lsp/actions/workflows/test.yml/badge.svg)

A Rust implementation of the [Language Server Protocol](https://microsoft.github.io/language-server-protocol/) for the [P4 Programming Language](https://p4.org/).

Development is active in the `dev` branch.

## Survey

Fill out the survey to help us design this tool : [Link](https://forms.office.com/r/4hzEvDvXbX)

## Installation

Disclaimer: This software is not production ready and probably won't work out of the box.

### MacOS and Linux

1. [Install Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
```bash
curl https://sh.rustup.rs -sSf | sh
```

2. Install the language server
```bash
cargo install --git https://github.com/ace-design/p4-lsp.git
```

### Windows
Windows is not currently supported.
