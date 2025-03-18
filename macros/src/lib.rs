#![feature(proc_macro_quote)]
#![feature(extend_one)]

mod iters;
mod parser;
mod structures;

use iters::interweave;
use parser::{parser::Parser, *};
use structures::struct_def;

extern crate proc_macro;
use proc_macro::{Delimiter, Group, Ident, Punct, Spacing, Span, TokenStream, TokenTree};
use struct_def::StructDef;

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
pub fn parse_struct(_: TokenStream, items: TokenStream) -> TokenStream {
    let mut parser = Parser::new(items.clone());
    match parser.parse_struct() {
        Ok(def) => println!("{:#?}", def),
        Err(err) => return err.emit(),
    }
    items
}

#[proc_macro_attribute]
pub fn test_struct(_: TokenStream, tkns: TokenStream) -> TokenStream {
    println!("{:#?}", tkns);

    tkns
}
