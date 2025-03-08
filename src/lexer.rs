/*
    Lexer for JSON!
*/

type Result<T> = std::result::Result<T, String>;
type Tokens<'ts> = Vec<Token<'ts>>;

#[derive(Debug)]
pub(crate) enum Token<'t> {
    LeftBracket(Span),
    RightBracket(Span),

    LeftSquareBracket(Span),
    RightSquareBracket(Span),

    Colon(Span),
    Comma(Span),

    String(&'t str, Span),
    Int(i64, Span),
    Float(f64, Span),
    Bool(bool, Span),

    Null(Span),
}

impl<'t> Token<'t> {
    fn span(&self) -> &Span {
        match self {
            &Self::LeftBracket(ref sp) => sp,
            &Self::RightBracket(ref sp) => sp,
            &Self::LeftSquareBracket(ref sp) => sp,
            &Self::RightSquareBracket(ref sp) => sp,
            &Self::Null(ref sp) => sp,
            &Self::Colon(ref sp) => sp,
            &Self::Comma(ref sp) => sp,

            &Self::String(_, ref sp) => sp,
            &Self::Int(_, ref sp) => sp,
            &Self::Float(_, ref sp) => sp,
            &Self::Bool(_, ref sp) => sp,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Span {
    start: usize,
    end: usize,
}

impl Span {
    fn new(start: usize, end: usize) -> Self {
        Span { start, end }
    }

    pub(crate) fn as_tuple(&self) -> (usize, usize) {
        (self.start, self.end)
    }
}

#[derive(Debug)]
pub(crate) struct Lexer<'l> {
    vec: Tokens<'l>,
    count: usize,
}

impl<'l> Lexer<'l> {
    pub fn new(input: &'l str) -> Result<Self> {
        let mut tokens = Vec::with_capacity(2048);

        let mut stream = input.char_indices().peekable();

        match input.chars().nth(0) {
            None => return Err("empty".to_string()),
            Some(ch) if ch != '{' => return Err(format!("invalid character at start: {ch}")),
            _ => {}
        }

        loop {
            let (index, ch) = match stream.next() {
                None => break,

                Some(ch) => ch,
            };

            let token = match ch {
                '{' => Token::LeftBracket(Span::new(index, index + 1)),
                '}' => Token::RightBracket(Span::new(index, index + 1)),
                '[' => Token::LeftSquareBracket(Span::new(index, index + 1)),
                ']' => Token::RightSquareBracket(Span::new(index, index + 1)),
                ':' => Token::Colon(Span::new(index, index + 1)),
                ',' => Token::Comma(Span::new(index, index + 1)),

                '"' => {
                    // Handling of strings

                    // loop till we find the ending quote
                    let start = index;
                    let mut end = index;
                    loop {
                        let next = match stream.next() {
                            None => {
                                return Err(format!("unexpected eof at char on pos: {end}"));
                            } // unexpected EOF.

                            Some((_, c)) => c,
                        };

                        end += 1;

                        if next == '"' {
                            break;
                        }
                    }

                    Token::String(&input[start + 1..end], Span::new(index, end))
                }

                ch if ch.is_ascii_digit() => {
                    let start = index;
                    let mut end = index;

                    let mut searching_for_float = false;

                    loop {
                        let next = match stream.next() {
                            None => return Err(format!("unexpected eof at char on pos: {index}")), // unexpected EOF.

                            Some((_, c)) => c,
                        };

                        end += 1;

                        match next {
                            '.' => searching_for_float = true,
                            ch if ch.is_ascii_digit() => {}

                            ',' => {
                                break;
                            }

                            _ => {
                                return Err(format!("invalid char passed as digit at span: {end}"));
                            } // invalid char
                        }
                    }

                    let token = if searching_for_float {
                        let parsed = &input[start..end].parse::<f64>();
                        let num = match parsed {
                            Err(_e) => {
                                return Err(format!(
                                    "failed to convert str to f64 at span ({start}, {end})"
                                ));
                            } // invalid value somehow?,

                            Ok(num) => num,
                        };

                        Token::Float(*num, Span::new(index, end))
                    } else {
                        let parsed = &input[start..end].parse::<i64>();
                        let num = match parsed {
                            Err(_e) => {
                                return Err(format!(
                                    "failed to convert str to i64 at span ({start}, {end})"
                                ));
                            } // invalid value somehow?

                            Ok(num) => num,
                        };

                        Token::Int(*num, Span::new(index, end))
                    };

                    token
                }

                ch if ch.is_ascii_alphabetic() => match ch {
                    't' => {
                        let val = &input[index..index + 4];
                        if val != "true" {
                            return Err(format!(
                                "invalid bool (expt: true) value: {val} at span: {index}, {}",
                                index + 2
                            ));
                        };

                        stream.nth(3);

                        Token::Bool(true, Span::new(index, index + 4))
                    }

                    'f' => {
                        let val = &input[index - 1..index + 4];
                        if val != "false" {
                            return Err(format!(
                                "invalid bool (expt: false) value: {val} at span: {index}, {}",
                                index + 2
                            ));
                        };

                        stream.nth(4);

                        Token::Bool(false, Span::new(index, index + 3))
                    }

                    'n' => {
                        let val = &input[index..index + 4];
                        if val == "null" {
                            stream.nth(3);
                            Token::Null(Span::new(index, index + 3))
                        } else {
                            return Err(format!(
                                "invalid bool value: {val} at span: {index}, {}",
                                index + 3
                            ));
                        }
                    }

                    _ => {
                        // In the name of diagnostics...
                        let start = index;
                        let mut end = index;

                        loop {
                            let (_index, ch) = match stream.next() {
                                None => return Err(format!("unexpected eof at char {index}")),

                                Some((index, ch)) => (index, ch),
                            };

                            end += 1;
                            if ch == ',' {
                                break;
                            }
                        }

                        let val = &input[start..end];
                        return Err(format!(
                            "invalid bool value: {val} at span: {index}, {}",
                            index + 2
                        ));
                    }
                },

                ch if ch.is_ascii_whitespace() => continue,

                _ => unreachable!(),
            };

            tokens.push(token);
        }

        tokens.reverse();
        let len = tokens.len();

        let lexer = Self {
            vec: tokens,
            count: len,
        };

        Ok(lexer)
    }

    pub(crate) fn peek(&self, peek: usize) -> Option<&Token> {
        if peek > self.count {
            return None;
        }

        self.vec.get(peek)
    }
}

impl<'l> Iterator for Lexer<'l> {
    type Item = Token<'l>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count == 0 {
            return None;
        }

        let item = self.vec.pop();

        self.count -= 1;

        item
    }
}
