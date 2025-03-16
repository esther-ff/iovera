use proc_macro::{Ident, Punct, Span};

use crate::TokenStream;
use crate::extra_iter::Extra;
use crate::proc_macro::TokenTree;
use crate::ty::ParsedType;
use std::iter;

#[derive(Debug)]
struct Arg {
    name: Ident,
    arg_type: ParsedType,
}

impl Arg {
    pub fn new(name: Ident, arg_type: ParsedType) -> Self {
        Self { name, arg_type }
    }

    pub fn to_tokens(self) -> IntoIterator<Item = TokenTree> {
        let base_iter = iter::once(TokenTree::Ident(self.name)).chain(self.arg_type.to_tokens());
    }
}

#[derive(Debug)]
/// Represents a function definition
pub(crate) struct FnDef {
    args: Vec<Arg>,
    name: Option<String>,
    ret_type: Option<String>,
}

impl FnDef {
    pub(crate) fn new() -> Self {
        Self {
            args: Vec::new(),
            name: None,
            ret_type: None,
        }
    }

    pub(crate) fn set_name(&mut self, nom: &str) -> &mut Self {
        self.name = Some(nom.to_string());
        self
    }

    pub(crate) fn add_arg(&mut self, arg: Arg) -> &mut Self {
        self.args.push(arg);
        self
    }

    pub(crate) fn set_ret_type(&mut self, ret_type: &str) -> &mut Self {
        self.ret_type = Some(ret_type.to_string());
        self
    }

    pub(crate) fn to_tokens(self) -> TokenStream {
        let stream = TokenStream::new();
        let args = Arg::complete_args(self.args.into_iter());

        stream
    }
}
