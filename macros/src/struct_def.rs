use proc_macro::{Group, Ident, Punct, Span, TokenStream, TokenTree};

use crate::generics::Generic;
use crate::lifetime::Lifetime;
use crate::proc_macro;
use crate::tuple_iter::Tuplenation;

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

type FieldName = String;
type FieldType = String;

#[derive(Debug)]
pub(crate) struct Field {
    field_type: FieldType,
    field_name: FieldName,
}

impl Field {
    pub(crate) fn new(field_type: TokenTree, field_name: TokenTree) -> Self {
        Self {
            field_type: field_type.to_string(),
            field_name: field_name.to_string(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct StructDef {
    fields: Vec<Field>,
    name: Ident,
    generics: Option<Vec<Generic>>,
    lifetimes: Option<Vec<Lifetime>>,
    orig: TokenStream,
}

impl StructDef {
    pub(crate) fn analyze_stream(tokens: TokenStream) -> Self {
        let mut temp_clone = tokens.clone().into_iter();
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
                    match p_match!(temp_clone.next(), "invalid sequence") {
                        TokenTree::Group(gr) => {
                            if finished {
                                Self::go_through_group(gr, &mut fields);
                                break;
                            } else {
                                panic!("end of generic bound before `>`");
                            }
                        }

                        TokenTree::Punct(punct) if Self::is_lifetime_marker(&punct) => {
                            match p_match!(temp_clone.next()) {
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

                        // TokenTree::Punct(punct) if punct.as_char() == ':' => {
                        //     //
                        //     //
                        // }

                        // TokenTree::Punct(punct) if punct.as_char() == ',' => {
                        //     //
                        //     //
                        // }
                        TokenTree::Punct(punct) if punct.as_char() == '>' => {
                            //

                            finished = true;
                        }

                        TokenTree::Ident(ident) => {
                            //
                            //
                        }

                        _ => panic!("literals should not be here..."),
                    }
                }

                match p_match!(temp_clone.next()) {
                    TokenTree::Group(gr) => Self::go_through_group(gr, &mut fields),
                    _ => panic!("invalid token while parsing generic type/lifetime"),
                }
            }
            _ => panic!("invalid token type"),
        };

        Self {
            name: Ident::new(&temp_name, Span::call_site()),
            fields,
            orig: tokens,
            generics,
            lifetimes,
        }
    }

    fn go_through_group(gr: Group, field_vec: &mut Vec<Field>) {
        let iter = gr.stream().into_iter().filter(|token| match token {
            &TokenTree::Punct(_) => false,
            _ => true,
        });

        let pairs = Tuplenation::new(iter);

        pairs.for_each(|(fname, ftype)| {
            field_vec.push(Field::new(ftype, fname));
        });
    }

    fn is_lifetime_marker(pt: &Punct) -> bool {
        pt.as_char() == '\''
    }
}
