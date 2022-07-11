use wasmedge_quickjs::*;

fn main() {
    let mut ctx = Context::new();
    let code = r#"
        import('peg-0.10.0.js').then((peg) => {
            const parser = peg.generate(`
                start = "a"
            `);
            return parser.parse("a");
        });
    "#;

    let p = ctx.eval_global_str(code);
    ctx.promise_loop_poll();
    println!("after poll:{:?}", p);
    if let JsValue::Promise(ref p) = p {
        let v = p.get_result();
        println!("v = {:?}", v);
    }
}

// extern crate pest;
// #[macro_use]
// extern crate pest_derive;

// use pest::Parser;

// #[derive(Parser)]
// #[grammar = "mongodb.pest"]
// pub struct MongoDbParser;

// fn main() {
//     let successful_parse =
//         MongoDbParser::parse(Rule::json, r#"{"status":{"$in":["A","D"]},"x":2}"#);
//     println!("{:?}", successful_parse);
// }
