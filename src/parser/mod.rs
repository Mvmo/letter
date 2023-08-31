type LSInt = i64;
type LSFloat = f64;

#[derive(Debug, PartialEq)]
enum NumberLiteral {
    FloatingPoint(LSFloat),
    Integer(LSInt),
}

#[derive(Debug, PartialEq)]
enum Operator {
    Plus,
    Minus,
    Assign,
    CmpEqual,
    CmpGreaterThanOrEqual,
    CmpGreaterThan,
    CmpLessThanOrEqual,
    CmpLessThan
}

#[derive(Debug, PartialEq)]
enum Keyword {
    Let,
    Fn,
}

#[derive(Debug, PartialEq)]
enum Token {
    Identifier(String),
    Number(NumberLiteral),
    Operator(Operator),
    Keyword(Keyword),
    Semicolon,
    LeftParen,
    RightParen,
}

fn as_num(word: String) -> Option<NumberLiteral> {
    if let Ok(int_val) = word.parse::<LSInt>() {
        Some(NumberLiteral::Integer(int_val))
    } else if let Ok(float_val) = word.parse::<LSFloat>() {
        Some(NumberLiteral::FloatingPoint(float_val))
    } else {
        None
    }
}

fn tokenize(input: String) -> Vec<Token> {
    let chars = input.chars();

    let mut tokens: Vec<Token> = vec![];

    let mut word: String = String::new();
    fn build_token(word: String) -> Token {
        match word.as_str() {
            "let" => Token::Keyword(Keyword::Let),
            "fn" => Token::Keyword(Keyword::Fn),
            "=" => Token::Operator(Operator::Assign),
            "==" => Token::Operator(Operator::CmpEqual),
            ">" => Token::Operator(Operator::CmpGreaterThan),
            ">=" => Token::Operator(Operator::CmpGreaterThanOrEqual),
            "<" => Token::Operator(Operator::CmpLessThan),
            "<=" => Token::Operator(Operator::CmpLessThanOrEqual),
            "+" => Token::Operator(Operator::Plus),
            "-" => Token::Operator(Operator::Minus),
            _ => {
                if let Ok(int_val) = word.parse::<LSInt>() {
                    Token::Number(NumberLiteral::Integer(int_val))
                } else if let Ok(float_val) = word.parse::<LSFloat>() {
                    Token::Number(NumberLiteral::FloatingPoint(float_val))
                } else {
                    Token::Identifier(word.to_string())
                }
            }
        }
    }
    for (idx, char) in chars.enumerate() {
        if !char.is_ascii() {
            panic!("Only ascii characters are allowed in letter scripts");
        }

        match char {
            ' ' => {
                if word.len() == 0 {
                    continue;
                }

                tokens.push(build_token(word.clone()));
                word.clear();
                continue;
            },
            ';' => {
                tokens.push(build_token(word.clone()));
                tokens.push(Token::Semicolon);
                word.clear();
                continue;
            },
            '(' => {
                tokens.push(build_token(word.clone()));
                tokens.push(Token::LeftParen);
                word.clear();
                continue;
            },
            ')' => {
                tokens.push(build_token(word.clone()));
                tokens.push(Token::RightParen);
                word.clear();
                continue;
            },
            _ => {}
        }

        word.push(char);
        if idx == input.len() - 1 {
            tokens.push(build_token(word.clone()));
            word.clear();
            continue;
        }
    }

    tokens
}

#[test]
fn test_tokenize() {
    let test_cases: Vec<(&str, Vec<Token>)> = vec![
        (
            "x = 5",
            vec![
                Token::Identifier("x".to_string()),
                Token::Operator(Operator::Assign),
                Token::Number(NumberLiteral::Integer(5))
            ]
        ),
        (
            "let x = 5;",
            vec![
                Token::Keyword(Keyword::Let),
                Token::Identifier("x".to_string()),
                Token::Operator(Operator::Assign),
                Token::Number(NumberLiteral::Integer(5)),
                Token::Semicolon
            ]
        ),
        (
            "let x; let y; let z; fn hallo(x) = 10 = 500;",
            vec![
                Token::Keyword(Keyword::Let),
                Token::Identifier("x".to_string()),
                Token::Semicolon,
                Token::Keyword(Keyword::Let),
                Token::Identifier("y".to_string()),
                Token::Semicolon,
                Token::Keyword(Keyword::Let),
                Token::Identifier("z".to_string()),
                Token::Semicolon,
                Token::Keyword(Keyword::Fn),
                Token::Identifier("hallo".to_string()),
                Token::LeftParen,
                Token::Identifier("x".to_string()),
                Token::RightParen,
                Token::Operator(Operator::Assign),
                Token::Number(NumberLiteral::Integer(10)),
                Token::Operator(Operator::Assign),
                Token::Number(NumberLiteral::Integer(500)),
                Token::Semicolon,
            ]
        )
    ];

    for test_case in test_cases {
        let input = test_case.0;
        let expected = test_case.1;
        let actual = tokenize(input.to_string());

        let expected = dbg!(expected);
        let actual = dbg!(actual);

        println!("{input}");
        assert_eq!(expected.len(), actual.len());
        for (idx, token) in expected.iter().enumerate() {
            assert_eq!(actual.get(idx).unwrap(), token);
        }
    }
}

