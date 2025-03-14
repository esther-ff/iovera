use proc_macro::Ident;

use super::{generics::Generic, lifetime::Lifetime};

#[derive(Debug)]
pub(crate) struct ParsedType {
    //span: (Span, Span),
    borrow: Option<Borrow>,

    base: Ident,

    generic: Option<Vec<Generic>>,

    lifetime: Option<Vec<Lifetime>>,
}

impl ParsedType {
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
}

#[derive(Debug)]
pub(crate) struct Borrow {
    mutable: bool,
    lifetime_name: String,
}

impl Borrow {
    pub(crate) fn new(mutable: bool, lifetime_name: String) -> Self {
        Self {
            mutable,
            lifetime_name,
        }
    }
}
