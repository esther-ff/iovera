use std::iter;

use proc_macro::{Ident, Punct, Span, TokenTree};

use super::{generics::Generic, lifetime::Lifetime};

#[derive(Debug)]
pub(crate) struct Ty {
    //span: (Span, Span),
    borrow: Option<Borrow>,

    base: Ident,

    generic: Option<Vec<Generic>>,

    lifetime: Option<Vec<Lifetime>>,
}

impl Ty {
    pub(crate) fn new(
        borrow: Option<Borrow>,
        base: Ident,
        generic: Option<Vec<Generic>>,
        lifetime: Option<Vec<Lifetime>>,
    ) -> Self {
        Self {
            borrow,
            base,
            generic,
            lifetime,
        }
    }

    pub(crate) fn to_tokens(self) -> impl IntoIterator<Item = TokenTree> {
        let base = iter::once(TokenTree::Ident(self.base));

        match self.borrow.take() {
            None => {
                todo!()
            }
            Some(borrow) => {
                let iter =
                    borrow
                        .to_tokens()
                        .into_iter()
                        .chain(base)
                        .chain(iter::once(TokenTree::Punct(Punct::new(
                            '<',
                            proc_macro::Spacing::Alone,
                        ))));

                match (self.generic.take(), self.lifetime.take()) {
                    (Some(generic), Some(lifetime)) => {
                        // interweave some `,`
                        let generic_iter =
                            generic.into_iter().flat_map(|generic| generic.to_tokens());

                        let lifetime_iter = lifetime
                            .into_iter()
                            .flat_map(|lifetime| lifetime.into_borrow().to_tokens());

                        return iter;
                    }
                    (Some(generic), None) => {
                        // interweave some `,`
                        let generic_iter =
                            generic.into_iter().flat_map(|generic| generic.to_tokens());

                        return iter;
                    }
                    (None, Some(lifetime)) => {
                        let lifetime_iter = lifetime
                            .into_iter()
                            .flat_map(|lifetime| lifetime.into_borrow().to_tokens());

                        return iter;
                    }
                    (None, None) => {
                        return iter.map(|token| {
                            token.set_span(Span::mixed_site());
                            token
                        });
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Borrow {
    mutable: bool,
    lifetime: Lifetime,
}

impl Borrow {
    pub fn new(mutable: bool, lifetime: Lifetime) -> Self {
        Self { mutable, lifetime }
    }

    pub fn to_tokens(self) -> impl IntoIterator<Item = TokenTree> {
        BorrowIter {
            mutable: self.mutable,
            iter: self.lifetime.into_tokens(),
            insert_now: false,
        }
        .map(|mut token| {
            token.set_span(Span::mixed_site());
            token
        })
    }
}

struct BorrowIter<I> {
    mutable: bool,
    iter: I,
    insert_now: bool,
}

impl<I> Iterator for BorrowIter<I>
where
    I: Iterator<Item = TokenTree>,
{
    type Item = TokenTree;

    fn next(&mut self) -> Option<Self::Item> {
        if self.mutable {
            self.insert_now = true;
            return self.iter.next();
        }

        if self.insert_now {
            let mut_ident = Ident::new("mut", Span::mixed_site());

            self.insert_now = false;

            return Some(TokenTree::Ident(mut_ident));
        }

        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(2 + self.mutable as usize))
    }
}
