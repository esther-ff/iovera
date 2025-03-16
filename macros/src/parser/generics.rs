use proc_macro::{Ident, TokenTree};

#[derive(Debug)]
pub struct Generic {
    traits: Option<Vec<Ident>>,
    name: Ident,
}

impl Generic {
    pub fn new(traits: Option<Vec<Ident>>, name: Ident) -> Self {
        Self { traits, name }
    }

    pub fn insert(&mut self, tr: Ident) {
        match self.traits {
            Some(ref mut vec) => vec.push(tr),
            None => {
                self.traits = Some(Vec::with_capacity(4));
                self.traits.as_mut().unwrap().push(tr)
            }
        }
    }

    pub fn to_tokens(self) -> impl IntoIterator<Item = TokenTree> {
        std::iter::once(TokenTree::Ident(self.name))
    }
}
