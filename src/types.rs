use serde::{Deserialize, Deserializer, Serialize, de::Error};

#[derive(Debug, Clone)]
pub(crate) enum Value {
    BitSpec(BitSpec),
    Access(Access),
    QuotedCSV(Vec<String>),
    Bare(String),
}

impl Value {
    pub(crate) fn as_bit_spec(&self) -> Option<BitSpec> {
        match self {
            Self::BitSpec(b) => Some(*b),
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
            Self::BitSpec(BitSpec::Single(n)) => Some(*n),
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
pub(crate) enum BitSpec {
    Single(u64),
    Span(u64, u64),
}

impl Serialize for BitSpec {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            BitSpec::Single(i) => serializer.serialize_str(&i.to_string()),
            BitSpec::Span(s, e) => serializer.serialize_str(&format!("{}:{}", s, e)),
        }
    }
}

impl<'de> Deserialize<'de> for BitSpec {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        match s.split_once(':') {
            None => s.parse::<u64>().map(BitSpec::Single).map_err(Error::custom),
            Some((lo, hi)) => {
                let lo = lo.parse::<u64>().map_err(Error::custom)?;
                let hi = hi.parse::<u64>().map_err(Error::custom)?;
                Ok(BitSpec::Span(lo, hi))
            }
        }
    }
}
