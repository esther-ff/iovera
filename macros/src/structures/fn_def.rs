use proc_macro::{Group, Ident, Punct, Spacing, Span};

use crate::TokenStream;
use crate::parser::marker::{self, Marker};
use crate::proc_macro::TokenTree;
use crate::ty::Ty;
use std::iter;

#[derive(Debug)]
struct Arg {
    name: Ident,
    arg_type: Ty,
}

impl Arg {
    pub fn new(name: Ident, arg_type: Ty) -> Self {
        Self { name, arg_type }
    }

    pub fn to_tokens(self) -> impl Iterator<Item = TokenTree> {
        iter::once(TokenTree::Ident(self.name))
            .chain(iter::once(TokenTree::Punct(Punct::new(
                ':',
                Spacing::Alone,
            ))))
            .chain(self.arg_type.to_tokens())
    }

    pub fn vec_to_group(args: Vec<Arg>) -> Group {
        let mut stream = TokenStream::new();

        let mut iter = args.into_iter();

        while let Some(arg) = iter.next() {
            stream.extend(arg.to_tokens());

            match iter.next() {
                Some(another_arg) => {
                    let sep = Punct::new(',', Spacing::Alone);

                    let iter = iter::once(TokenTree::Punct(sep.clone()))
                        .chain(another_arg.to_tokens())
                        .chain(iter::once(TokenTree::Punct(sep)));

                    stream.extend(iter);
                }

                None => break,
            }
        }

        Group::new(proc_macro::Delimiter::Parenthesis, stream)
    }
}

#[derive(Debug)]
pub struct RetType {
    ty: Ty,
}

impl RetType {
    pub fn new(ty: Ty) -> Self {
        Self { ty }
    }

    pub fn to_tokens(self) -> impl Iterator<Item = TokenTree> {
        let arrow_start = TokenTree::Punct(Punct::new('-', Spacing::Joint));
        let arrow_end = TokenTree::Punct(Punct::new('>', Spacing::Joint));

        iter::once(arrow_start)
            .chain(iter::once(arrow_end))
            .chain(self.ty.to_tokens())
    }
}

#[derive(Debug)]
pub struct FnBody {
    // Here go dragons!
    // TODO!
    // mayb just tokens
    // tkns: Vec<TokenTree>,
}

#[derive(Debug)]
/// Represents a function definition
pub struct FnDef {
    markers: Vec<Marker>,
    args: Vec<Arg>,
    name: String,
    ret_type: Option<RetType>,
    body: Option<FnBody>,
}

impl FnDef {
    pub fn new(name: String) -> Self {
        Self {
            markers: Vec::new(),
            args: Vec::new(),
            name,
            ret_type: None,
            body: None,
        }
    }

    pub fn set_name(&mut self, nom: &str) -> &mut Self {
        self.name = nom.to_string();
        self
    }

    pub fn add_arg(&mut self, arg: Arg) -> &mut Self {
        self.args.push(arg);
        self
    }

    pub fn set_ret_type(&mut self, ret_type: Ty) -> &mut Self {
        self.ret_type = Some(RetType::new(ret_type));
        self
    }

    pub fn to_tokens(mut self) -> TokenStream {
        let mut stream = TokenStream::new();

        for marker in self.markers {
            marker::marker_to_tokens!(marker, &mut stream);
        }

        let name = Ident::new(&self.name, Span::mixed_site());
        let args = Arg::vec_to_group(self.args);

        stream.extend([TokenTree::Ident(name), TokenTree::Group(args)]);

        if let Some(ret) = self.ret_type.take() {
            stream.extend(ret.to_tokens());
        }

        /* TODO!
        if let Some(body) = self.body.take() {
            stream.extend(ret.to_tokens());
        }
        */

        stream
    }
}
