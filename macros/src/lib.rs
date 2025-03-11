#![feature(proc_macro_quote)]
#![feature(extend_one)]

mod iters;
mod parser;
mod structures;

use iters::*;
use parser::{parser::Parser, *};
use structures::struct_def;

extern crate proc_macro;
use proc_macro::{Delimiter, Group, Ident, Punct, Spacing, Span, TokenStream, TokenTree, quote};
use struct_def::StructDef;

#[proc_macro_attribute]
pub fn test_stuff(_attr: TokenStream, items: TokenStream) -> TokenStream {
    println!("{:#?}", items);
    items
}

#[proc_macro_attribute]
pub fn json_deser(_attr: TokenStream, items: TokenStream) -> TokenStream {
    let mut iter = items.clone().into_iter();
    let mut struct_name = Ident::new("invalid", Span::call_site());

    'rhein: loop {
        let token = match iter.next() {
            Some(tkn) => tkn,
            None => panic!("empty token stream"),
        };

        match token {
            TokenTree::Ident(name) => {
                if name.to_string() == "struct".to_string() {
                    match iter.next().unwrap() {
                        TokenTree::Ident(ident) => {
                            struct_name = ident;
                            break 'rhein;
                        }

                        _ => panic!("invalid!"),
                    }
                } else {
                    match iter.next() {
                        None => panic!("incomplete token stream"),

                        Some(token) => match token {
                            TokenTree::Ident(name) => {
                                if name.to_string() == "struct".to_string() {
                                    match iter.next().unwrap() {
                                        TokenTree::Ident(ident) => {
                                            struct_name = ident;
                                            break 'rhein;
                                        }

                                        _ => panic!("invalid!"),
                                    }
                                }
                            }

                            _ => panic!("no struct name"),
                        },
                    }
                }
            }

            _ => {
                break;
            }
        };
    }

    let mut fields: Vec<TokenTree> = vec![];
    let mut f_types: Vec<TokenTree> = vec![];

    let group = match iter.next() {
        Some(group) => {
            //
            match group {
                TokenTree::Group(gr) => gr.stream(),
                _ => panic!("invalid struct, no group after name"),
            }
        }
        None => panic!("invalid struct, braces but no body"),
    };

    let mut group_iter = group.into_iter().filter(|token| match token {
        TokenTree::Punct(_) => false,
        _ => true,
    });

    loop {
        let field_name = match group_iter.next() {
            None => break,
            Some(name) => name,
        };

        fields.push(field_name);

        let field_type = match group_iter.next() {
            None => break,
            Some(ftype) => ftype,
        };

        f_types.push(field_type);
    }

    let struct_def = self_struct_def_gen(&fields);
    let struct_def_args = struct_gen_new_fn_args(&fields, &f_types);

    let stream = quote!(
        impl $struct_name {
            pub fn new($struct_def_args) -> Self {
                $struct_def
            }
        }
    );

    println!("{:#?}", stream);

    let mut items_mut = items.clone();
    items_mut.extend(stream.into_iter());
    items_mut
}

fn self_struct_def_gen(fields: &[TokenTree]) -> TokenStream {
    let mut tokens = TokenStream::new();
    let mut group_tokens = TokenStream::new();

    tokens.extend_one(TokenTree::Ident(Ident::new("Self", Span::call_site())));

    fields.iter().for_each(|name| {
        group_tokens.extend_one(TokenTree::Ident(Ident::new(
            &name.to_string(),
            Span::call_site(),
        )));
        group_tokens.extend_one(TokenTree::Punct(Punct::new(',', Spacing::Alone)));
    });

    let group = Group::new(Delimiter::Brace, group_tokens);

    tokens.extend_one(TokenTree::Group(group));
    tokens
}

fn struct_gen_new_fn_args(fl: &[TokenTree], ty: &[TokenTree]) -> TokenStream {
    if fl.len() != ty.len() {
        panic!("the amounts of the fields and their types is not the same");
    }

    let mut args = TokenStream::new();
    let mut count = 1;

    fl.iter().zip(ty.iter()).for_each(|(field, ftype)| {
        let mut arr: Vec<TokenTree> = Vec::with_capacity(3);

        let mut clone_field = field.clone();
        clone_field.set_span(Span::call_site());
        arr.push(clone_field);

        arr.push(TokenTree::Punct(Punct::new(':', Spacing::Alone)));

        let mut clone_ftype = ftype.clone();
        clone_ftype.set_span(Span::call_site());
        arr.push(clone_ftype);

        if count != fl.len() {
            arr.push(TokenTree::Punct(Punct::new(',', Spacing::Alone)));
        }

        args.extend(arr.into_iter());

        count += 1
    });

    args
}

#[proc_macro_attribute]
pub fn show_token_stream_debug(_: TokenStream, items: TokenStream) -> TokenStream {
    let mut parser = Parser::new(items.clone());
    parser.test_spit();
    items
}

#[proc_macro_attribute]
pub fn parse_struct(_: TokenStream, tkns: TokenStream) -> TokenStream {
    println!("{:#?}", StructDef::analyze_stream(tkns.clone()));

    tkns
}
