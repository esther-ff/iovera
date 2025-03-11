use proc_macro::{Ident, Punct, Span, TokenStream, TokenTree};

#[derive(Debug)]
pub(crate) struct Lifetime {
    name: String,
    apostrophe: Span,
}

impl Lifetime {
    pub(crate) fn new(name: String, apostrophe: Span) -> Lifetime {
        Self { name, apostrophe }
    }

    pub(crate) fn to_tkn_stream(&self) -> TokenStream {
        let mut tkns = TokenStream::new();

        let mut vec = Vec::with_capacity(3);

        let mark = Punct::new('\'', proc_macro::Spacing::Joint);
        vec.push(TokenTree::Punct(mark));

        let name = Ident::new(&self.name, Span::call_site());
        vec.push(TokenTree::Ident(name));

        tkns.extend(vec.into_iter());

        tkns
    }
}
