# mongodb-language-model

[![CI workflow](https://github.com/fcoury/mongodb-language-model-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/fcoury/mongodb-language-model-rust/actions/workflows/ci.yml)

Parses a MongoDB query and creates an abstract syntax tree (AST) with part of speech
tagging. Currently, only [strict extended json][docs-extended-json] syntax is
supported and not all the cases are being created correctly and some may not be missing
altogether.

This library is based on previous Node.js work by Thomas Rueckstiess on the [mongodb-language-model](https://github.com/mongodb-js/mongodb-language-model) repository. It has been ported from Node.js and PEGjs to Rust and [pest.rs](https://pest.rs/).

## Usage

The module exposes a function `parse(query: &str) -> Result<Expression, Error<Rule>>` that takes a mongo JSON query
string and returns the parsed AST structure.

[docs-extended-json]: https://docs.mongodb.com/manual/reference/mongodb-extended-json/
