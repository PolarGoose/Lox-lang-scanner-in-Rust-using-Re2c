use anyhow::Result;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
#[allow(non_camel_case_types, dead_code)]
pub enum TokenType<'a> {
    // Single-character tokens.
    LEFT_PAREN,
    RIGHT_PAREN,
    LEFT_BRACE,
    RIGHT_BRACE,
    COMMA,
    DOT,
    MINUS,
    PLUS,
    SEMICOLON,
    SLASH,
    STAR,

    // One or two character tokens.
    BANG,
    BANG_EQUAL,
    EQUAL,
    EQUAL_EQUAL,
    GREATER,
    GREATER_EQUAL,
    LESS,
    LESS_EQUAL,

    // Literals.
    IDENTIFIER(&'a str),
    STRING(&'a str),
    NUMBER(f64),

    // Keywords.
    AND,
    CLASS,
    ELSE,
    FALSE,
    FUN,
    FOR,
    IF,
    NIL,
    OR,
    PRINT,
    RETURN,
    SUPER,
    THIS,
    TRUE,
    VAR,
    WHILE,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Token<'a> {
    pub token_type: TokenType<'a>,
    pub line_number: usize,
    pub line_start_index: usize,
    pub start_index_within_input: usize,
    pub end_index_within_input: usize,
    pub start_index_within_line: usize,
    pub end_index_within_line: usize,
}

#[derive(Debug, Error)]
#[error("Unexpected symbol at {line_number}:{error_index_within_line}")]
pub struct UnexpectedSymbolError {
    pub line_number: usize,
    pub line_start_index_within_input: usize,
    pub error_index_within_line: usize,
    pub error_index_within_input: usize,
}

pub struct Scanner<'a> {
    s: &'a [u8],
    cursor: usize,
    #[allow(dead_code)]
    mark: usize,
    #[allow(dead_code)]
    ctxmarker: usize,
    current_line_number: usize,
    current_line_start_index: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            s: input.as_bytes(),
            cursor: 0,
            mark: 0,
            ctxmarker: 0,
            current_line_number: 0,
            current_line_start_index: 0,
        }
    }

    fn create_token(
        &mut self,
        token_type: TokenType<'a>,
        beginning_of_token: usize,
        end_of_token: usize,
    ) -> Option<Result<Token<'a>>> {
        Some(Ok(Token {
            token_type,
            line_number: self.current_line_number,
            line_start_index: self.current_line_start_index,
            start_index_within_input: beginning_of_token,
            end_index_within_input: end_of_token,
            start_index_within_line: beginning_of_token - self.current_line_start_index,
            end_index_within_line: end_of_token - self.current_line_start_index,
        }))
    }

    fn create_number_token(
        &mut self,
        beginning_of_token: usize,
        end_of_token: usize,
    ) -> Option<Result<Token<'a>>> {
        let s = std::str::from_utf8(&self.s[beginning_of_token..end_of_token]).unwrap();
        self.create_token(
            TokenType::NUMBER(s.parse::<f64>().unwrap()),
            beginning_of_token,
            end_of_token,
        )
    }

    fn create_string_token(
        &mut self,
        beginning_of_token: usize,
        end_of_token: usize,
    ) -> Option<Result<Token<'a>>> {
        let s = std::str::from_utf8(&self.s[beginning_of_token..end_of_token]).unwrap();
        self.create_token(TokenType::STRING(s), beginning_of_token, end_of_token)
    }

    fn create_identifier(&mut self, beg: usize, end: usize) -> Option<Result<Token<'a>>> {
        let s = std::str::from_utf8(&self.s[beg..end]).unwrap();
        self.create_token(TokenType::IDENTIFIER(s), beg, end)
    }

    fn create_unexpected_symbol_error(
        &self,
        error_index_within_input: usize,
    ) -> Option<Result<Token<'a>>> {
        Some(Err(anyhow::Error::new(UnexpectedSymbolError {
            line_number: self.current_line_number,
            line_start_index_within_input: self.current_line_start_index,
            error_index_within_line: error_index_within_input - self.current_line_start_index,
            error_index_within_input,
        })))
    }
}

impl<'a> Iterator for Scanner<'a> {
    type Item = Result<Token<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        /*!svars:re2c format = '#[allow(unused_mut)] let mut @@;'; */
        /*!stags:re2c format = 'let mut @@ = std::usize::MAX;'; */

        'lex: loop { /*!local:re2c
            re2c:encoding:utf8     = 1;
            re2c:api               = generic;
            re2c:tags              = 1;
            re2c:eof               = 0;
            re2c:yyfill:enable     = 0;

            re2c:define:YYCTYPE    = u8;
            re2c:define:YYPEEK     = "if self.cursor < self.s.len() { *self.s.get_unchecked(self.cursor) } else { 0 }";
            re2c:define:YYSKIP     = "self.cursor += 1;";
            re2c:define:YYBACKUP   = "self.mark = self.cursor;";
            re2c:define:YYRESTORE  = "self.cursor = self.mark;";
            re2c:YYBACKUPCTX        = "self.ctxmarker = self.cursor;";
            re2c:YYRESTORECTX       = "self.cursor = self.ctxmarker;";
            re2c:define:YYLESSTHAN = "self.s.len() <= self.cursor";
            re2c:YYSHIFT           = "self.cursor = (self.cursor as isize + @@{shift}) as usize;";
            re2c:YYSTAGP           = "@@{tag} = self.cursor;";
            re2c:YYSTAGN           = "@@{tag} = std::usize::MAX;";
            re2c:YYSHIFTSTAG       = "@@{tag} = (@@{tag} as isize + @@{shift}) as usize;";

            // New lines. Update the line number and line start index
            "\r\n" | "\n\r" | "\r" | "\n"           { self.current_line_number += 1; self.current_line_start_index = self.cursor; continue 'lex; }

            // Skip whitespace and tabs
            [\t ]+                                  { continue 'lex;}

            // Skip comments
            "//" [^\r\n]*                           { continue 'lex; }

            // Numbers. Following formats are supported: "123", "123.456". Not supported: ".123" or "123."
            @beg [0-9]+ ("." [0-9]+)?          @end { return self.create_number_token(beg, end) }

            // Strings. Only one line. Not allowed: escape sequences like "\n" inside the string.
            // We save the string without quotation marks.
            ["] @beg [^\r\n"]* @end ["]              { return self.create_string_token(beg, end) }

            // Tokens
            @beg "("                           @end { return self.create_token(TokenType::LEFT_PAREN, beg, end) }
            @beg ")"                           @end { return self.create_token(TokenType::RIGHT_PAREN, beg, end) }
            @beg "{"                           @end { return self.create_token(TokenType::LEFT_BRACE, beg, end) }
            @beg "}"                           @end { return self.create_token(TokenType::RIGHT_BRACE, beg, end) }
            @beg ","                           @end { return self.create_token(TokenType::COMMA, beg, end) }
            @beg "."                           @end { return self.create_token(TokenType::DOT, beg, end) }
            @beg "-"                           @end { return self.create_token(TokenType::MINUS, beg, end) }
            @beg "+"                           @end { return self.create_token(TokenType::PLUS, beg, end) }
            @beg ";"                           @end { return self.create_token(TokenType::SEMICOLON, beg, end) }
            @beg "*"                           @end { return self.create_token(TokenType::STAR, beg, end) }
            @beg "!="                          @end { return self.create_token(TokenType::BANG_EQUAL, beg, end) }
            @beg "!"                           @end { return self.create_token(TokenType::BANG, beg, end) }
            @beg "=="                          @end { return self.create_token(TokenType::EQUAL_EQUAL, beg, end) }
            @beg "="                           @end { return self.create_token(TokenType::EQUAL, beg, end) }
            @beg ">="                          @end { return self.create_token(TokenType::GREATER_EQUAL, beg, end) }
            @beg ">"                           @end { return self.create_token(TokenType::GREATER, beg, end) }
            @beg "<="                          @end { return self.create_token(TokenType::LESS_EQUAL, beg, end) }
            @beg "<"                           @end { return self.create_token(TokenType::LESS, beg, end) }
            @beg "/"                           @end { return self.create_token(TokenType::SLASH, beg, end) }
            @beg "and"                         @end { return self.create_token(TokenType::AND, beg, end) }
            @beg "class"                       @end { return self.create_token(TokenType::CLASS, beg, end) }
            @beg "else"                        @end { return self.create_token(TokenType::ELSE, beg, end) }
            @beg "false"                       @end { return self.create_token(TokenType::FALSE, beg, end) }
            @beg "true"                        @end { return self.create_token(TokenType::TRUE, beg, end) }
            @beg "fun"                         @end { return self.create_token(TokenType::FUN, beg, end) }
            @beg "for"                         @end { return self.create_token(TokenType::FOR, beg, end) }
            @beg "if"                          @end { return self.create_token(TokenType::IF, beg, end) }
            @beg "nil"                         @end { return self.create_token(TokenType::NIL, beg, end) }
            @beg "or"                          @end { return self.create_token(TokenType::OR, beg, end) }
            @beg "print"                       @end { return self.create_token(TokenType::PRINT, beg, end) }
            @beg "return"                      @end { return self.create_token(TokenType::RETURN, beg, end) }
            @beg "super"                       @end { return self.create_token(TokenType::SUPER, beg, end) }
            @beg "this"                        @end { return self.create_token(TokenType::THIS, beg, end) }
            @beg "var"                         @end { return self.create_token(TokenType::VAR, beg, end) }
            @beg "while"                       @end { return self.create_token(TokenType::WHILE, beg, end) }

            // Identifiers. For example: "var123", "_var", "var_123"
            @beg [A-Za-z_][A-Za-z0-9_]*        @end { return self.create_identifier(beg, end) }

            // Any other character is an error
            .                                       { return self.create_unexpected_symbol_error(self.cursor - 1) }

            // Catch ill-formed UTF-8 or orphan bytes
            *                                       { return self.create_unexpected_symbol_error(self.cursor - 1) }

            // End of input
            $                                       { return None; } */
        }
    }
}
