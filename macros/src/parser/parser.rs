use proc_macro::{Group, Ident, Literal, Punct, TokenStream, TokenTree};

use super::lifetime::Lifetime;
use super::ty::Ty;

use std::marker::PhantomData;

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
}

impl Parser {
    pub fn new(stream: TokenStream) -> Self {
        Self {
            tkns: Stream::new(stream),
        }
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
}
