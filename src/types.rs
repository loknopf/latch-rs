use serde::{Deserialize, Deserializer, Serialize, de::Error};

#[derive(Debug, Clone)]
pub(crate) enum Value {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) enum Access {
    RW,
    RO,
    WO,
}

impl Access {
    pub(crate) fn is_read_only(&self) -> bool {
        matches!(self, Access::RO)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BitRange {
    Single(u32),
    Span(u32, u32),
}

impl Serialize for BitRange {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            BitRange::Single(i) => serializer.serialize_str(&i.to_string()),
            BitRange::Span(s, e) => serializer.serialize_str(&format!("{}:{}", s, e)),
        }
    }
}

impl<'de> Deserialize<'de> for BitRange {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        match s.split_once(':') {
            None => s
                .parse::<u32>()
                .map(BitRange::Single)
                .map_err(Error::custom),
            Some((lo, hi)) => {
                let lo = lo.parse::<u32>().map_err(Error::custom)?;
                let hi = hi.parse::<u32>().map_err(Error::custom)?;
                Ok(BitRange::Span(lo, hi))
            }
        }
    }
}
