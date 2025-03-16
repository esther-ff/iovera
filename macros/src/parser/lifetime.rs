use proc_macro::{Ident, Punct, Span, TokenTree};

use super::ty::Borrow;

#[derive(Debug)]
pub struct Lifetime {
    name: String,
    apostrophe: Span,
}

impl Lifetime {
    pub fn new(name: String, apostrophe: Span) -> Lifetime {
        Self { name, apostrophe }
    }

    pub fn into_tokens(self) -> LifetimeIter {
        let mark = Punct::new('\'', proc_macro::Spacing::Joint);
        let name = Ident::new(&self.name, Span::mixed_site());

        let iterable = [Some(TokenTree::Punct(mark)), Some(TokenTree::Ident(name))];
        LifetimeIter { iterable, slot: 0 }
    }

    pub fn into_borrow(self) -> Borrow {
        Borrow::new(false, self)
    }

    pub fn into_borrow_mut(self) -> Borrow {
        Borrow::new(true, self)
    }
}

struct LifetimeIter {
    iterable: [Option<TokenTree>; 2],
    slot: usize,
}

impl Iterator for LifetimeIter {
    type Item = TokenTree;

    fn next(&mut self) -> Option<TokenTree> {
        if self.slot == 2 {
            return None;
        }

        unsafe { self.iterable.get_unchecked_mut(self.slot).take() }
    }
}

impl IntoIterator for Lifetime {
    type Item = TokenTree;
    type IntoIter = LifetimeIter;

    fn into_iter(self) -> Self::IntoIter {
        self.into_tokens()
    }
}
