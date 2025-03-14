use proc_macro::{Ident, Punct, Span, TokenTree};

use crate::lifetime::Lifetime;

use super::fn_def::FnDef;
use crate::parser::parser::OptVec;

/// Represents a trait bound
/// like `<T: Iterator>`
#[derive(Debug)]
struct TraitBound {
    /// Name of the type being bounded
    ///```rust
    /// impl<T: Iterator> for Test<T> {}
    ///      ^
    ///```
    marker: Ident,

    /// Traits bounding the type
    traits: Option<Vec<Ident>>,

    /// Lifetimes specified in the bound
    /// `<T: Iterator + 'a>`
    ///                 ^^
    lifetime: Option<Vec<Lifetime>>,
}

macro_rules! intersperse_with_punct {
    ($iter:ident, $tokens:ident) => {{
        while let Some(val) = $iter.next() {
            $tokens.push(TokenTree::Ident(val));
            match $iter.next() {
                None => break,
                Some(mut nom) => {
                    let punct = Punct::new('+', proc_macro::Spacing::Alone);
                    $tokens.push(TokenTree::Punct(punct));

                    nom.set_span(Span::mixed_site());
                    $tokens.push(TokenTree::Ident(nom));
                }
            }
        }
    }};
}

impl TraitBound {
    pub fn new(name: Ident) -> Self {
        Self {
            marker: name,
            traits: None,
            lifetime: None,
        }
    }

    pub fn new_with(
        marker: Ident,
        traits: Option<Vec<Ident>>,
        lifetime: Option<Vec<Lifetime>>,
    ) -> Self {
        Self {
            marker,
            traits,
            lifetime,
        }
    }

    pub fn token_usize_hint(&self) -> usize {
        let mut base = 2;

        base += self.lifetime.as_ref().map_or_else(|| 0, |vec| vec.len());
        base += self.traits.as_ref().map_or_else(|| 0, |vec| vec.len() * 3);

        base
    }

    pub fn into_tokens(mut self) -> impl Iterator<Item = TokenTree> {
        let mut tokens = Vec::with_capacity(self.token_usize_hint());

        let mut marker = self.marker.clone();
        marker.set_span(Span::mixed_site());
        let name = TokenTree::Ident(marker);

        tokens.push(name);

        match self.traits.take() {
            None => {}
            Some(traits) => {
                let punct = Punct::new(':', proc_macro::Spacing::Alone);
                tokens.push(TokenTree::Punct(punct));

                let mut iter = traits.into_iter().map(|mut x| {
                    x.set_span(Span::mixed_site());
                    x
                });

                intersperse_with_punct!(iter, tokens);
            }
        }

        match self.lifetime.take() {
            None => {}
            Some(lfs) => {
                if tokens.len() > 2 {
                    let punct = Punct::new('+', proc_macro::Spacing::Alone);
                    tokens.push(TokenTree::Punct(punct));
                }

                let mut iter = lfs.into_iter();

                while let Some(lf) = iter.next() {
                    tokens.extend(lf.into_tokens());

                    match iter.next() {
                        None => break,
                        Some(another) => {
                            let punct = Punct::new('+', proc_macro::Spacing::Alone);
                            tokens.push(TokenTree::Punct(punct));

                            tokens.extend(another.into_tokens());
                        }
                    }
                }
            }
        }

        tokens.into_iter()
    }
}

/// Represents trait bounds
struct Properties {
    bounds: Option<Vec<TraitBound>>,
}

pub struct ImplBlockBuilder {
    fn_defs: OptVec<FnDef>,
    name: Option<Ident>,
    lside_props: OptVec<TraitBound>,
    is_unsafe: bool,
}

impl ImplBlockBuilder {
    pub fn name(&mut self, name: Ident) -> &mut Self {
        self.name = Some(name);
        self
    }

    pub fn function(&mut self, fn_def: FnDef) -> &mut Self {
        self.fn_defs.push(fn_def);
        self
    }

    pub fn bounds(&mut self, bound: TraitBound) -> &mut Self {
        self.lside_props.push(bound);
        self
    }

    pub fn is_unsafe(&mut self, safety: bool) -> &mut Self {
        self.is_unsafe = safety;
        self
    }
}

pub struct ImplBlock {
    /// Trait bounds and lifetimes on the left side of the `impl` block
    /// ```rust
    /// impl <'a, T: Trait> ... {}
    ///      ^^^^^^^^^^^^^^
    ///```
    left_side_properties: Properties,

    /// Thing we are binding this impl block to
    name: Ident,

    /// Function definitions inside the `impl` block
    function_defs: Option<Vec<FnDef>>,

    /// Describes if the impl is prefixed with `unsafe`
    /// ex: `unsafe impl Send for TestStruct {}`
    is_unsafe: bool,
}

impl ImplBlock {
    pub fn new() -> ImplBlockBuilder {
        ImplBlockBuilder {
            fn_defs: OptVec::new(0),
            name: None,
            lside_props: OptVec::new(0),
            is_unsafe: false,
        }
    }

    pub fn new_with_capacity(cap: usize) -> ImplBlockBuilder {
        ImplBlockBuilder {
            fn_defs: OptVec::new(cap),
            name: None,
            lside_props: OptVec::new(cap),
            is_unsafe: false,
        }
    }
}
