extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::{error::Error, iterators::Pair, Parser};

#[derive(Parser)]
#[grammar = "mongodb.pest"]
pub struct MongoDbParser;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
struct Expression {
    clauses: Vec<LeafClause>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
struct LeafClause {
    pub key: String,
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
struct ExpressionTreeClause {
    operator: String,
    expressions: Vec<Expression>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
enum Value {
    Leaf(LeafValue),
    Operators(Vec<Operator>),
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
struct LeafValue {
    value: String,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
enum Operator {
    // ElemMatch(ElemMatchOperator),
    // ElemMatchOperatorObject(ElemMatchOperatorObjectOperator),
    List(ListOperator),
    Value(ValueOperator),
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
struct ListOperator {
    pub operator: String,
    pub values: Vec<LeafValue>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
struct ValueOperator {
    pub operator: String,
    pub value: LeafValue,
}

fn main() {
    let str = r#"{"status": "1"}"#;
    let res = parse(str);
    println!("{:?}", res);
}

fn parse(query: &str) -> Result<Expression, Error<Rule>> {
    let pair = MongoDbParser::parse(Rule::query, query)?.next().unwrap();

    fn parse_query(query: Pair<Rule>) -> Expression {
        let expression = query.into_inner().next().unwrap();

        let clause_list = expression.into_inner().next().unwrap();
        match clause_list.as_rule() {
            Rule::clause_list => Expression {
                clauses: parse_clause_list(clause_list),
            },
            _ => unreachable!(),
        }
    }

    fn parse_clause_list(clause: Pair<Rule>) -> Vec<LeafClause> {
        clause.into_inner().map(|pair| parse_clause(pair)).collect()
    }

    fn parse_clause(outer_clause: Pair<Rule>) -> LeafClause {
        let clause = outer_clause.into_inner().next().unwrap();

        match clause.as_rule() {
            Rule::leaf_clause => {
                let mut inner = clause.into_inner();
                let key = inner.next().unwrap().as_str();
                let value = parse_value(inner.next().unwrap());
                LeafClause {
                    key: key.to_string(),
                    value,
                }
            }
            _ => unreachable!(),
        }
    }

    fn parse_value(outer_value: Pair<Rule>) -> Value {
        let value = outer_value.into_inner().next().unwrap();

        match value.as_rule() {
            Rule::leaf_value => Value::Leaf(parse_leaf_value(value)),
            Rule::operator_expression => Value::Operators(parse_operator_expression(value)),
            _ => unreachable!(),
        }
    }

    fn parse_leaf_value(value: Pair<Rule>) -> LeafValue {
        LeafValue {
            value: value.as_str().to_string(),
        }
    }

    fn parse_operator_expression(operator_expression: Pair<Rule>) -> Vec<Operator> {
        println!("operator_expression = {:?}", operator_expression);

        let inner = operator_expression.into_inner().next().unwrap();
        println!("operator_expression inner = {:?}", inner);

        match inner.as_rule() {
            Rule::operator_list => parse_operator_list(inner),
            _ => unreachable!(),
        }
    }

    // FIXME this can be different operator types:
    //       value_operator_type, list_operator_type, elemmatch_expression_operator_type,
    //       operator_expression_operator_type and more special cases not yet handled
    fn parse_operator_list(operator_list: Pair<Rule>) -> Vec<Operator> {
        operator_list
            .into_inner()
            .map(|pair| parse_operator(pair))
            .collect()
    }

    // FIXME this can be different operator types:
    //       value_operator_type, list_operator_type, elemmatch_expression_operator_type,
    //       operator_expression_operator_type and more special cases not yet handled
    fn parse_operator(operator: Pair<Rule>) -> Operator {
        println!("operator = {:?}", operator);
        let operator_type = operator.into_inner().next().unwrap();

        println!("operator_type = {:?}", operator_type);

        match operator_type.as_rule() {
            Rule::list_operator_type => Operator::List(parse_list_operator_type(operator_type)),
            _ => unreachable!(),
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

    fn parse_leaf_value_list(leaf_value_list: Pair<Rule>) -> Vec<LeafValue> {
        println!("\nleaf_value_list = {:?}", leaf_value_list);

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

    #[test]
    fn test_parse_with_list_operator() {
        let expression = parse(r#"{"status":{"$in":["A","D"]},"x":2}"#).unwrap();
        assert_eq!(
            expression,
            Expression {
                clauses: vec![
                    LeafClause {
                        key: "\"status\"".to_string(),
                        value: Value::Operators(vec![Operator::List(ListOperator {
                            operator: "$in".to_string(),
                            values: vec![
                                LeafValue {
                                    value: "\"A\"".to_string(),
                                },
                                LeafValue {
                                    value: "\"D\"".to_string(),
                                },
                            ],
                        }),]),
                    },
                    LeafClause {
                        key: "\"x\"".to_string(),
                        value: Value::Leaf(LeafValue {
                            value: "2".to_string(),
                        })
                    },
                ]
            }
        );
    }

    #[test]
    fn test_parse_simple() {
        let expression = parse(r#"{"status": "1"}"#).unwrap();
        assert_eq!(
            expression,
            Expression {
                clauses: vec![LeafClause {
                    key: "\"status\"".to_string(),
                    value: Value::Leaf(LeafValue {
                        value: "\"1\"".to_string(),
                    })
                }]
            }
        );
    }

    #[test]
    fn test_json_object_null() {
        let parse = MongoDbParser::parse(Rule::json, r#""#);
        assert!(parse.is_ok());
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
}
