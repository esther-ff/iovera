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
        dbg!((self.len, self.pos));
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
        dbg!(self.len);
        dbg!(self.pos);
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

    pub fn skip(&mut self) {
        self.next();
    }

    pub fn peek(&mut self) -> Option<TokenTree> {
        self.tkns.peek()
    }

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
            None => return parse_error!(Eof, self),

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
        let aaa = dbg!(self.eof_next()?);
        let err = match aaa {
            TokenTree::Ident(ident) => return Ok(ident),

            TokenTree::Group(gr) => {
                parse_error!(WrongToken, gr.span(), "Ident", "Group")
            }

            TokenTree::Punct(pt) => {
                parse_error!(WrongToken, pt.span(), "Ident", "Punct")
            }

            TokenTree::Literal(lt) => {
                parse_error!(WrongToken, lt.span(), "Ident", "Literal")
            }
        };

        err
    }

    pub fn punct(&mut self) -> Result<Punct> {
        let err = match self.eof_next()? {
            TokenTree::Punct(pc) => return Ok(pc),

            TokenTree::Group(gr) => {
                parse_error!(WrongToken, gr.span(), "Punct", "Group")
            }

            TokenTree::Ident(id) => {
                parse_error!(WrongToken, id.span(), "Punct", "Group")
            }

            TokenTree::Literal(lt) => {
                parse_error!(WrongToken, lt.span(), "Punct", "Literal")
            }
        };

        err
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

        let mut struct_fields: Vec<Field> = Vec::with_capacity(4);
        let mut struct_generics: Option<Vec<Generic>> = None;
        let mut struct_lifetimes: Option<Vec<Lifetime>> = None;

        use TokenTree::*;
        'vabank: loop {
            dbg!(&struct_fields);
            match self.eof_next()? {
                Group(gr) => {
                    //
                    let mut parser = Parser::new(gr.stream());

                    loop {
                        let field_name = parser.ident()?;

                        match parser.punct()?.as_char() {
                            ':' => {}
                            ch => {
                                return parse_error!(WrongToken, self.get_last_span(), ":", ch);
                            }
                        }

                        let mut tokens = Vec::with_capacity(64);

                        if parser.tkns.end() {
                            break;
                        }

                        match parser.next().expect("this should not be `None`") {
                            Ident(id) => {
                                // Just the type name
                                tokens.push(TokenTree::Ident(id));
                            }

                            Punct(pt) if pt.as_char() == '&' => {
                                let lifetime_name = match parser.next_st(2) {
                                    None => return parse_error!(Eof, self),

                                    Some(name) => distinguish!(Ident, name),
                                };

                                let is_mutable = match parser.peek() {
                                    None => return parse_error!(Eof, self),

                                    Some(tkn) => tkn.to_string() == "mut",
                                };

                                let type_name = if is_mutable {
                                    match parser.next_st(2) {
                                        None => return parse_error!(Eof, self),

                                        Some(token) => match token {
                                            Ident(name) => name,
                                            _ => {
                                                return parse_error!(
                                                    WrongToken,
                                                    self.get_last_span(),
                                                    "Ident",
                                                    "nothing good"
                                                );
                                            }
                                        },
                                    }
                                } else {
                                    parser.ident()?
                                };

                                let field = create_field(
                                    self,
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

                    break 'vabank;
                }

                // Here we check if the punct is a `<`
                // this denotes type lifetimes, generics, etc...
                Punct(pc) if pc.as_char() == '<' => {
                    let (gens, lfs) = dig_up_generics_lifetimes(self)?;

                    struct_generics = Some(gens);
                    struct_lifetimes = Some(lfs);
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
fn dig_up_generics_lifetimes(parser: &mut Parser) -> Result<(Vec<Generic>, Vec<Lifetime>)> {
    let mut generics: Vec<Generic> = Vec::with_capacity(4);
    let mut lifetimes: Vec<Lifetime> = Vec::with_capacity(4);

    loop {
        dbg!(parser.peek());
        let tkn = parser.eof_next()?;

        dbg!(&tkn);
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
                        //println!("Are we here?");
                        let ident = distinguish!(Ident, tkn);
                        //dbg!(&ident);
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

    Ok((generics, lifetimes))
}

// Creates a field from a borrowed parser and variables
fn create_field(
    parser: &mut Parser,
    is_mutable: bool,
    lf_name: Ident,
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

    let borrow = Borrow::new(is_mutable, lf_name.to_string());
    let ty: ParsedType;

    if has_extra_markers {
        // We'll check for additional lifetimes, marks on the type itself
        // like `Test<'a, T>`
        let (gens, lfs) = dig_up_generics_lifetimes(parser)?;
        ty = ParsedType::new(Some(borrow), type_name, Some(gens), Some(lfs));
    } else {
        ty = ParsedType::new(Some(borrow), type_name, None, None);
    }

    Ok(Field::new(ty, field_name.to_string()))
}
