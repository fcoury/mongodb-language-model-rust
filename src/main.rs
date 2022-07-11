extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::{error::Error, iterators::Pair, Parser};

#[derive(Parser)]
#[grammar = "mongodb.pest"]
pub struct MongoDbParser;

#[derive(Debug, Clone)]
struct Expression {
    clauses: Vec<LeafClause>,
}

#[derive(Debug, Clone)]
struct LeafClause {
    pub key: String,
    pub value: LeafValue,
}

#[derive(Debug, Clone)]
struct LeafValue {
    value: String,
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

    fn parse_value(value: Pair<Rule>) -> LeafValue {
        LeafValue {
            value: value.as_str().to_string(),
        }
    }

    Ok(parse_query(pair))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_object_null() {
        let parse = MongoDbParser::parse(Rule::json, r#"{"status": null}"#);
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
