# mongodb-language-model

Parses a MongoDB query and creates an abstract syntax tree (AST) with part of speech
tagging. Currently, only [strict extended json][docs-extended-json] syntax is
supported and not all the cases are being created correctly and some may not be missing
altogether.

This library is based on previous work by Thomas Rueckstiess <thomas@mongodb.com> and
has been ported to Rust and pest.rs.

## Usage

The module exposes a function `parse(queryStr)`.

#### `parse(queryStr)`

The `accepts(queryStr)` function takes a query string and returns `true` if the
string is a valid MongoDB query, `false` otherwise.

Example:

```javascript
var accepts = require('mongodb-language-model').accepts;
var assert = require('assert');

assert.ok(accepts('{"foo": 1}'));
assert.ok(accepts('{"age": {"$gt": 35}}'));
assert.ok(accepts('{"$or": [{"email": {"$exists": true}}, {"phone": {"$exists": true}}]}'));

assert.equal(accepts('{"$invalid": "key"}'), false);
```

#### `parse(queryStr)`

The `parse(queryStr)` function takes a query string and returns an abstract
syntax tree (AST) as a javascript object, if the query is valid. If the
query is not valid, the function throws a `pegjs.SyntaxError` with a message
explaining the failure.

Example:

```javascript
var parse = require('mongodb-language-model').parse;
var assert = require('assert');
var pegjs = require('pegjs');

var ast = parse('{"foo": "bar"}');
assert.deepEqual(ast, {
  'pos': 'expression',
  'clauses': [
    {
      'pos': 'leaf-clause',
      'key': 'foo',
      'value': {
        'pos': 'leaf-value',
        'value': 'bar'
      }
    }
  ]
});
```

## UML diagram

This is the hierarchical model that is created when a query is parsed:

![](./docs/query_language_uml.png)



## Installation

```
npm install --save mongodb-language-model
```

## Testing

```
npm test
```

## License

Apache 2.0

[docs-extended-json]: https://docs.mongodb.com/manual/reference/mongodb-extended-json/
