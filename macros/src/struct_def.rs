#![allow(dead_code)]
use proc_macro::{Group, Ident, Punct, Span, TokenStream, TokenTree};

use crate::generics::Generic;
use crate::lifetime::Lifetime;
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

macro_rules! is_this {
    ($tkn: ident, $target: ident) => {
        match $tkn {
            TokenTree::$target(_) => true,

            _ => false,
        }
    };

    ($tkn: ident, $target: ident, $cond: expr) => {
        match $tkn {
            TokenTree::$target(p) => p.to_string() == $cond,
            _ => false,
        }
    };
}

type FieldName = String;
type FieldType = String;

#[derive(Debug)]
pub(crate) struct Field {
    field_type: FieldType,
    field_name: FieldName,
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
    fields: Vec<Field>,
    name: Ident,
    generics: Option<Vec<Generic>>,
    lifetimes: Option<Vec<Lifetime>>,
}

impl StructDef {
    pub(crate) fn analyze_stream(tokens: TokenStream) -> Self {
        let mut temp_clone = tokens.clone().into_iter().peekable();
        let temp_name: String;

        loop {
            let token = p_match!(temp_clone.next());
            if token.to_string() == "struct" {
                let next = p_match!(temp_clone.next());
                temp_name = next.to_string();
                break;
            }
        }

        let mut fields = Vec::with_capacity(16);
        let mut generics: Option<Vec<Generic>> = None;
        let mut lifetimes: Option<Vec<Lifetime>> = None;

        match p_match!(temp_clone.next()) {
            TokenTree::Group(group) => {
                Self::go_through_group(group, &mut fields);
            }

            TokenTree::Punct(punct) => {
                if punct.as_char() != '<' {
                    panic!("invalid char: {}, should be `<`", punct.as_char());
                }

                // loop to extract anything of the bounds
                let mut finished = false;
                loop {
                    match p_match!(temp_clone.next(), "eof during start of trait bound") {
                        TokenTree::Group(gr) => {
                            if finished {
                                Self::go_through_group(gr, &mut fields);
                                break;
                            } else {
                                panic!("end of trait bound before `>`");
                            }
                        }

                        TokenTree::Punct(punct) if Self::is_lifetime_marker(&punct) => {
                            match p_match!(temp_clone.next(), "eof inside trait bound") {
                                TokenTree::Ident(lt_name) => {
                                    if lt_name.to_string() == "mut" {
                                        panic!("definition trait bounds do NOT have mut(s)");
                                    }

                                    if lifetimes.is_none() {
                                        lifetimes = Some(Vec::with_capacity(4))
                                    }

                                    lifetimes
                                        .as_mut()
                                        .unwrap()
                                        .push(Lifetime::new(false, lt_name.to_string()))
                                }

                                _ => panic!("invalid token while parsing lifetime"),
                            }
                        }

                        TokenTree::Punct(punct) if punct.as_char() == '>' => {
                            finished = true;
                        }

                        TokenTree::Ident(ident) => {
                            let mut generic = Generic::new(None, ident);

                            match temp_clone.next() {
                                None => panic!("end of stream"),

                                Some(ref v) if is_this!(v, Punct, ":") => loop {
                                    let token = p_match!(
                                        temp_clone.next(),
                                        "eof during analyzing trait bound"
                                    );

                                    if is_this!(token, Ident) {
                                        match token {
                                            TokenTree::Ident(ident) => generic.insert(ident),

                                            _ => unreachable!(),
                                        }
                                    }

                                    match p_match!(
                                        temp_clone.peek(),
                                        "eof during peeking for chars"
                                    ) {
                                        TokenTree::Punct(punct) if punct.as_char() == '+' => {
                                            temp_clone.next();
                                        }
                                        TokenTree::Punct(punct) if punct.as_char() == ',' => {
                                            temp_clone.next();
                                            break;
                                        }

                                        TokenTree::Punct(punct) if punct.as_char() == '>' => {
                                            finished = true;
                                        }

                                        TokenTree::Group(_) => break,

                                        _ => panic!("invalid char stream aaaaa"),
                                    }
                                },

                                Some(ch) if ch.to_string() == "," => {}

                                Some(ch) => panic!("invalid char at generic bound: {ch}"),
                            };

                            match generics {
                                Some(ref mut v) => v.push(generic),
                                None => {
                                    generics = Some(Vec::with_capacity(16));
                                    generics.as_mut().unwrap().push(generic)
                                }
                            }
                        }

                        TokenTree::Punct(p) => {
                            dbg!(p);
                        }

                        _ => panic!("literals should not be here..."),
                    }
                }
            }
            _ => panic!("invalid token type"),
        };

        Self {
            name: Ident::new(&temp_name, Span::call_site()),
            fields,
            generics,
            lifetimes,
        }
    }

    fn go_through_group(gr: Group, field_vec: &mut Vec<Field>) {
        let mut iter = gr.stream().into_iter().peekable();

        loop {
            let monte = iter.next();
            dbg!(&monte);
            match p_match!(monte) {
                TokenTree::Ident(ident) => {
                    if p_match!(iter.next()).to_string() != ":" {
                        panic!("invalid sequence, field name and no `:` delimeter")
                    }

                    // fixme: This should match for a ident or punct (for a `&` denoting a lifetime.)
                    let _basic_type = get_ident(p_match!(iter.next()));

                    // We're inside a generic/trait bound whatever!
                    if p_match!(iter.next()).to_string() == "<" {
                        // arbitrary guess, less heap allocation
                        let mut tokens = Vec::with_capacity(8);

                        // read till `>`
                        loop {
                            let token =
                                p_match!(iter.next(), "end of stream during parsing fields");

                            if token.to_string() == ">" {
                                break;
                            }

                            tokens.push(token);
                        }

                        // NASTY!
                        let field_type = TokenStream::from_iter(tokens.into_iter()).to_string();

                        field_vec.push(Field {
                            field_type,
                            field_name: ident.to_string(),
                            lifetime: None, // <-- evil
                        });
                    }
                }

                _ => unreachable!(),
            }
        }
        //let pairs = Tuplenation::new(iter);

        // pairs.for_each(|(fname, ftype)| {
        //     field_vec.push(Field::new(ftype, fname));
        // });
    }

    fn is_lifetime_marker(pt: &Punct) -> bool {
        pt.as_char() == '\''
    }
}

// Remake those functions into a some sort Enum trait
// this is bullshit.

fn get_ident(tree: TokenTree) -> Ident {
    match tree {
        TokenTree::Ident(ident) => ident,
        _ => panic!("invalid token type, should be a ident"),
    }
}

fn get_punct(tree: TokenTree) -> Punct {
    match tree {
        TokenTree::Punct(punct) => punct,
        _ => panic!("invalid token type, should be a punct"),
    }
}

fn get_group(tree: TokenTree) -> Group {
    match tree {
        TokenTree::Group(gr) => gr,
        _ => panic!("invalid token type, should be a group"),
    }
}
