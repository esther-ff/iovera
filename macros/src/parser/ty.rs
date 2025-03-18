use proc_macro::{Ident, Punct, Spacing, Span, TokenTree};

use super::{generics::Generic, lifetime::Lifetime};
use crate::iters::interweave::Separator;
use std::iter;

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

    pub(crate) fn to_tokens(mut self) -> impl IntoIterator<Item = TokenTree> {
        let mut scratch = Vec::with_capacity(256);
        scratch.push(TokenTree::Ident(self.base.clone()));

        match self.borrow.take() {
            None => {}
            Some(borrow) => scratch.extend(borrow.to_tokens()),
        }

        match self.generics_to_tokens() {
            None => {}
            Some(spec) => {
                scratch.push(TokenTree::Punct(Punct::new('<', Spacing::Alone)));
                scratch.extend(spec);
                scratch.push(TokenTree::Punct(Punct::new('>', Spacing::Alone)));
            }
        }

        scratch.into_iter()
    }

    fn generics_to_tokens(&mut self) -> Option<Vec<TokenTree>> {
        let sep = Punct::new(',', Spacing::Alone);

        match (self.generic.take(), self.lifetime.take()) {
            (Some(generics), Some(lifetimes)) => {
                let mut scratch: Vec<TokenTree> = Vec::with_capacity(256);

                let sep = Punct::new(',', Spacing::Alone);

                for generic in generics {
                    scratch.push(generic.as_token());
                    scratch.push(TokenTree::Punct(sep.clone()))
                }

                for lifetime in lifetimes {
                    scratch.extend(lifetime.into_tokens());
                    scratch.push(TokenTree::Punct(sep.clone()));
                }

                scratch.pop();

                Some(scratch)
            }
            (Some(g), None) => {
                let iter = g.into_iter().map(|x| x.as_token());

                let sep_iter = Separator::new(iter, TokenTree::Punct(sep.clone())).collect();

                Some(sep_iter)
            }
            (None, Some(lifetimes)) => {
                let mut scratch: Vec<TokenTree> = Vec::with_capacity(256);

                for lifetime in lifetimes {
                    scratch.extend(lifetime.into_tokens());
                    scratch.push(TokenTree::Punct(sep.clone()));
                }

                scratch.pop();

                Some(scratch)
            }

            _ => None,
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

    pub fn to_tokens(self) -> impl Iterator<Item = TokenTree> {
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
