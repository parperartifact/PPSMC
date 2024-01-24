use nom::{
    bytes::complete::take, combinator::verify, IResult, InputIter, InputLength, InputTake, Needed,
    Slice,
};
use std::{
    iter::Enumerate,
    ops::{Range, RangeFrom, RangeFull, RangeTo},
};

#[derive(PartialEq, Debug, Clone)]
pub enum Token {
    // identifier and literals
    Ident(String),
    IntLiteral(i64),
    BoolLiteral(bool),
    // operators
    Conditional,
    Becomes,
    And,
    Or,
    Not,
    Xor,
    Iff,
    Imply,
    LtlFinally,
    LtlGlobally,
    // LtlHistorically,
    LtlOnce,
    LtlSince,
    // LtlTriggered,
    LtlUntil,
    LtlRelease,
    LtlNext,
    // LtlYesterday,
    // LtlWeakyesterday,

    // reserved words
    Case,
    Esac,
    Next,
    // reserved type word
    Boolean,
    // reserved section words
    Module,
    Define,
    InputVar,
    LatchVar,
    Constant,
    Init,
    Invariant,
    Trans,
    Fairness,
    LtlSpec,

    // punctuations
    Comma,
    Colon,
    SemiColon,
    LParen,
    RParen,
}

macro_rules! tag_token (
    ($func_name:ident, $tag: expr) => (
        pub fn $func_name(tokens: Tokens) -> IResult<Tokens, Token> {
            verify(take(1usize), |t: &Tokens| t.tok[0] == $tag)(tokens).map(|(tokens, ret)| {
                assert!(ret.tok.len() == 1);
                (tokens, ret.tok[0].clone())
            })
        }
    )
  );

tag_token!(module_tag, Token::Module);
tag_token!(define_tag, Token::Define);
tag_token!(latch_var_tag, Token::LatchVar);
tag_token!(input_var_tag, Token::InputVar);
tag_token!(init_tag, Token::Init);
tag_token!(trans_tag, Token::Trans);
tag_token!(invariant_tag, Token::Invariant);
tag_token!(fairness_tag, Token::Fairness);
tag_token!(ltlspec_tag, Token::LtlSpec);
tag_token!(becomes_tag, Token::Becomes);
tag_token!(not_tag, Token::Not);
tag_token!(and_tag, Token::And);
tag_token!(or_tag, Token::Or);
tag_token!(xor_tag, Token::Xor);
tag_token!(imply_tag, Token::Imply);
tag_token!(iff_tag, Token::Iff);
tag_token!(case_tag, Token::Case);
tag_token!(esac_tag, Token::Esac);
tag_token!(lparen_tag, Token::LParen);
tag_token!(rparen_tag, Token::RParen);
tag_token!(_conditional_tag, Token::Conditional);
tag_token!(colon_tag, Token::Colon);
tag_token!(semicolon_tag, Token::SemiColon);
tag_token!(boolean_tag, Token::Boolean);
tag_token!(next_tag, Token::Next);
tag_token!(ltl_globally_tag, Token::LtlGlobally);
tag_token!(ltl_finally_tag, Token::LtlFinally);
tag_token!(ltl_next_tag, Token::LtlNext);
tag_token!(ltl_once_tag, Token::LtlOnce);
tag_token!(ltl_until_tag, Token::LtlUntil);
tag_token!(ltl_release_tag, Token::LtlRelease);
tag_token!(ltl_since_tag, Token::LtlSince);

#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(C)]
pub struct Tokens<'a> {
    pub tok: &'a [Token],
    pub start: usize,
    pub end: usize,
}

impl<'a> Tokens<'a> {
    pub fn new(vec: &'a [Token]) -> Self {
        Tokens {
            tok: vec,
            start: 0,
            end: vec.len(),
        }
    }
}

impl<'a> InputLength for Tokens<'a> {
    fn input_len(&self) -> usize {
        self.tok.len()
    }
}

impl<'a> InputTake for Tokens<'a> {
    fn take(&self, count: usize) -> Self {
        Tokens {
            tok: &self.tok[0..count],
            start: 0,
            end: count,
        }
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
        let (prefix, suffix) = self.tok.split_at(count);
        let first = Tokens {
            tok: prefix,
            start: 0,
            end: prefix.len(),
        };
        let second = Tokens {
            tok: suffix,
            start: 0,
            end: suffix.len(),
        };
        (second, first)
    }
}

impl InputLength for Token {
    #[inline]
    fn input_len(&self) -> usize {
        1
    }
}

impl<'a> Slice<Range<usize>> for Tokens<'a> {
    #[inline]
    fn slice(&self, range: Range<usize>) -> Self {
        Tokens {
            tok: self.tok.slice(range.clone()),
            start: self.start + range.start,
            end: self.start + range.end,
        }
    }
}

impl<'a> Slice<RangeTo<usize>> for Tokens<'a> {
    #[inline]
    fn slice(&self, range: RangeTo<usize>) -> Self {
        self.slice(0..range.end)
    }
}

impl<'a> Slice<RangeFrom<usize>> for Tokens<'a> {
    #[inline]
    fn slice(&self, range: RangeFrom<usize>) -> Self {
        self.slice(range.start..self.end - self.start)
    }
}

impl<'a> Slice<RangeFull> for Tokens<'a> {
    #[inline]
    fn slice(&self, _: RangeFull) -> Self {
        Tokens {
            tok: self.tok,
            start: self.start,
            end: self.end,
        }
    }
}

impl<'a> InputIter for Tokens<'a> {
    type Item = &'a Token;
    type Iter = Enumerate<::std::slice::Iter<'a, Token>>;
    type IterElem = ::std::slice::Iter<'a, Token>;

    fn iter_indices(&self) -> Enumerate<::std::slice::Iter<'a, Token>> {
        self.tok.iter().enumerate()
    }

    fn iter_elements(&self) -> ::std::slice::Iter<'a, Token> {
        self.tok.iter()
    }

    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.tok.iter().position(predicate)
    }

    fn slice_index(&self, count: usize) -> Result<usize, Needed> {
        if self.tok.len() >= count {
            Ok(count)
        } else {
            Err(Needed::Unknown)
        }
    }
}
