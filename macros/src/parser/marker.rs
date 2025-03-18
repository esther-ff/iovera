/// Identifiers preceeding some definition
/// like:
/// ```rust
/// extern "C" unsafe fn foo() {}
///
/// const STATIC_VAL: usize = 0;
///
/// unsafe { *(some_ptr) };
///
/// extern "syscall" struct AbiTest{};
///```
#[derive(Debug)]
pub enum Marker {
    Extern(&'static str),
    Pub,
    Const,
    Unsafe,
}

macro_rules! marker_to_tokens {
    ($marker:expr, $stream:expr) => {
        use Marker::*;
        use proc_macro::{Ident, Literal, Span, TokenTree};
        match $marker {
            Pub => {
                let ident = Ident::new("pub", Span::mixed_site());

                $stream.extend([TokenTree::Ident(ident)].into_iter())
            }

            Const => {
                let ident = Ident::new("const", Span::mixed_site());

                $stream.extend([TokenTree::Ident(ident)].into_iter())
            }

            Unsafe => {
                let ident = Ident::new("unsafe", Span::mixed_site());

                $stream.extend([TokenTree::Ident(ident)].into_iter())
            }

            Extern(abi) => {
                let ident = Ident::new("extern", Span::mixed_site());
                let literal = Literal::string(abi);

                $stream.extend([TokenTree::Ident(ident), TokenTree::Literal(literal)].into_iter())
            }
        }
    };
}

pub(crate) use marker_to_tokens;
