use proc_macro::Ident;

#[derive(Debug)]
pub(crate) struct Generic {
    traits: Option<Vec<Ident>>,
    name: Ident,
}

impl Generic {
    pub(crate) fn new(traits: Option<Vec<Ident>>, name: Ident) -> Self {
        Self { traits, name }
    }

    pub(crate) fn insert(&mut self, tr: Ident) {
        match self.traits {
            Some(ref mut vec) => vec.push(tr),
            None => {
                self.traits = Some(Vec::with_capacity(4));
                self.traits.as_mut().unwrap().push(tr)
            }
        }
    }

    pub(crate) fn as_stream() {}
}
