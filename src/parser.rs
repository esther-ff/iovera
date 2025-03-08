use crate::lexer::{Lexer, Token};

struct Parser<'p> {
    lexer: Lexer<'p>,
    last_token: Token,
}
