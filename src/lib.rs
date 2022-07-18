//! `mongodb-language-model` is a library for parsing the MongoDB language and
//! returning an abstract syntax tree using pest.rs.
//!
//! # Example
//!
//! ```rust
//! use mongodb_language_model::*;
//!
//! let input = r#"{ "$or": [ { "status": "A" }, { "qty": { "$lt": 30 } }] }"#;
//! let ast = parse(input).unwrap();
//! ```

extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::{error::Error, iterators::Pair, Parser};

#[derive(Parser)]
#[grammar = "mongodb.pest"]
pub struct MongoDbParser;

#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
    pub clauses: Vec<Clause>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Clause {
    Leaf(LeafClause),
    ExpressionTree(ExpressionTreeClause),
}

#[derive(Debug, Clone, PartialEq)]
pub struct LeafClause {
    pub key: String,
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExpressionTreeClause {
    pub operator: String,
    pub expressions: Vec<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Leaf(LeafValue),
    Operators(Vec<Operator>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct LeafValue {
    pub value: serde_json::Value,
}

// FIXME this can be different operator types:
//       value_operator_type, list_operator_type, elemmatch_expression_operator_type,
//       operator_expression_operator_type and more special cases not yet handled
#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    // ElemMatch(ElemMatchOperator),
    // ElemMatchOperatorObject(ElemMatchOperatorObjectOperator),
    List(ListOperator),
    Value(ValueOperator),
    ExpressionOperator(OperatorExpressionOperator),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListOperator {
    pub operator: String,
    pub values: Vec<LeafValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValueOperator {
    pub operator: String,
    pub value: LeafValue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OperatorExpressionOperator {
    pub operator: String,
    pub operators: Vec<Operator>,
}

pub fn parse(query: &str) -> Result<Expression, Error<Rule>> {
    let pair = MongoDbParser::parse(Rule::query, query)?.next().unwrap();

    fn parse_query(query: Pair<Rule>) -> Expression {
        let expression = query.into_inner().next().unwrap();
        parse_expression(expression)
    }

    fn parse_expression(expression: Pair<Rule>) -> Expression {
        let clause_list = expression.clone().into_inner().next().unwrap();
        match clause_list.as_rule() {
            Rule::clause_list => Expression {
                clauses: parse_clause_list(clause_list),
            },
            t => unreachable!("parse_expression: {:?}\ngot: {:?}", t, expression),
        }
    }

    fn parse_clause_list(clause: Pair<Rule>) -> Vec<Clause> {
        clause.into_inner().map(|pair| parse_clause(pair)).collect()
    }

    fn parse_clause(outer_clause: Pair<Rule>) -> Clause {
        let clause = outer_clause.clone().into_inner().next().unwrap();

        match clause.as_rule() {
            Rule::leaf_clause => {
                let mut inner = clause.into_inner();
                let key = inner.next().unwrap().as_str();
                let value = parse_value(inner.next().unwrap());
                Clause::Leaf(LeafClause {
                    key: serde_json::from_str(key).unwrap(),
                    value,
                })
            }
            Rule::expression_tree_clause => {
                let mut inner = clause.into_inner();
                inner.next(); // quotation_mark
                let operator = inner.next().unwrap().as_str();
                inner.next(); // quotation_mark
                let expression_list = inner.next().unwrap();
                let expressions: Vec<Expression> = expression_list
                    .into_inner()
                    .map(|pair| parse_expression(pair))
                    .collect();
                Clause::ExpressionTree(ExpressionTreeClause {
                    operator: operator.to_string(),
                    expressions, // TODO parse_expression_tree(inner.next().unwrap()),
                })
            }
            t => unreachable!("parse_clause: {:?}\nGot: {:?}", t, outer_clause),
        }
    }

    fn parse_value(outer_value: Pair<Rule>) -> Value {
        let value = outer_value.into_inner().next().unwrap();
        parse_value_inner(value)
    }

    fn parse_value_inner(value: Pair<Rule>) -> Value {
        match value.as_rule() {
            Rule::leaf_value => Value::Leaf(parse_leaf_value(value)),
            Rule::operator_expression => Value::Operators(parse_operator_expression(value)),
            t => unreachable!("parse_value: {:?}\nGot: {:?}", t, value),
        }
    }

    fn parse_leaf_value(value: Pair<Rule>) -> LeafValue {
        let inner = value.clone().into_inner().next().unwrap();
        match inner.as_rule() {
            Rule::string => LeafValue {
                value: serde_json::from_str(inner.as_str()).unwrap(),
            },
            Rule::number => LeafValue {
                value: serde_json::from_str(inner.as_str()).unwrap(),
            },
            Rule::object => LeafValue {
                value: parse_value_object(inner),
            },
            Rule::false_lit => LeafValue {
                value: serde_json::json!(false),
            },
            Rule::true_lit => LeafValue {
                value: serde_json::json!(true),
            },
            Rule::null => LeafValue {
                value: serde_json::json!(null),
            },
            t => unreachable!("parse_leaf_value: {:?}\nGot: {:?}", t, inner),
        }
    }

    fn parse_value_object(value: Pair<Rule>) -> serde_json::Value {
        let json: serde_json::Value = serde_json::from_str(value.as_str()).unwrap();
        let key = json.as_object().unwrap().keys().next().unwrap().as_str();
        match key {
            "$f" | "$numberDecimal" => json.get(key).unwrap().clone(),
            _ => json,
        }
    }

    fn parse_operator_expression(operator_expression: Pair<Rule>) -> Vec<Operator> {
        let inner = operator_expression.clone().into_inner().next().unwrap();

        match inner.as_rule() {
            Rule::operator_list => parse_operator_list(inner),
            t => unreachable!("parse_operator_expression: {:?}\nGot: {:?}", t, inner),
        }
    }

    fn parse_operator_list(operator_list: Pair<Rule>) -> Vec<Operator> {
        operator_list
            .into_inner()
            .map(|pair| parse_operator(pair))
            .collect()
    }

    fn parse_operator(operator: Pair<Rule>) -> Operator {
        let operator_type = operator.clone().into_inner().next().unwrap();
        match operator_type.as_rule() {
            Rule::list_operator_type => Operator::List(parse_list_operator_type(operator_type)),
            Rule::value_operator_type => Operator::Value(parse_value_operator_type(operator_type)),
            Rule::operator_expression_operator_type => {
                Operator::ExpressionOperator(parse_operator_expression_operator_type(operator_type))
            }
            t => unreachable!("parse_operator: {:?}\nGot: {:?}", t, operator_type),
        }
    }

    fn parse_operator_expression_operator_type(
        operator_type: Pair<Rule>,
    ) -> OperatorExpressionOperator {
        let mut inner = operator_type.into_inner();
        inner.next(); // quotation_mark
        let operator = inner.next().unwrap().as_str();
        inner.next(); // quotation_mark
        let operator_expression = inner.next().unwrap();
        let operators = parse_operator_expression(operator_expression);
        OperatorExpressionOperator {
            operator: operator.to_string(),
            operators,
        }
    }

    fn parse_list_operator_type(operator_type: Pair<Rule>) -> ListOperator {
        let mut inner = operator_type.into_inner();
        inner.next();
        let operator = inner.next().unwrap();
        inner.next();
        let leaf_value_list = inner.next().unwrap();

        ListOperator {
            operator: operator.as_str().to_string(),
            values: parse_leaf_value_list(leaf_value_list),
        }
    }

    fn parse_value_operator_type(operator_type: Pair<Rule>) -> ValueOperator {
        let mut inner = operator_type.into_inner();
        inner.next(); // quotation_mark
        let operator = inner.next().unwrap().as_str();
        inner.next(); // quotation_mark
        let leaf_value = inner.next().unwrap();

        ValueOperator {
            operator: operator.to_string(),
            value: parse_leaf_value(leaf_value),
        }
    }

    fn parse_leaf_value_list(leaf_value_list: Pair<Rule>) -> Vec<LeafValue> {
        leaf_value_list
            .into_inner()
            .map(|pair| parse_leaf_value(pair))
            .collect()
    }

    Ok(parse_query(pair))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_simple() {
        let expression = parse(r#"{"status": "1"}"#).unwrap();
        assert_eq!(
            expression,
            Expression {
                clauses: vec![Clause::Leaf(LeafClause {
                    key: "status".to_string(),
                    value: Value::Leaf(LeafValue { value: json!("1") })
                })]
            }
        );
    }

    #[test]
    fn test_parse_true_bool() {
        let expression = parse(r#"{"status": true}"#).unwrap();
        assert_eq!(
            expression,
            Expression {
                clauses: vec![Clause::Leaf(LeafClause {
                    key: "status".to_string(),
                    value: Value::Leaf(LeafValue { value: json!(true) })
                })]
            }
        );
    }

    #[test]
    fn test_parse_false_bool() {
        let expression = parse(r#"{"status": false}"#).unwrap();
        assert_eq!(
            expression,
            Expression {
                clauses: vec![Clause::Leaf(LeafClause {
                    key: "status".to_string(),
                    value: Value::Leaf(LeafValue {
                        value: json!(false)
                    })
                })]
            }
        );
    }

    #[test]
    fn test_parse_null() {
        let expression = parse(r#"{"status": null}"#).unwrap();
        assert_eq!(
            expression,
            Expression {
                clauses: vec![Clause::Leaf(LeafClause {
                    key: "status".to_string(),
                    value: Value::Leaf(LeafValue { value: json!(null) })
                })]
            }
        );
    }

    #[test]
    fn test_parse_simple_with_extended_double() {
        let expression = parse(r#"{"x":{"$f":1.2}}"#).unwrap();
        assert_eq!(
            expression,
            Expression {
                clauses: vec![Clause::Leaf(LeafClause {
                    key: "x".to_string(),
                    value: Value::Leaf(LeafValue { value: json!(1.2) })
                })]
            }
        );
    }

    #[test]
    fn test_parse_simple_with_alt_extended_double() {
        let expression = parse(r#"{"status": { "$numberDecimal": 1.2 }}"#).unwrap();
        assert_eq!(
            expression,
            Expression {
                clauses: vec![Clause::Leaf(LeafClause {
                    key: "status".to_string(),
                    value: Value::Leaf(LeafValue { value: json!(1.2) })
                })]
            }
        );
    }

    #[test]
    fn test_parse_simple_with_double() {
        let expression = parse(r#"{"status": 1.2}"#).unwrap();
        assert_eq!(
            expression,
            Expression {
                clauses: vec![Clause::Leaf(LeafClause {
                    key: "status".to_string(),
                    value: Value::Leaf(LeafValue { value: json!(1.2) })
                })]
            }
        );
    }

    #[test]
    fn test_parse_with_regex() {
        // FIXME support regex
        // let expression =
        //     parse(r#"{"status":"A","$or":[{"qty":{"$lt":30}},{"item":{"$regex":"/^p/"}}]}"#)
        //         .unwrap();
        // assert_eq!(expression, Expression { clauses: vec![] });
    }

    #[test]
    fn test_parse_with_or() {
        let expression = parse(r#"{"$or":[{"status":"A"},{"qty":{"$lt":30}}]}"#).unwrap();
        assert_eq!(
            expression,
            Expression {
                clauses: vec![Clause::ExpressionTree(ExpressionTreeClause {
                    operator: "$or".to_string(),
                    expressions: vec![
                        Expression {
                            clauses: vec![Clause::Leaf(LeafClause {
                                key: "status".to_string(),
                                value: Value::Leaf(LeafValue { value: json!("A") })
                            })],
                        },
                        Expression {
                            clauses: vec![Clause::Leaf(LeafClause {
                                key: "qty".to_string(),
                                value: Value::Operators(vec![Operator::Value(ValueOperator {
                                    operator: "$lt".to_string(),
                                    value: LeafValue { value: json!(30) }
                                })])
                            })],
                        },
                    ],
                })]
            }
        );
    }

    #[test]
    fn test_parse_with_list_operator() {
        let expression = parse(r#"{"status":{"$in":["A","D"]},"x":2}"#).unwrap();
        assert_eq!(
            expression,
            Expression {
                clauses: vec![
                    Clause::Leaf(LeafClause {
                        key: "status".to_string(),
                        value: Value::Operators(vec![Operator::List(ListOperator {
                            operator: "$in".to_string(),
                            values: vec![
                                LeafValue { value: json!("A") },
                                LeafValue { value: json!("D") },
                            ],
                        }),]),
                    }),
                    Clause::Leaf(LeafClause {
                        key: "x".to_string(),
                        value: Value::Leaf(LeafValue { value: json!(2) })
                    }),
                ]
            }
        );
    }

    #[test]
    fn test_parse_simple_with_not() {
        let expression = parse(r#"{"age":{"$not":{"$gt":12}}}"#).unwrap();
        assert_eq!(
            expression,
            Expression {
                clauses: vec![Clause::Leaf(LeafClause {
                    key: "age".to_string(),
                    value: Value::Operators(vec![Operator::ExpressionOperator(
                        OperatorExpressionOperator {
                            operator: "$not".to_string(),
                            operators: vec![Operator::Value(ValueOperator {
                                operator: "$gt".to_string(),
                                value: LeafValue { value: json!(12) }
                            })]
                        }
                    )])
                })]
            }
        );
    }

    #[test]
    fn test_object_null() {
        let parse = MongoDbParser::parse(Rule::object, r#"{"status": null}"#);
        assert!(parse.is_ok());
    }

    #[test]
    fn test_object_string() {
        let parse = MongoDbParser::parse(Rule::object, r#"{"status": "some"}"#);
        assert!(parse.is_ok());
    }

    #[test]
    fn test_member_null() {
        let parse = MongoDbParser::parse(Rule::member, r#""status": null"#);
        assert!(parse.is_ok());
    }

    #[test]
    fn test_member_false() {
        let parse = MongoDbParser::parse(Rule::member, r#""status": false"#);
        assert!(parse.is_ok());
    }

    #[test]
    fn test_member_true() {
        let parse = MongoDbParser::parse(Rule::member, r#""status": true"#);
        assert!(parse.is_ok());
    }

    #[test]
    fn test_member_string() {
        let parse = MongoDbParser::parse(Rule::member, r#""status": "true""#);
        assert!(parse.is_ok());
    }

    #[test]
    fn test_member_decimal_number() {
        let parse = MongoDbParser::parse(Rule::member, r#""status": 1.2"#);
        assert!(parse.is_ok());
    }

    #[test]
    fn test_member_explicit_decimal_number() {
        let parse = MongoDbParser::parse(Rule::member, r#""status": { "$numberDecimal": 1.2 }"#);
        assert!(parse.is_ok());
    }

    #[test]
    fn test_member_explicit_alt_decimal_number() {
        let parse = MongoDbParser::parse(Rule::member, r#""status": { "$f": 1.2 }"#);
        assert!(parse.is_ok());
    }
}
