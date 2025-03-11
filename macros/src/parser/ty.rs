use proc_macro::{Ident, Span};

use super::{generics::Generic, lifetime::Lifetime};

pub(crate) struct Ty {
    span: (Span, Span),

    borrow: Option<i32>,

    base: Ident,

    generic: Generic,

    lifetime: Lifetime,
}
