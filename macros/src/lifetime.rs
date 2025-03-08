use proc_macro::{Ident, Punct, Span, TokenStream, TokenTree};

#[derive(Debug)]
pub(crate) struct Lifetime {
    is_mut: bool,
    name: String,
}

impl Lifetime {
    pub(crate) fn new(is_mut: bool, name: String) -> Lifetime {
        Self { is_mut, name }
    }

    pub(crate) fn to_tkn_stream(&self) -> TokenStream {
        let mut tkns = TokenStream::new();

        let mut vec = Vec::with_capacity(3);

        let mark = Punct::new('\'', proc_macro::Spacing::Joint);
        vec.push(TokenTree::Punct(mark));

        if self.is_mut {
            let mut_keyword = Ident::new("mut", Span::call_site());
            vec.push(TokenTree::Ident(mut_keyword));
        };

        let name = Ident::new(&self.name, Span::call_site());
        vec.push(TokenTree::Ident(name));

        tkns.extend(vec.into_iter());

        tkns
    }

    pub(crate) fn is_mut(&self) -> bool {
        self.is_mut
    }
}
