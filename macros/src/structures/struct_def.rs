#![allow(dead_code)]
use std::iter::Peekable;

use proc_macro::{Group, Ident, Literal, Punct, Span, TokenStream, TokenTree};

use crate::parser::generics::Generic;
use crate::parser::lifetime::Lifetime;
use crate::proc_macro;

macro_rules! p_match {
    ($ex: expr) => {
        match $ex {
            Some(val) => val,
            None => panic!("unexpected end of TokenStream"),
        }
    };

    ($ex: expr, $msg: expr) => {
        match $ex {
            Some(val) => val,
            None => panic!("{}", $msg),
        }
    };
}

static EOF_AT_DEF_NAME: &'static str = "unexpected eof while parsing struct name";
static EOF_AT_TRAIT_BOUND: &'static str = "unexpected eof while parsing trait bounds";
static EOF_WHILE_PARSING_GROUP: &'static str = "unexpected eof while parsing a group of tokens";
static EOF_WHILE_PARSING_TRAIT_BOUND_GROUP: &'static str =
    "unexpected eof while parsing trait bounds in a field of a struct";
static INVALID_TOKEN_TYPE: &'static str = "invalid token parsed";

// struct Parser<I: Iterator<Item = TokenTree>> {
//     iter: Peekable<I>,
//     fin: bool,
// }

// impl<I: Iterator<Item = TokenTree>> Parser<I> {
//     fn eat(&mut self, msg: &str) -> EnumWrap {
//         if self.fin {
//             panic!("continued itearting the parser after finish")
//         }

//         match self.iter.next() {
//             None => panic!("{}", msg),
//             Some(item) => EnumWrap(item),
//         }
//     }

//     fn empty(&mut self) -> bool {
//         self.iter.peek().is_none()
//     }

//     fn peek(&mut self) -> Option<&I::Item> {
//         if self.fin {
//             return None;
//         }

//         self.iter.peek()
//     }

//     fn peek_panic(&mut self, msg: &str) -> &I::Item {
//         if self.fin {
//             panic!("continued iterating the parser after finish")
//         }

//         self.iter.peek().expect(msg)
//     }

//     fn finished(&mut self) {
//         self.fin = true
//     }

//     fn skip(&mut self) {
//         self.iter.next();
//     }

//     fn get_struct_name(&mut self) -> String {
//         loop {
//             if self
//                 .eat(EOF_AT_DEF_NAME)
//                 .ident(INVALID_TOKEN_TYPE)
//                 .to_string()
//                 == "struct"
//             {
//                 break;
//             }
//         }

//         self.eat(EOF_AT_DEF_NAME).to_string()
//     }
// }

struct OptVec<T> {
    vec: Option<Vec<T>>,
    num: usize,
    is_ready: bool,
}

impl<T: std::fmt::Debug> OptVec<T> {
    fn new(num: usize) -> Self {
        Self {
            vec: None,
            num,
            is_ready: false,
        }
    }

    fn push(&mut self, item: T) {
        dbg!(&item);
        if self.is_ready {
            self.vec
                .as_mut()
                .expect("this vec should be here")
                .push(item);

            return;
        }

        let mut container = Vec::with_capacity(self.num);
        container.push(item);

        self.vec = Some(container);
    }

    fn into_self(mut self) -> Option<Vec<T>> {
        self.vec.take()
    }
}

#[derive(Debug)]
pub(crate) struct Field {
    field_type: String,
    field_name: String,
    lifetime: Option<Lifetime>,
}

impl Field {
    pub(crate) fn new(
        field_type: TokenTree,
        field_name: TokenTree,
        lifetime: Option<Lifetime>,
    ) -> Self {
        Self {
            field_type: field_type.to_string(),
            field_name: field_name.to_string(),
            lifetime,
        }
    }
}

#[derive(Debug)]
pub(crate) struct StructDef {
    attrs: Vec<Ident>,
    fields: Vec<Field>,
    name: String,
    generics: Option<Vec<Generic>>,
    lifetimes: Option<Vec<Lifetime>>,
}

impl StructDef {
    pub(crate) fn analyze_stream(tokens: TokenStream) -> Self {
        let mut parser = Parser {
            iter: tokens.clone().into_iter().peekable(),
            fin: false,
        };

        let name = parser.get_struct_name();

        let mut fields = Vec::with_capacity(16);
        let mut generics: OptVec<Generic> = OptVec::new(8);
        let mut lifetimes: OptVec<Lifetime> = OptVec::new(8);

        match parser.eat("eof after struct name").0 {
            TokenTree::Group(group) => {
                Self::go_through_group(group, &mut fields);
                parser.finished();
            }

            TokenTree::Punct(punct) => {
                if punct.as_char() != '<' {
                    panic!("invalid char: {}, should be `<`", punct.as_char());
                }

                // loop to extract anything of the bounds
                let mut finished = false;
                loop {
                    match parser.eat("eof during start of trait bound").0 {
                        TokenTree::Group(gr) => {
                            if finished {
                                Self::go_through_group(gr, &mut fields);
                                break;
                            };

                            panic!("end of trait bound before `>`");
                        }

                        TokenTree::Punct(punct) if Self::is_lifetime_marker(&punct) => {}

                        TokenTree::Punct(punct) if punct.as_char() == '>' => {
                            finished = true;
                            continue;
                        }

                        TokenTree::Punct(punct) => match punct.as_char() {
                            // lifetime marker
                            '\'' => {
                                let lt_name = parser
                                    .eat("eof inside trait bound")
                                    .ident("invalid token while parsing lifetime");
                                if lt_name.to_string() == "mut" {
                                    panic!("definition trait bounds do NOT have mut(s)");
                                }

                                lifetimes.push(Lifetime::new(lt_name.to_string(), punct.span()))
                            }

                            '+' => {}

                            ',' => continue,

                            _ => panic!("invalid char in punct at span: {:#?}", punct.span()),
                        },

                        TokenTree::Ident(ident) => {
                            let mut generic = Generic::new(None, ident);

                            let punct = parser.eat("end of stream").punct(INVALID_TOKEN_TYPE);

                            match punct.as_char() {
                                ':' => {
                                    let poz_trait =
                                        parser.eat(EOF_AT_TRAIT_BOUND).ident(INVALID_TOKEN_TYPE);

                                    generic.insert(poz_trait);

                                    let next =
                                        p_match!(parser.peek(), "eof during peeking for chars");

                                    if next.to_string() == "+" {
                                        parser.skip();

                                        loop {
                                            let poz_trait = parser.eat(EOF_AT_TRAIT_BOUND).0;

                                            match poz_trait {
                                                TokenTree::Ident(tr) => generic.insert(tr),
                                                TokenTree::Punct(punct)
                                                    if punct.as_char() == ',' =>
                                                {
                                                    break;
                                                }

                                                TokenTree::Punct(_punct) => {
                                                    // This is probably a lifetime related trait
                                                    // like `<T: 'a>`
                                                    // TODO: make it...
                                                }

                                                _ => {
                                                    // Right now im gonna consider it unreachable to get a Group or Literal here
                                                    panic!("group or literal obtained")
                                                }
                                            }
                                        }
                                    }
                                }

                                ',' => continue,

                                ch => panic!("invalid char at generic bound: {ch}"),
                            };

                            generics.push(generic)
                        }

                        _ => panic!("found a Literal in struct definition"),
                    }
                }
            }

            _ => panic!("invalid token type"),
        };

        Self {
            name,
            fields,
            generics: generics.into_self(),
            lifetimes: lifetimes.into_self(),
        }
    }

    fn go_through_group(gr: Group, field_vec: &mut Vec<Field>) {
        let mut gr_parser = Parser {
            iter: gr.stream().into_iter().peekable(),
            fin: false,
        };

        'vistula: loop {
            if gr_parser.empty() {
                break 'vistula;
            };

            let monte = gr_parser.eat(EOF_WHILE_PARSING_GROUP);
            dbg!(&monte);
            match monte.0 {
                TokenTree::Ident(ident) => {
                    if gr_parser.eat(EOF_WHILE_PARSING_GROUP).to_string() != ":" {
                        panic!("invalid sequence, field name and no `:` delimeter")
                    }

                    // arbitrary guess, less heap allocation
                    let mut tokens = Vec::with_capacity(64);

                    // fixme: This should match for a ident or punct (for a `&` denoting a lifetime.)
                    match gr_parser.eat(EOF_WHILE_PARSING_GROUP).0 {
                        TokenTree::Ident(type_name) => {
                            // Just a type name
                            tokens.push(TokenTree::Ident(type_name));
                        }

                        TokenTree::Punct(punct) if punct.as_char() == '&' => {
                            // Lifetime marker
                            // Right now it will just skip 2 tokens
                            gr_parser.skip();
                            gr_parser.skip();

                            let type_name = {
                                let token = gr_parser
                                    .eat(EOF_WHILE_PARSING_GROUP)
                                    .ident(INVALID_TOKEN_TYPE);

                                if token.to_string() == "mut" {
                                    gr_parser
                                        .eat(EOF_WHILE_PARSING_GROUP)
                                        .ident(INVALID_TOKEN_TYPE)
                                } else {
                                    // skips the `,`
                                    // invalid for a type like `&'a mut IoPipe<'a>`
                                    // TODO:
                                    gr_parser.skip();
                                    token
                                }
                            };

                            tokens.push(TokenTree::Ident(type_name));
                        }

                        // Rest are impossible
                        wh => panic!("{}, {wh:#?}", INVALID_TOKEN_TYPE),
                    }

                    println!("Did we get here?");

                    // We're inside a generic/trait bound whatever!
                    let token = gr_parser.eat(EOF_WHILE_PARSING_TRAIT_BOUND_GROUP);
                    if token.to_string() == "<" {
                        tokens.push(token.0);

                        // read till `>`
                        'volga: loop {
                            let token = gr_parser.eat(EOF_WHILE_PARSING_TRAIT_BOUND_GROUP);

                            if token.to_string() == ">" {
                                tokens.push(token.0);
                                break 'volga;
                            }

                            tokens.push(token.0);
                        }
                    };

                    // NASTY!
                    let field_type = TokenStream::from_iter(tokens.into_iter()).to_string();

                    field_vec.push(Field {
                        field_type,
                        field_name: ident.to_string(),
                        lifetime: None, // <-- evil
                    });
                }

                what => {
                    dbg!(what);
                }
            }
        }
    }

    fn is_lifetime_marker(pt: &Punct) -> bool {
        pt.as_char() == '\''
    }
}

#[derive(Debug)]
struct EnumWrap(TokenTree);

macro_rules! enum_wrap_impl {
    ($item: ident, $what: expr, $msg: expr) => {
        match $what.0 {
            TokenTree::$item(item) => item,
            _ => panic!("{}", $msg),
        }
    };
}

impl EnumWrap {
    fn group(self, msg: &str) -> Group {
        enum_wrap_impl!(Group, self, msg)
    }

    fn ident(self, msg: &str) -> Ident {
        enum_wrap_impl!(Ident, self, msg)
    }

    fn punct(self, msg: &str) -> Punct {
        enum_wrap_impl!(Punct, self, msg)
    }

    fn literal(self, msg: &str) -> Literal {
        enum_wrap_impl!(Literal, self, msg)
    }
}

impl std::fmt::Display for EnumWrap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
