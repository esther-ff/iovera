use proc_macro::{Ident, Punct, Span};

use crate::TokenStream;
use crate::extra_iter::Extra;
use crate::proc_macro::TokenTree;

#[derive(Debug)]
struct Arg {
    name: String,
    arg_type: String,
}

impl Arg {
    fn make_to_iter(&self) -> impl Iterator<Item = TokenTree> {
        let arg_name = TokenTree::Ident(Ident::new(&self.name, Span::call_site()));
        let delim = TokenTree::Punct(Punct::new(':', proc_macro::Spacing::Alone));
        let arg_type = TokenTree::Ident(Ident::new(&self.arg_type, Span::call_site()));

        [arg_name, delim, arg_type].into_iter()
    }

    fn complete_args(args: impl Iterator<Item = Arg>) -> Extra<impl Iterator<Item = TokenTree>> {
        fn gen_ident() -> TokenTree {
            TokenTree::Punct(Punct::new(',', proc_macro::Spacing::Alone))
        }

        let mut tokens = Vec::new();

        args.for_each(|arg| arg.make_to_iter().for_each(|item| tokens.push(item)));

        let iter = Extra::new(tokens.into_iter(), 4, gen_ident);
        //
        //let tokens = match iter.size_hint() {
        //    (0, None) => Vec::with_capacity(32),
        //    (0, Some(num)) => Vec::with_capacity(num)
        //};

        iter
    }
}

#[derive(Debug)]
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

    pub(crate) fn build(self) -> TokenStream {
        let mut stream = TokenStream::new();
        let args = Arg::complete_args(self.args.into_iter());

        stream
    }
}
