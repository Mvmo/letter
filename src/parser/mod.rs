type LSInt = i64;
type LSFloat = f64;

#[derive(Debug, PartialEq, Clone)]
enum NumberLiteral {
    FloatingPoint(LSFloat),
    Integer(LSInt),
}

#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, PartialEq, Clone)]
enum Keyword {
    Let,
    Fn,
}

#[derive(Debug, PartialEq, Clone)]
enum Token {
    Identifier(String),
    Number(NumberLiteral),
    Operator(Operator),
    Keyword(Keyword),
    Semicolon,
    LeftParen,
    RightParen,
    Eof,
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

    tokens.push(Token::Eof);
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
                Token::Number(NumberLiteral::Integer(5)),
                Token::Eof
            ]
        ),
        (
            "let x = 5;",
            vec![
                Token::Keyword(Keyword::Let),
                Token::Identifier("x".to_string()),
                Token::Operator(Operator::Assign),
                Token::Number(NumberLiteral::Integer(5)),
                Token::Semicolon,
                Token::Eof
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
                Token::Eof
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

enum Precedence {
    Lowest = 0,
    Sum,
    Prefix,
    Call
}

#[derive(Debug, PartialEq)]
struct IdentifierExpr {
    token: Token,
    value: String
}

#[derive(Debug, PartialEq)]
enum Node {
    ExpressionNode(),
    StatementNode()
}

#[derive(Debug, PartialEq)]
enum Stmt {
    LetStmt(LetStmt),
    N
}

#[derive(Debug, PartialEq)]
enum Expr {
    Identifier(IdentifierExpr)
}

#[derive(Default, Debug, PartialEq)]
struct ProgramNode {
    stmt_nodes: Vec<Stmt>
}

#[derive(Debug)]
struct Parser {
    tokens: Vec<Token>,
    current_tkn_idx: usize,
    peek_token_idx: usize,
    program_node: ProgramNode
}

#[derive(Debug, PartialEq)]
struct LetStmt {
    token: Token,
    identifier: Option<IdentifierExpr>,
    value: Option<Expr>
}

impl LetStmt {
    fn new(token: Token) -> Self {
        LetStmt { token, identifier: None, value: None }
    }
}

struct ExprStmt {
    token: Token,
    expression_node: Option<Expr>
}

//impl Parser {
//    fn new(tokens: Vec<Token>) -> Self {
//        Self {
//            tokens,
//            current_tkn_idx: 0,
//            peek_token_idx: 1,
//            program_node: ProgramNode::default()
//        }
//    }
//
//    fn parse_program(mut self) -> ProgramNode {
//        while self.current_token() != &Token::Eof {
//            let stmt = self.parse_statement();
//            self.program_node.stmt_nodes.push(stmt);
//
//            self.next_token();
//        }
//
//        return self.program_node;
//    }
//
//    fn parse_statement(&mut self) -> Stmt {
//        match self.current_token() {
//            Token::Keyword(Keyword::Let) => Stmt::LetStmt(self.parse_let_statement()),
//            _ => Stmt::N
//        }
//    }
//
//    fn parse_let_statement(&mut self) -> LetStmt {
//        let mut stmt_node = LetStmt::new(self.current_token().clone());
//        let peek_tkn = self.peek_token().clone();
//        if let Token::Identifier(ident) = peek_tkn {
//            self.next_token();
//            let current_token = self.current_token();
//            stmt_node.identifier = Some(IdentifierExpr { token: current_token.clone(), value: ident.to_string() });
//        } else {
//            Self::unexpected_token("Identifier", peek_tkn);
//        }
//
//        let peek_tkn = self.peek_token().clone();
//        if let Token::Operator(Operator::Assign) = self.peek_token() {
//            self.next_token();
//        } else {
//            Self::unexpected_token("Assign", peek_tkn);
//        }
//
//        self.next_token();
//
//        stmt_node.value = Some(self.parse_expression(Precedence::Lowest));
//        self.skip_semicolon();
//
//        stmt_node
//    }
//
//    fn parse_expression_statement(&mut self) -> ExprStmt {
//        let mut stmt_node = ExprStmt { token: self.current_token().clone(), expression_node: None };
//        stmt_node.expression_node = Some(self.parse_expression(Precedence::Lowest));
//
//        let peek_tkn = self.peek_token();
//        if let Token::Semicolon = peek_tkn {
//            self.next_token()
//        }
//
//        stmt_node
//    }
//
//    fn parse_expression(&mut self, precedence: Precedence) -> Expr {
//        if let Some(left_expr) = self.prefix_parse() {
//
//        } else {
//            panic!("no prefix parse fn found for current token");
//        }
//        //return Expr::N
//    }
//
//    fn skip_semicolon(&mut self) {
//        if let Token::Semicolon = self.peek_token() {
//            self.next_token()
//        }
//    }
//
//    fn prefix_parse(&mut self) -> Option<Expr> {
//        let token = self.current_token();
//        return match token {
//            Token::Identifier(_) => Some(Expr::Identifier(self.parse_identifier())),
//            _ => None,
//        };
//    }
//
//    fn parse_identifier(&mut self) -> IdentifierExpr {
//        if let Token::Identifier(ident) = self.current_token() {
//            IdentifierExpr {
//                token: self.current_token().clone(),
//                value: ident.clone(),
//            }
//        } else {
//            Self::unexpected_token("identifier", self.current_token().clone());
//
//        }
//    }
//
//    fn next_token(&mut self) {
//        self.current_tkn_idx = self.peek_token_idx;
//        self.peek_token_idx += 1;
//    }
//
//    fn peek_token(&self) -> &Token {
//        self.tokens.get(self.peek_token_idx).unwrap_or(&Token::Eof)
//    }
//
//    fn current_token(&self) -> &Token {
//        self.tokens.get(self.current_tkn_idx).unwrap_or(&Token::Eof)
//    }
//
//    fn unexpected_token(expected: &str, token: Token) -> ! {
//        dbg!(token);
//        panic!("Unexpected token | expected -> {expected}");
//    }
//}
//
//#[test]
//fn parser_test() {
//    let tokens = tokenize("let x = 9;".to_string());
//    let tokens = dbg!(tokens);
//    let program = Parser::new(tokens);
//    let program_node = program.parse_program();
//    let program_node_expected = ProgramNode {
//        stmt_nodes: vec![
//            Stmt::LetStmt(LetStmt {
//                token: Token::Keyword(Keyword::Let),
//                identifier: Some(IdentifierExpr { token: Token::Identifier("x".to_string()), value: "x".to_string() }),
//            })
//        ]
//    };
//
//    //dbg!(program_node);
//    assert_eq!(program_node, program_node_expected);
//}
