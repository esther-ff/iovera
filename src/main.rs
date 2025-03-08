mod lexer;
mod parser;
mod traits;

use ioveca_macros;
use lexer::Lexer;

fn main() {
    let json = concat!(
        "{\n",
        "    \"stringvalue\": \"some string\",\n",
        "    \"intvalue\": 123456789,\n",
        "    \"floatvalue\": 1.23456789,\n",
        "    \"boolvalue\": true,\n",
        "    \"nullvalue\": null,\n",
        "    \"objvalue\": {\"obj\":\"objval\"}, \n",
        "}",
    );

    println!("JSON to test the Lexer: \n{json}\n");
    println!("Text in debug form: {json:#?}\n");

    let lexed = Lexer::new(json).unwrap();

    println!("Processed data:\n\n{lexed:#?}");
}
