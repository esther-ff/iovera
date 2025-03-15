#![allow(dead_code)]
use proc_macro::{Group, Ident, Literal, Punct, Span, TokenStream, TokenTree};

use super::generics::Generic;
use super::lifetime::Lifetime;
use super::ty::{Borrow, ParsedType};
use crate::StructDef;
use crate::struct_def::Field;

use std::marker::PhantomData;

type Result<T> = std::result::Result<T, ParserError>;

pub(crate) struct Stream {
    ptr: *mut [TokenTree],
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

        let ptr = Box::into_raw(token_vec.into_boxed_slice());

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
        };
        // if self.pos + 1 > self.len {
        //     return None;
        // }

        unsafe {
            let val = (*self.ptr).get_unchecked(self.pos);
            Some(val.clone())
        }
    }

    pub fn peek_steps(&mut self, steps: usize) -> Option<TokenTree> {
        if steps == 0 {
            return unsafe {
                let val = (*self.ptr).get_unchecked(self.pos);

                Some(val.clone())
            };
        } else if self.end() {
            return None;
        } else if self.pos + steps > self.len {
            return None;
        };

        unsafe {
            let val = (*self.ptr).get_unchecked(self.pos + steps);
            Some(val.clone())
        }
    }

    pub fn forward(&mut self) -> Option<TokenTree> {
        if self.end() {
            return None;
        };

        unsafe {
            let val = (*self.ptr).get_unchecked(self.pos);
            self.pos += 1;

            Some(val.clone())
        }
    }

    pub fn forward_steps(&mut self, steps: usize) -> Option<TokenTree> {
        if steps == 0 {
            return unsafe {
                let val = (*self.ptr).get_unchecked(self.pos);

                Some(val.clone())
            };
        } else if self.end() {
            return None;
        } else if self.pos + steps > self.len {
            return None;
        };

        unsafe {
            self.pos += steps;

            let val = (*self.ptr).get_unchecked(self.pos);

            Some(val.clone())
        }
    }

    pub fn back_steps(&mut self, steps: usize) -> Option<TokenTree> {
        if steps == 0 {
            return unsafe {
                let val = (*self.ptr).get_unchecked(self.pos);

                Some(val.clone())
            };
        } else if self.pos == 0 || self.pos == steps - 1 {
            return None;
        };

        unsafe {
            self.pos -= steps;

            let val = (*self.ptr).get_unchecked(self.pos);

            Some(val.clone())
        }
    }

    pub fn back(&mut self) -> Option<TokenTree> {
        if self.pos == 0 {
            return None;
        } else {
            self.pos -= 1;
        }

        unsafe {
            let val = (*self.ptr).get_unchecked(self.pos);

            Some(val.clone())
        }
    }
}

impl std::ops::Drop for Stream {
    fn drop(&mut self) {
        drop(unsafe { Box::from_raw(self.ptr) })
    }
}

#[derive(Debug)]
pub enum ParserError {
    Eof(&'static str, Span),
    WrongToken(String, Span),
}

impl ParserError {
    /// Emits the error as an `TokenStream`
    /// the stream contains an invocation of the `compile_error!` macro
    /// which nicely reports errors.
    pub fn emit(self) -> TokenStream {
        match self {
            Self::Eof(payload, span) => Self::create_compile_error(span, payload),
            Self::WrongToken(payload, span) => Self::create_compile_error(span, &payload),
        }
    }

    fn create_compile_error(span: Span, txt: &str) -> TokenStream {
        let ident = TokenTree::Ident(Ident::new("compile_error", span));
        let punct = TokenTree::Punct(Punct::new('!', proc_macro::Spacing::Alone));

        let gr_stream = [TokenTree::Literal(Literal::string(txt))]
            .into_iter()
            .collect();
        let group = TokenTree::Group(Group::new(proc_macro::Delimiter::Parenthesis, gr_stream));

        [ident, punct, group].into_iter().collect()
    }
}

macro_rules! parse_error {
    (Eof, $self:ident) => {{
        dbg!("eof tracker");
        let err = ParserError::Eof("Unexpected EOF", $self.get_last_span());

        Err(err)
    }};

    (WrongToken, $span: expr, $corr: expr, $incor: expr) => {{
        let err =
            ParserError::WrongToken(format!("got `{}` but expected: {}", $incor, $corr), $span);
        Err(err)
    }};

    (WrongToken, $span: expr, $cor: expr) => {{
        let err = ParserError::WrongToken(format!("expected {}", $cor), $span);
        Err(err)
    }};
}

macro_rules! distinguish {
    (Ident, $match:ident) => {{
        dbg!(&$match);
        match $match {
            TokenTree::Ident(id) => id,
            t => return parse_error!(WrongToken, t.span(), "ident"),
        }
    }};

    (Punct, $match:ident) => {{
        match $match {
            TokenTree::Punct(pc) => pc,
            t => return parse_error!(WrongToken, t.span(), "punct"),
        }
    }};
}

macro_rules! eof_match {
    ($match:expr, $parser:ident) => {{
        match $match {
            None => return parse_error!(Eof, $parser),
            Some(val) => val,
        }
    }};
}

pub(crate) struct OptVec<T> {
    vec: Option<Vec<T>>,
    num: usize,
    is_ready: bool,
}

impl<T: std::fmt::Debug> OptVec<T> {
    pub(crate) fn new(num: usize) -> Self {
        Self {
            vec: None,
            num,
            is_ready: false,
        }
    }

    pub(crate) fn push(&mut self, item: T) {
        dbg!(&item);
        if self.is_ready {
            self.vec
                .as_mut()
                .expect("this vec should be here")
                .push(item);

            return;
        }

        let mut container = Vec::with_capacity(self.num);
        container.push(item);

        self.vec = Some(container);
    }

    pub(crate) fn into_self(mut self) -> Option<Vec<T>> {
        self.vec.take()
    }

    pub(crate) fn do_something<F, Type>(&self, default: Type, function: F) -> Type
    where
        F: FnOnce(&Vec<T>) -> Type,
    {
        match self.vec {
            Some(ref vec) => function(vec),
            None => default,
        }
    }

    pub(crate) fn do_take<F, Type>(&mut self, default: Type, function: F) -> Type
    where
        F: FnOnce(Vec<T>) -> Type,
    {
        match self.vec.take() {
            None => default,
            Some(vec) => function(vec),
        }
    }

    pub(crate) fn get_inside(&mut self) -> Option<Vec<T>> {
        self.vec.take()
    }
}

pub struct Parser {
    tkns: Stream,
    last_span: Option<Span>,
}

impl Parser {
    /// Creates a new `Parser`.
    pub fn new(stream: TokenStream) -> Self {
        Self {
            tkns: Stream::new(stream),
            last_span: None,
        }
    }

    /// Skips one element
    /// equivalent to calling a `Parser`'s `next` method and
    /// discarding the result.
    pub fn skip(&mut self) {
        self.next();
    }

    /// Peeks one element forward.
    /// returns `None` if there are no more elements
    /// otherwise returns `Some` containing a `TokenTree`
    pub fn peek(&mut self) -> Option<TokenTree> {
        self.tkns.peek()
    }

    /// Moves one element forward
    /// returns `None` if there are no more elements
    /// otherwise returns `Some` containing a `TokenTree`
    pub fn next(&mut self) -> Option<TokenTree> {
        let token = self.tkns.forward();
        self.last_span = Some(
            token
                .as_ref()
                .map_or_else(Span::call_site, |token| token.span()),
        );

        token
    }

    /// Moves one element forward
    /// instead of giving us an `Option`
    /// it returns a `Result<TokenTree, ParseError>`
    pub fn eof_next(&mut self) -> Result<TokenTree> {
        match self.tkns.forward() {
            None => return parse_error!(Eof, self),

            Some(tkn) => Ok(tkn),
        }
    }

    /// Goes back one token.
    pub fn back(&mut self) -> Option<TokenTree> {
        self.tkns.back()
    }

    /// Goes forward `steps` tokens
    pub fn next_st(&mut self, steps: usize) -> Option<TokenTree> {
        self.tkns.forward_steps(steps)
    }

    /// Goes back `steps` tokens
    pub fn back_st(&mut self, steps: usize) -> Option<TokenTree> {
        self.tkns.back_steps(steps)
    }

    /// Asserts the result is an `Ident`, else returns `Err`
    pub fn ident(&mut self) -> Result<Ident> {
        let err = match self.eof_next()? {
            TokenTree::Ident(ident) => return Ok(ident),

            TokenTree::Group(gr) => {
                parse_error!(WrongToken, gr.span(), "Ident", gr.to_string())
            }

            TokenTree::Punct(pt) => {
                parse_error!(WrongToken, pt.span(), "Ident", pt.as_char())
            }

            TokenTree::Literal(lt) => {
                parse_error!(WrongToken, lt.span(), "Ident", lt.to_string())
            }
        };

        err
    }

    /// Asserts the result is a `Punct`, else returns `Err`
    pub fn punct(&mut self) -> Result<Punct> {
        let err = match self.eof_next()? {
            TokenTree::Punct(pc) => return Ok(pc),

            TokenTree::Group(gr) => {
                parse_error!(WrongToken, gr.span(), "Punct", gr.to_string())
            }

            TokenTree::Ident(id) => {
                parse_error!(WrongToken, id.span(), "Punct", id.to_string())
            }

            TokenTree::Literal(lt) => {
                parse_error!(WrongToken, lt.span(), "Punct", lt.to_string())
            }
        };

        err
    }

    /// Last span of item obtained.
    fn get_last_span(&self) -> Span {
        self.last_span.map_or_else(Span::call_site, |span| span)
    }

    /// Parses a struct.
    /// Returns an error once one occurs.
    pub fn parse_struct(&mut self) -> Result<StructDef> {
        let mut struct_attrs: Vec<proc_macro::Ident> = Vec::with_capacity(3);

        loop {
            let ident = self.ident()?;

            if ident.to_string() == "struct" {
                break;
            }

            struct_attrs.push(ident);
        }

        let struct_name = self.ident()?.to_string();

        let mut struct_fields: Vec<Field> = Vec::with_capacity(4);
        let mut struct_generics: Option<Vec<Generic>> = None;
        let mut struct_lifetimes: Option<Vec<Lifetime>> = None;

        use TokenTree::*;
        'vabank: loop {
            match self.eof_next()? {
                Group(gr) => {
                    let mut parser = Parser::new(gr.stream());

                    loop {
                        if parser.tkns.end() {
                            break;
                        }

                        let field_name = parser.ident()?;

                        match parser.punct()?.as_char() {
                            ':' => {
                                if parser.peek().is_none() {
                                    return parse_error!(Eof, parser);
                                }
                            }
                            ch => {
                                return parse_error!(WrongToken, self.get_last_span(), ":", ch);
                            }
                        }

                        match parser.next().expect("this should not be `None`") {
                            Ident(type_name) => {
                                // Just the type name
                                let field = match parser.punct()?.as_char() {
                                    ',' => {
                                        let field_type =
                                            ParsedType::new(None, type_name, None, None);
                                        Field::new(field_type, field_name.to_string())
                                    }

                                    '<' => {
                                        let (gens, lfs) = dig_up_generics_lifetimes(&mut parser)?;

                                        let field_type =
                                            ParsedType::new(None, type_name, gens, lfs);

                                        // After the `dig_up_generics_lifetimes` function is called
                                        // the cursor will be on the character after `>`
                                        // It should be `,` so we can just skip it
                                        dbg!(parser.next());

                                        Field::new(field_type, field_name.to_string())
                                    }

                                    ch => {
                                        return parse_error!(
                                            WrongToken,
                                            parser.get_last_span(),
                                            "`<` or `,`",
                                            ch
                                        );
                                    }
                                };

                                struct_fields.push(field)
                            }

                            Punct(pt) if pt.as_char() == '&' => {
                                // `next_st` faulty
                                // also additional check for correctness
                                parser.skip();
                                let lifetime_name = match parser.next() {
                                    None => return parse_error!(Eof, parser),

                                    Some(name) => {
                                        let tkn = distinguish!(Ident, name);

                                        Some(tkn)
                                    }
                                };

                                let is_mutable =
                                    eof_match!(parser.peek(), parser).to_string() == "mut";
                                let type_name = if is_mutable {
                                    // `next_st` might be faulty
                                    // test it out!
                                    parser.skip();
                                    match eof_match!(parser.next(), parser) {
                                        Ident(name) => dbg!(name),
                                        any => {
                                            return parse_error!(
                                                WrongToken,
                                                self.get_last_span(),
                                                "Ident",
                                                any
                                            );
                                        }
                                    }
                                } else {
                                    dbg!(parser.ident())?
                                };

                                let field = create_field(
                                    &mut parser,
                                    is_mutable,
                                    lifetime_name,
                                    type_name,
                                    field_name,
                                )?;

                                struct_fields.push(field);
                            }

                            ch => {
                                return parse_error!(
                                    WrongToken,
                                    ch.span(),
                                    "identifier or punct",
                                    ch
                                );
                            }
                        };
                    }

                    // Ends the entire loop
                    // We already parsed the struct's group
                    break 'vabank;
                }

                // Here we check if the punct is a `<`
                // this denotes type lifetimes, generics, etc...
                Punct(pc) if pc.as_char() == '<' => {
                    let (gens, lfs) = dig_up_generics_lifetimes(self)?;

                    struct_generics = gens;
                    struct_lifetimes = lfs;
                }
                das => {
                    return parse_error!(WrongToken, self.get_last_span(), "a `<` or a Group", das);
                }
            };
        }

        let struct_def = StructDef::new(
            None,
            struct_fields,
            struct_name,
            struct_generics,
            struct_lifetimes,
        );
        Ok(struct_def)
    }
}

// --- HELPER FUNCTIONS --- //

// Requires that the current parser's position be 1 after the detected `<`
fn dig_up_generics_lifetimes(
    parser: &mut Parser,
) -> Result<(Option<Vec<Generic>>, Option<Vec<Lifetime>>)> {
    let mut generics: OptVec<Generic> = OptVec::new(4);
    let mut lifetimes: OptVec<Lifetime> = OptVec::new(4);

    loop {
        match parser.peek() {
            None => return parse_error!(Eof, parser),
            Some(tkn) => match tkn {
                TokenTree::Group(_) => break,
                _ => {}
            },
        }

        let tkn = parser.eof_next()?;

        match tkn {
            // checking for the `'` character
            // used in lifetimes, like `&'a`
            TokenTree::Punct(pc) if pc.as_char() == '\'' => {
                let lifetime_name = parser.ident()?;

                let lf = Lifetime::new(lifetime_name.to_string(), pc.span());
                lifetimes.push(lf);

                // for correctness
                let tkn = match parser.peek() {
                    None => return parse_error!(Eof, parser),
                    Some(tkn) => tkn,
                };

                match tkn {
                    TokenTree::Punct(pc) if pc.as_char() == '>' => break,
                    TokenTree::Punct(pc) if pc.as_char() == ',' => {
                        parser.next();
                    }

                    ch => {
                        return parse_error!(WrongToken, ch.span(), "a `>` or `,`", ch.to_string());
                    }
                }
            }

            TokenTree::Punct(pc) if pc.as_char() == '>' => break,

            TokenTree::Ident(id) => {
                let next_token = match parser.peek() {
                    None => return parse_error!(Eof, parser),
                    Some(tkn) => distinguish!(Punct, tkn),
                };

                if next_token.as_char() == ':' {
                    parser.skip();
                    let mut trait_bounds: Vec<Ident> = Vec::with_capacity(4);

                    loop {
                        let tkn = parser.eof_next()?;
                        let ident = distinguish!(Ident, tkn);
                        trait_bounds.push(ident);

                        let next_token = match parser.peek() {
                            None => return parse_error!(Eof, parser),
                            Some(tkn) => tkn,
                        };

                        match next_token {
                            TokenTree::Punct(pc) => match pc.as_char() {
                                '+' => {
                                    parser.skip();
                                }

                                ',' => {
                                    parser.skip();
                                    //break;
                                }

                                '>' => {
                                    parser.skip();

                                    break;
                                }

                                ch => {
                                    return parse_error!(WrongToken, pc.span(), "`+` or `,`", ch);
                                }
                            },

                            tkn => {
                                return parse_error!(WrongToken, tkn.span(), "`+` or `,`", tkn);
                            }
                        }
                    }

                    generics.push(Generic::new(Some(trait_bounds), id))
                } else {
                    generics.push(Generic::new(None, id))
                }
            }

            TokenTree::Punct(pc) if pc.as_char() == ',' => {}

            tkn => {
                return parse_error!(WrongToken, tkn.span(), "punct or ident", tkn);
            }
        }
    }
    Ok((generics.into_self(), lifetimes.into_self()))
}

// Creates a field from a borrowed parser and variables
fn create_field(
    parser: &mut Parser,
    is_mutable: bool,
    lf_name: Option<Ident>,
    type_name: Ident,
    field_name: Ident,
) -> Result<Field> {
    // Checks if we have a `<`
    // this can be the start of a sequence of generic types, lifetimes, etc...
    // like `Test<'a, T, I, 'b`.
    let has_extra_markers = match parser.peek() {
        None => return parse_error!(Eof, parser),
        Some(val) => {
            parser.skip();
            val.to_string() == "<"
        }
    };

    let borrow = match lf_name {
        None => None,
        Some(name) => {
            let borrow = Borrow::new(is_mutable, name.to_string());
            Some(borrow)
        }
    };

    let ty = if has_extra_markers {
        // We'll check for additional lifetimes, marks on the type itself
        // like `Test<'a, T>`
        let (gens, lfs) = dig_up_generics_lifetimes(parser)?;
        ParsedType::new(borrow, type_name, gens, lfs)
    } else {
        ParsedType::new(borrow, type_name, None, None)
    };

    Ok(Field::new(ty, field_name.to_string()))
}
