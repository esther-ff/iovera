use proc_macro::Ident;

#[derive(Debug)]
pub(crate) struct Generic {
    pub traits: Vec<Ident>,
}
