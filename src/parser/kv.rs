use crate::parser::{LatchParser, Rule};
use pest::{Parser, error::Error, iterators::Pair};

pub(crate) fn parse_kv_pairs(input: &str) -> Result<Vec<KvPair>, Error<Rule>> {
    let kv_list = LatchParser::parse(Rule::kv_list, input)?
        .next()
        .expect("Expecting some pair to be parsed.");
    let mut pairs = Vec::new();
    for pair in kv_list.into_inner() {
        if pair.as_rule() == Rule::kv_pair {
            pairs.push(KvPair::from(pair));
        }
    }

    Ok(pairs)
}

pub(crate) struct KvPair {
    //maybe intern later
    pub key: String,
    pub value: Value,
}

impl From<Pair<'_, Rule>> for KvPair {
    fn from(value: Pair<'_, Rule>) -> Self {
        let mut inner = value.into_inner();
        let key = inner.next().unwrap().as_str().to_string();
        let value = Value::from(inner.next().expect("Expecting a kv_pair to have a value."));
        KvPair { key, value }
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    BitRange(BitRange),
    Access(Access),
    HexNumber(u64),
    BinNumber(u64),
    QuotedCSV(Vec<String>),
    Bare(String),
}

impl Value {
    pub(crate) fn as_bit_range(&self) -> Option<BitRange> {
        match self {
            Self::BitRange(b) => Some(*b),
            _ => None,
        }
    }

    pub(crate) fn as_access(&self) -> Option<Access> {
        match self {
            Self::Access(a) => Some(*a),
            _ => None,
        }
    }

    pub(crate) fn as_u64(&self) -> Option<u64> {
        match self {
            Self::BinNumber(n) | Self::HexNumber(n) => Some(*n),
            _ => None,
        }
    }

    pub(crate) fn as_string(&self) -> Option<&String> {
        match self {
            Self::Bare(s) => Some(s),
            _ => None,
        }
    }

    pub(crate) fn as_vec_string(&self) -> Option<&Vec<String>> {
        match self {
            Self::QuotedCSV(vs) => Some(vs),
            _ => None,
        }
    }
}

impl From<Pair<'_, Rule>> for Value {
    fn from(value: Pair<'_, Rule>) -> Self {
        let rule = value.as_rule();
        debug_assert!(matches!(rule, Rule::value));
        let mut inner = value.into_inner();
        let value = inner.next().unwrap();
        match value.as_rule() {
            Rule::access => Value::Access(Access::from(value)),
            Rule::bare_value => Value::Bare(value.as_str().to_string()),
            Rule::bin_number => {
                Value::BinNumber(u64::from_str_radix(&value.as_str()[2..], 2).unwrap())
            }
            Rule::bit_range => Value::BitRange(BitRange::from(value)),
            Rule::hex_number => {
                Value::HexNumber(u64::from_str_radix(&value.as_str()[2..], 16).unwrap())
            }
            Rule::quoted_csv => {
                let s = value.as_str();
                Value::QuotedCSV(
                    s[1..s.len() - 1] //skip " at beginning and end
                        .split(',') // split at , (separator)
                        .map(|f| f.to_string())
                        .collect(),
                )
            }
            _ => unreachable!("Only values can be contained in a value token."),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Access {
    RW,
    RO,
    WO,
}

impl From<Pair<'_, Rule>> for Access {
    fn from(value: Pair<'_, Rule>) -> Self {
        debug_assert!(matches!(value.as_rule(), Rule::access));
        match value.as_str() {
            "RW" => Access::RW,
            "RO" => Access::RO,
            "WO" => Access::WO,
            _ => unreachable!(
                "This access declaration is not supported. Only RO, RW or WO are supported."
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitRange {
    Single(u32),
    Span(u32, u32),
}

impl From<Pair<'_, Rule>> for BitRange {
    fn from(value: Pair<'_, Rule>) -> Self {
        debug_assert!(matches!(value.as_rule(), Rule::bit_range));
        let split: Vec<&str> = value.as_str().split(':').collect();
        if split.len() == 1 {
            BitRange::Single(split[0].parse().unwrap())
        } else if split.len() == 2 {
            BitRange::Span(split[0].parse().unwrap(), split[1].parse().unwrap())
        } else {
            unreachable!(
                "Only two kinds of bit ranges are supported: single indice (i.e. bits = 14) or ranges (i.e. bits = 3:6)",
            )
        }
    }
}
