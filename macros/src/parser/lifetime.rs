use proc_macro::{Ident, Punct, Span, TokenTree};

#[derive(Debug)]
pub(crate) struct Lifetime {
    name: String,
    apostrophe: Span,
}

impl Lifetime {
    pub(crate) fn new(name: String, apostrophe: Span) -> Lifetime {
        Self { name, apostrophe }
    }

    pub(crate) fn into_tokens(&self) -> impl Iterator<Item = TokenTree> {
        let mark = Punct::new('\'', proc_macro::Spacing::Joint);
        let name = Ident::new(&self.name, Span::mixed_site());

        [TokenTree::Punct(mark), TokenTree::Ident(name)].into_iter()
    }
}
