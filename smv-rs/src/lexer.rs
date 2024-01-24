use crate::token::Token;
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    character::complete::{alpha1, alphanumeric1, anychar, char, digit1, multispace0},
    combinator::{map, map_res, recognize},
    multi::{many0, many1},
    sequence::{delimited, pair, tuple},
    IResult,
};
use std::str::FromStr;

macro_rules! syntax {
    ($func_name: ident, $tag_string: literal, $output_token: expr) => {
        fn $func_name(s: &str) -> IResult<&str, Token> {
            map(tag($tag_string), |_| $output_token)(s)
        }
    };
}

syntax! {and_operator, "&", Token::And}
syntax! {or_operator, "|", Token::Or}
syntax! {xor_operator, "xor", Token::Xor}
syntax! {question_operator, "?", Token::Conditional}
syntax! {becomes_operator, ":=", Token::Becomes}
syntax! {not_operator, "!", Token::Not}
syntax! {iff_operator, "<->", Token::Iff}
syntax! {imply_operator, "->", Token::Imply}

pub fn lex_operator(input: &str) -> IResult<&str, Token> {
    alt((
        and_operator,
        or_operator,
        xor_operator,
        question_operator,
        becomes_operator,
        not_operator,
        iff_operator,
        imply_operator,
    ))(input)
}

// punctuations
syntax! {comma_punctuation, ",", Token::Comma}
syntax! {semicolon_punctuation, ";", Token::SemiColon}
syntax! {colon_punctuation, ":", Token::Colon}
syntax! {lparen_punctuation, "(", Token::LParen}
syntax! {rparen_punctuation, ")", Token::RParen}
// syntax! {lbrace_punctuation, "{", Token::LBrace}
// syntax! {rbrace_punctuation, "}", Token::RBrace}
// syntax! {lbracket_punctuation, "[", Token::LBracket}
// syntax! {rbracket_punctuation, "]", Token::RBracket}

pub fn lex_punctuations(input: &str) -> IResult<&str, Token> {
    alt((
        comma_punctuation,
        semicolon_punctuation,
        colon_punctuation,
        lparen_punctuation,
        rparen_punctuation,
        // lbrace_punctuation,
        // rbrace_punctuation,
        // lbracket_punctuation,
        // rbracket_punctuation,
    ))(input)
}

fn lex_integer(input: &str) -> IResult<&str, Token> {
    map(map_res(digit1, FromStr::from_str), Token::IntLiteral)(input)
}

fn lex_reserved_ident(input: &str) -> IResult<&str, Token> {
    map(
        recognize(pair(
            alt((alpha1, tag("_"))),
            many0(alt((alphanumeric1, tag("_"), tag(".")))),
        )),
        |s| match s {
            "case" => Token::Case,
            "esac" => Token::Esac,
            "next" => Token::Next,
            "boolean" => Token::Boolean,
            "MODULE" => Token::Module,
            "DEFINE" => Token::Define,
            "IVAR" => Token::InputVar,
            "VAR" => Token::LatchVar,
            "CONSTANTS" => Token::Constant,
            "INIT" => Token::Init,
            "INVAR" => Token::Invariant,
            "TRANS" => Token::Trans,
            "FAIRNESS" => Token::Fairness,
            "LTLSPEC" => Token::LtlSpec,
            "TRUE" => Token::BoolLiteral(true),
            "FALSE" => Token::BoolLiteral(false),
            "F" => Token::LtlFinally,
            "G" => Token::LtlGlobally,
            "U" => Token::LtlUntil,
            "V" => Token::LtlRelease,
            "X" => Token::LtlNext,
            "O" => Token::LtlOnce,
            "S" => Token::LtlSince,
            _ => Token::Ident(s.to_string()),
        },
    )(input)
}

fn lex_token(input: &str) -> IResult<&str, Token> {
    alt((
        lex_operator,
        lex_punctuations,
        lex_integer,
        lex_reserved_ident,
    ))(input)
}

fn comment_line(input: &str) -> IResult<&str, Vec<Token>> {
    tuple((many0(char(' ')), tag("--"), many0(anychar)))(input).map(|res| (res.0, Vec::new()))
}

fn lex_tokens_in_line(input: &str) -> IResult<&str, Vec<Token>> {
    let (input, line) = is_not("\n")(input)?;
    alt((
        comment_line,
        many1(delimited(multispace0, lex_token, multispace0)),
    ))(line)
    .map(|(remain, token)| {
        assert!(remain.is_empty());
        (input, token)
    })
}

pub fn lex_tokens(input: &str) -> Result<Vec<Token>, nom::Err<nom::error::Error<&str>>> {
    many0(delimited(multispace0, lex_tokens_in_line, multispace0))(input).map(|(remain, token)| {
        assert!(remain.is_empty());
        token.concat()
    })
}
