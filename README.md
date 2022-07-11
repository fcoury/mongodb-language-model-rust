# mongodb-language-model

[![CI workflow](https://github.com/fcoury/mongodb-language-model/actions/workflows/ci.yml/badge.svg)](https://github.com/fcoury/mongodb-language-model/actions/workflows/ci.yml)

Parses a MongoDB query and creates an abstract syntax tree (AST) with part of speech
tagging. Currently, only [strict extended json][docs-extended-json] syntax is
supported and not all the cases are being created correctly and some may not be missing
altogether.

This library is based on previous work by Thomas Rueckstiess <thomas@mongodb.com> and
has been ported to Rust and pest.rs.

## Usage

The module exposes a function `parse(query: &str) -> Result<Expression, Error<Rule>>`.

