pub mod lox_language_scanner {
    include!(concat!(env!("OUT_DIR"), "/lox_language_scanner.rs"));
}

#[cfg(test)]
mod tests {
    use crate::lox_language_scanner::*;

    #[test]
    fn scan_tokens_empty_source() {
        let tokens: Vec<_> = Scanner::new("").collect();
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn token_has_correct_line_information() {
        let tokens: Vec<_> = Scanner::new("123\n\r345\n678")
            .map(|result| result.unwrap())
            .collect();
        assert_eq!(tokens.len(), 3);

        assert_token_number(&tokens[0].token_type, 123.0);
        assert_eq!(tokens[0].line_number, 0);
        assert_eq!(tokens[0].line_start_index, 0);
        assert_eq!(tokens[0].start_index_within_input, 0);
        assert_eq!(tokens[0].end_index_within_input, 3);
        assert_eq!(tokens[0].start_index_within_line, 0);
        assert_eq!(tokens[0].end_index_within_line, 3);

        assert_token_number(&tokens[1].token_type, 345.0);
        assert_eq!(tokens[1].line_number, 1);
        assert_eq!(tokens[1].line_start_index, 5);
        assert_eq!(tokens[1].start_index_within_input, 5);
        assert_eq!(tokens[1].end_index_within_input, 8);
        assert_eq!(tokens[1].start_index_within_line, 0);
        assert_eq!(tokens[1].end_index_within_line, 3);

        assert_token_number(&tokens[2].token_type, 678.0);
        assert_eq!(tokens[2].line_number, 2);
        assert_eq!(tokens[2].line_start_index, 9);
        assert_eq!(tokens[2].start_index_within_input, 9);
        assert_eq!(tokens[2].end_index_within_input, 12);
        assert_eq!(tokens[2].start_index_within_line, 0);
        assert_eq!(tokens[2].end_index_within_line, 3);
    }

    #[test]
    fn parses_numbers_correctly() {
        let tokens: Vec<_> = Scanner::new("123.345 345 678.0 99999999999999999999999999999.99999999999999999999999999999999999999999999999")
            .map(|result| result.unwrap())
            .collect();
        assert_eq!(tokens.len(), 4);

        assert_token_number(&tokens[0].token_type, 123.345);
        assert_token_number(&tokens[1].token_type, 345.0);
        assert_token_number(&tokens[2].token_type, 678.0);
        assert_token_number(&tokens[3].token_type, 1e29);
    }

    #[test]
    fn correctly_report_unexpected_symbol_error() {
        let input = String::from("\" \n;\"");
        let tokens: Vec<_> = Scanner::new(&input).collect();
        assert_eq!(tokens.len(), 3);

        match &tokens[0] {
            Ok(_) => panic!("Expected an error, but got Ok"),
            Err(err) => {
                err.downcast_ref::<UnexpectedSymbolError>()
                    .expect("Error should be UnexpectedSymbolError");
            }
        }
    }

    #[test]
    fn identifiers_and_keywords_are_distinguished() {
        let src = "and class else false true fun for if nil or print return super this var while foo _bar bar123";
        let toks: Vec<_> = Scanner::new(src).map(|r| r.unwrap()).collect();

        assert_eq!(toks.len(), 19);

        use TokenType::*;
        let expect = [
            AND, CLASS, ELSE, FALSE, TRUE, FUN, FOR, IF, NIL, OR, PRINT, RETURN, SUPER, THIS, VAR,
            WHILE,
        ];

        for (i, kw) in expect.iter().enumerate() {
            assert_token_variant(&toks[i].token_type, kw);
        }

        assert_token_ident(&toks[16].token_type, "foo");
        assert_token_ident(&toks[17].token_type, "_bar");
        assert_token_ident(&toks[18].token_type, "bar123");
    }

    #[test]
    fn comments_are_skipped() {
        let src = "123 // this is a comment\n 456";
        let toks: Vec<_> = Scanner::new(src).map(|r| r.unwrap()).collect();
        assert_eq!(toks.len(), 2);

        assert_token_number(&toks[0].token_type, 123.0);
        assert_token_number(&toks[1].token_type, 456.0);

        assert_eq!(toks[1].line_number, 1);
    }

    #[test]
    fn operator_disambiguation_prefers_longer_tokens() {
        let src = "!= ! == = >= > <= <";
        let toks: Vec<_> = Scanner::new(src).map(|r| r.unwrap()).collect();
        use TokenType::*;
        let expected = [
            BANG_EQUAL,
            BANG,
            EQUAL_EQUAL,
            EQUAL,
            GREATER_EQUAL,
            GREATER,
            LESS_EQUAL,
            LESS,
        ];

        assert_eq!(toks.len(), expected.len());
        for (tok, exp) in toks.iter().zip(expected.iter()) {
            assert_token_variant(&tok.token_type, exp);
        }
    }

    #[test]
    fn slash_as_token_and_comment() {
        let src = "/ // comment here\n/ ";
        let toks: Vec<_> = Scanner::new(src).map(|r| r.unwrap()).collect();
        assert_eq!(toks.len(), 2);

        use TokenType::SLASH;
        assert_token_variant(&toks[0].token_type, &SLASH);
        assert_token_variant(&toks[1].token_type, &SLASH);

        assert_eq!(toks[1].line_number, 1);
    }

    #[test]
    fn dot_vs_numbers_edge_cases() {
        let src = "123. .123 123.";
        let toks: Vec<_> = Scanner::new(src).map(|r| r.unwrap()).collect();
        assert_eq!(toks.len(), 6);

        use TokenType::DOT;

        assert_token_number(&toks[0].token_type, 123.0);
        assert_token_variant(&toks[1].token_type, &DOT);

        assert_token_variant(&toks[2].token_type, &DOT);
        assert_token_number(&toks[3].token_type, 123.0);

        assert_token_number(&toks[4].token_type, 123.0);
        assert_token_variant(&toks[5].token_type, &DOT);
    }

    #[test]
    fn unterminated_string_reports_error() {
        let tokens: Vec<_> = Scanner::new("\"abc").collect();
        assert_eq!(tokens.len(), 2);

        match &tokens[0] {
            Ok(_) => panic!("Expected an error for unterminated string"),
            Err(err) => {
                err.downcast_ref::<UnexpectedSymbolError>()
                    .expect("Error should be UnexpectedSymbolError");
            }
        }
    }

    #[test]
    fn unexpected_symbol_reports_error() {
        let tokens: Vec<_> = Scanner::new("@").collect();
        assert_eq!(tokens.len(), 1);

        match &tokens[0] {
            Ok(_) => panic!("Expected an error, but got Ok"),
            Err(err) => {
                let e = err
                    .downcast_ref::<UnexpectedSymbolError>()
                    .expect("Error should be UnexpectedSymbolError");
                assert_eq!(e.line_number, 0);
                assert_eq!(e.error_index_within_line, 0);
                assert_eq!(e.error_index_within_input, 0);
                assert_eq!(e.line_start_index_within_input, 0);
            }
        }
    }

    #[test]
    fn utf8_inside_strings() {
        let src = "\"héllö 😊\"";
        let toks: Vec<_> = Scanner::new(src).map(|r| r.unwrap()).collect();
        assert_eq!(toks.len(), 1);

        match &toks[0].token_type {
            TokenType::STRING(s) => assert_eq!(*s, "héllö 😊"),
            _ => panic!("Expected STRING token, got {:?}", toks[0].token_type),
        }

        assert_eq!(toks[0].start_index_within_input, 1);
        assert_eq!(toks[0].end_index_within_input, src.len() - 1);
    }

    #[test]
    fn continue_parsing_after_utf8_symbol() {
        let src = "ю123";
        let toks: Vec<_> = Scanner::new(src).collect();
        assert_eq!(toks.len(), 2);

        assert_token_number(&toks[1].as_ref().unwrap().token_type, 123.0);
    }

    #[test]
    fn example_test() {
        // Valid Lox source code
        let lox_src = r#"
            // variables and math
            var x = 42;
            var y = 3.14;
            print "hello, world";
            if (x >= y) {
                x = x + 1;
            } else {
                y = y - 1;
            }"#;

        // Scanner implements Iterator. Each iteration returns Result<Token>
        for token in Scanner::new(lox_src) {
            println!("{:?}", token);

            // If token is Err, then it means that the parsing error happened.
            if token.is_err() {
                // Handle UnexpectedSymbolError

                // In case of an error, we can continue parsing.
                // In this example we just break the loop.
                break;
            }
        }
    }

    fn assert_token_ident(token_type: &TokenType, expected: &str) {
        match token_type {
            TokenType::IDENTIFIER(s) => assert_eq!(*s, expected),
            _ => panic!("Expected IDENTIFIER token, got {:?}", token_type),
        }
    }

    fn assert_token_variant(token_type: &TokenType, expected: &TokenType) {
        use TokenType::*;
        let ok = match (token_type, expected) {
            (LEFT_PAREN, LEFT_PAREN)
            | (RIGHT_PAREN, RIGHT_PAREN)
            | (LEFT_BRACE, LEFT_BRACE)
            | (RIGHT_BRACE, RIGHT_BRACE)
            | (COMMA, COMMA)
            | (DOT, DOT)
            | (MINUS, MINUS)
            | (PLUS, PLUS)
            | (SEMICOLON, SEMICOLON)
            | (SLASH, SLASH)
            | (STAR, STAR)
            | (BANG, BANG)
            | (BANG_EQUAL, BANG_EQUAL)
            | (EQUAL, EQUAL)
            | (EQUAL_EQUAL, EQUAL_EQUAL)
            | (GREATER, GREATER)
            | (GREATER_EQUAL, GREATER_EQUAL)
            | (LESS, LESS)
            | (LESS_EQUAL, LESS_EQUAL)
            | (AND, AND)
            | (CLASS, CLASS)
            | (ELSE, ELSE)
            | (FALSE, FALSE)
            | (FUN, FUN)
            | (FOR, FOR)
            | (IF, IF)
            | (NIL, NIL)
            | (OR, OR)
            | (PRINT, PRINT)
            | (RETURN, RETURN)
            | (SUPER, SUPER)
            | (THIS, THIS)
            | (TRUE, TRUE)
            | (VAR, VAR)
            | (WHILE, WHILE) => true,
            _ => false,
        };
        if !ok {
            panic!(
                "Token mismatch. Got {:?}, expected {:?}",
                token_type, expected
            );
        }
    }

    fn assert_token_number(token_type: &TokenType, expected_value: f64) {
        match token_type {
            TokenType::NUMBER(value) => assert_eq!(*value, expected_value),
            _ => panic!("Expected NUMBER token, got {:?}", token_type),
        }
    }
}

fn main() {}
