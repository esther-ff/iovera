use proc_macro::{Group, Ident, Literal, Punct, Span, TokenStream, TokenTree};

use super::lifetime::Lifetime;
use super::ty::Ty;
use crate::StructDef;

use std::marker::PhantomData;

type Result<T> = std::result::Result<T, (String, Span)>;
// enum Token2 {
//     Ident(Ident),
//     Group(Group),
//     Punct(Punct),
//     Literal(Literal),
// }

pub(crate) struct Stream {
    ptr: *mut TokenTree,
    len: usize,
    pos: usize,

    _boo: PhantomData<TokenTree>,
}

impl Stream {
    pub fn new(tokens: TokenStream) -> Self {
        let token_vec = tokens.into_iter().collect::<Vec<_>>();

        let len = token_vec
            .len()
            .checked_sub(1)
            .expect("an empty token stream was given");

        let ptr = Box::into_raw(token_vec.into_boxed_slice()) as *mut TokenTree;

        Self {
            ptr,
            len,
            pos: 0,
            _boo: PhantomData,
        }
    }

    pub fn end(&self) -> bool {
        self.pos > self.len
    }

    pub fn peek(&mut self) -> Option<TokenTree> {
        if self.end() {
            return None;
        } else if self.pos + 1 > self.len {
            return None;
        }

        unsafe { Some(self.ptr.add(1).read()) }
    }

    pub fn peek_steps(&mut self, steps: usize) -> Option<TokenTree> {
        if steps == 0 {
            return unsafe { Some(self.ptr.read()) };
        } else if self.end() {
            return None;
        } else if self.pos + steps > self.len {
            return None;
        };

        unsafe { Some(self.ptr.add(steps).read()) }
    }

    pub fn forward(&mut self) -> Option<TokenTree> {
        if self.end() {
            return None;
        };

        unsafe {
            let val = Some(self.ptr.add(self.pos).read());
            self.pos += 1;

            val
        }
    }

    pub fn forward_steps(&mut self, steps: usize) -> Option<TokenTree> {
        if steps == 0 {
            return unsafe { Some(self.ptr.read()) };
        } else if self.end() {
            return None;
        } else if self.pos + steps > self.len {
            return None;
        };

        unsafe {
            self.pos += steps;

            Some(self.ptr.add(self.pos).read())
        }
    }

    pub fn back_steps(&mut self, steps: usize) -> Option<TokenTree> {
        if steps == 0 {
            return unsafe { Some(self.ptr.read()) };
        } else if self.pos == 0 || self.pos == steps - 1 {
            return None;
        };

        unsafe {
            self.pos -= steps;

            Some(self.ptr.add(self.pos).read())
        }
    }

    pub fn back(&mut self) -> Option<TokenTree> {
        if self.pos == 0 {
            return None;
        } else {
            self.pos -= 1;
        }

        unsafe { Some(self.ptr.add(self.pos).read()) }
    }
}

impl std::ops::Drop for Stream {
    fn drop(&mut self) {
        drop(unsafe { Box::from_raw(self.ptr) })
    }
}

pub struct Parser {
    tkns: Stream,
    last_span: Option<Span>,
}

impl Parser {
    pub fn new(stream: TokenStream) -> Self {
        Self {
            tkns: Stream::new(stream),
            last_span: None,
        }
    }

    // pub fn last_token(&self) -> Option<&'p TokenTree> {
    //     self.last_token
    // }

    pub fn next(&mut self) -> Option<TokenTree> {
        let token = self.tkns.forward();
        self.last_span = Some(
            token
                .as_ref()
                .map_or_else(Span::call_site, |token| token.span()),
        );

        token
    }

    pub fn eof_next(&mut self) -> Result<TokenTree> {
        match self.tkns.forward() {
            None => {
                let str_err = format!("EOF while expecting more tokens");
                let span = Span::call_site();

                return Err((str_err, span));
            }

            Some(tkn) => Ok(tkn),
        }
    }

    pub fn back(&mut self) -> Option<TokenTree> {
        self.tkns.back()
    }

    pub fn next_st(&mut self, steps: usize) -> Option<TokenTree> {
        self.tkns.forward_steps(steps)
    }

    pub fn back_st(&mut self, steps: usize) -> Option<TokenTree> {
        self.tkns.back_steps(steps)
    }

    pub fn ident(&mut self) -> Result<Ident> {
        match self.eof_next()? {
            TokenTree::Ident(ident) => Ok(ident),

            TokenTree::Group(gr) => Err((format!("Expected an Ident, got a Group"), gr.span())),
            TokenTree::Punct(pt) => Err((format!("Expected an Ident, got a Punct"), pt.span())),
            TokenTree::Literal(lt) => Err((format!("Expected an Ident, got a Literal"), lt.span())),
        }
    }

    pub fn punct(&mut self) -> Result<Punct> {
        match self.eof_next()? {
            TokenTree::Ident(id) => Err((format!("Expected an Punct, got a Ident"), id.span())),

            TokenTree::Group(gr) => Err((format!("Expected an Ident, got a Group"), gr.span())),
            TokenTree::Punct(pt) => Ok(pt),
            TokenTree::Literal(lt) => Err((format!("Expected an Ident, got a Literal"), lt.span())),
        }
    }

    fn get_last_span(&self) -> Span {
        self.last_span.map_or_else(Span::call_site, |span| span)
    }

    pub fn test_spit(&mut self) {
        loop {
            let tkn = self.tkns.forward();

            if tkn.is_none() {
                break;
            } else {
                dbg!(tkn.unwrap());
            }
        }
    }

    pub fn parse_struct(&mut self) -> Result<StructDef> {
        // Get the struct's name and attributes
        let mut struct_attrs: Vec<proc_macro::Ident> = Vec::with_capacity(3);

        loop {
            let ident = self.ident()?;

            if ident.to_string() == "struct" {
                break;
            }

            struct_attrs.push(ident);
        }

        let struct_name = self.ident()?.to_string();

        use TokenTree::*;
        match self.eof_next()? {
            Group(gr) => {
                //
                let mut parser = Parser::new(gr.stream());

                loop {
                    let field_name = parser.ident()?;

                    match parser.punct()?.as_char() {
                        ch => {
                            let str_err = format!("Expected `:` got `{ch}``");
                            let span = self.get_last_span();

                            return Err((str_err, span));
                        }
                    }
                }
            }

            Punct(pc) => {
                //
                todo!()
            }

            Ident(id) => {
                //
                todo!()
            }

            _ => {
                let str_err = "Literals should not be here".to_string();
                let span = self.last_span.map_or_else(Span::call_site, |span| span);

                return Err((str_err, span));
            }
        }
    }
}

// --- HELPER FUNCTIONS --- //
