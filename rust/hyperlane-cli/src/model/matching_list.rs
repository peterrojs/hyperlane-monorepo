//! The correct settings shape is defined in the TypeScript SDK metadata. While the the exact shape
//! and validations it defines are not applied here, we should mirror them.
//! ANY CHANGES HERE NEED TO BE REFLECTED IN THE TYPESCRIPT SDK.

use std::{
    fmt,
    fmt::{Debug, Display, Formatter},
    marker::PhantomData,
};

use hyperlane_core::{config::StrOrInt, utils::hex_or_base58_to_h256, H256};
use serde::ser::SerializeStruct;
use serde::{
    de::{Error, SeqAccess, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};

/// Defines a set of patterns for determining if a message should or should not
/// be relayed. This is useful for determine if a message matches a given set or
/// rules.
///
/// Valid options for each of the tuple elements are
/// - wildcard "*"
/// - single value in decimal or hex (must start with `0x`) format
/// - list of values in decimal or hex format
#[derive(Debug, Default, Clone)]
pub struct MatchingList(pub Option<Vec<ListElement>>);

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
enum Filter<T> {
    Wildcard,
    Enumerated(Vec<T>),
}

impl<T> Default for Filter<T> {
    fn default() -> Self {
        Self::Wildcard
    }
}

impl<T: Debug> Display for Filter<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Wildcard => write!(f, "*"),
            Self::Enumerated(l) if l.len() == 1 => write!(f, "{:?}", l[0]),
            Self::Enumerated(l) => {
                write!(f, "[")?;
                for i in l {
                    write!(f, "{i:?},")?;
                }
                write!(f, "]")
            }
        }
    }
}

struct MatchingListVisitor;
impl<'de> Visitor<'de> for MatchingListVisitor {
    type Value = MatchingList;

    fn expecting(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "an optional list of matching rules")
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(MatchingList(None))
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let list: Vec<ListElement> = deserializer.deserialize_seq(MatchingListArrayVisitor)?;
        Ok(if list.is_empty() {
            // this allows for empty matching lists to be treated as if no matching list was set
            MatchingList(None)
        } else {
            MatchingList(Some(list))
        })
    }
}

struct MatchingListArrayVisitor;
impl<'de> Visitor<'de> for MatchingListArrayVisitor {
    type Value = Vec<ListElement>;

    fn expecting(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "a list of matching rules")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut rules = seq.size_hint().map(Vec::with_capacity).unwrap_or_default();
        while let Some(rule) = seq.next_element::<ListElement>()? {
            rules.push(rule);
        }
        Ok(rules)
    }
}

struct FilterVisitor<T>(PhantomData<T>);
impl<'de> Visitor<'de> for FilterVisitor<u32> {
    type Value = Filter<u32>;

    fn expecting(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "Expecting either a wildcard \"*\", decimal/hex value string, or list of decimal/hex value strings")
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Self::Value::Enumerated(vec![v]))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        if v <= u32::MAX as u64 {
            Ok(Self::Value::Enumerated(vec![v as u32]))
        } else {
            Err(E::custom("Domain Id must fit within a u32 value"))
        }
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(if v == "*" {
            Self::Value::Wildcard
        } else {
            Self::Value::Enumerated(vec![v.parse::<u32>().map_err(to_serde_err)?])
        })
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(i) = seq.next_element::<StrOrInt>()? {
            values.push(i.try_into().map_err(to_serde_err)?);
        }
        Ok(Self::Value::Enumerated(values))
    }
}

impl<'de> Visitor<'de> for FilterVisitor<H256> {
    type Value = Filter<H256>;

    fn expecting(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(
            fmt,
            "Expecting either a wildcard \"*\", hex/base58 address string, or list of hex/base58 address strings"
        )
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(if v == "*" {
            Self::Value::Wildcard
        } else {
            Self::Value::Enumerated(vec![parse_addr(v)?])
        })
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(i) = seq.next_element::<String>()? {
            values.push(parse_addr(&i)?)
        }
        Ok(Self::Value::Enumerated(values))
    }
}

impl<'de> Deserialize<'de> for MatchingList {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        d.deserialize_option(MatchingListVisitor)
    }
}

impl<'de> Deserialize<'de> for Filter<u32> {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        d.deserialize_any(FilterVisitor::<u32>(Default::default()))
    }
}

impl<'de> Deserialize<'de> for Filter<H256> {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        d.deserialize_any(FilterVisitor::<H256>(Default::default()))
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ListElement {
    #[serde(default, rename = "origindomain")]
    origin_domain: Filter<u32>,
    #[serde(default, rename = "senderaddress")]
    sender_address: Filter<H256>,
    #[serde(default, rename = "destinationdomain")]
    destination_domain: Filter<u32>,
    #[serde(default, rename = "recipientaddress")]
    recipient_address: Filter<H256>,
}

impl Serialize for ListElement {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("ListElement", 4)?;
        s.serialize_field("origindomain", &self.origin_domain)?;
        s.serialize_field("destinationdomain", &self.destination_domain)?;

        serialize_filter_h256::<S>(&mut s, "recipientaddress", &self.recipient_address)?;
        serialize_filter_h256::<S>(&mut s, "senderaddress", &self.sender_address)?;

        s.end()
    }
}

fn serialize_filter_h256<S: Serializer>(
    s: &mut S::SerializeStruct,
    field: &'static str,
    filter: &Filter<H256>,
) -> Result<(), S::Error> {
    if let Filter::Enumerated(values) = filter {
        let hex_values: Vec<String> = values
            .iter()
            .map(|value| {
                let hex_encoded = hex::encode(value.as_bytes());
                if hex_encoded.chars().count() > 40 {
                    let last_40 = hex_encoded
                        .chars()
                        .rev()
                        .take(40)
                        .collect::<Vec<_>>()
                        .into_iter()
                        .rev()
                        .collect::<String>();
                    format!("\\x{}", last_40)
                } else {
                    format!("\\x{}", hex_encoded)
                }
            })
            .collect();
        s.serialize_field(field, &hex_values)?;
    }
    Ok(())
}

impl Display for ListElement {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{originDomain: {}, senderAddress: {}, destinationDomain: {}, recipientAddress: {}}}",
            self.origin_domain,
            self.sender_address,
            self.destination_domain,
            self.recipient_address
        )
    }
}

impl Display for MatchingList {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(wl) = &self.0 {
            write!(f, "[")?;
            for i in wl {
                write!(f, "{i},")?;
            }
            write!(f, "]")
        } else {
            write!(f, "null")
        }
    }
}

fn to_serde_err<IE: ToString, OE: Error>(e: IE) -> OE {
    OE::custom(e.to_string())
}

fn parse_addr<E: Error>(addr_str: &str) -> Result<H256, E> {
    hex_or_base58_to_h256(addr_str).map_err(to_serde_err)
}

#[cfg(test)]
mod test {
    use super::{Filter::*, MatchingList};

    #[test]
    fn config_with_multiple_domains() {
        let whitelist: MatchingList =
            serde_json::from_str(r#"[{"destinationdomain": ["13372", "13373"]}]"#).unwrap();
        assert!(whitelist.0.is_some());
        assert_eq!(whitelist.0.as_ref().unwrap().len(), 1);
        let elem = &whitelist.0.as_ref().unwrap()[0];
        assert_eq!(elem.destination_domain, Enumerated(vec![13372, 13373]));
        assert_eq!(elem.recipient_address, Wildcard);
        assert_eq!(elem.origin_domain, Wildcard);
        assert_eq!(elem.sender_address, Wildcard);
    }

    #[test]
    fn config_with_empty_list_is_none() {
        let whitelist: MatchingList = serde_json::from_str(r#"[]"#).unwrap();
        assert!(whitelist.0.is_none());
    }

    #[test]
    fn supports_base58() {
        serde_json::from_str::<MatchingList>(
            r#"[{"origindomain":1399811151,"senderaddress":"DdTMkk9nuqH5LnD56HLkPiKMV3yB3BNEYSQfgmJHa5i7","destinationdomain":11155111,"recipientaddress":"0x6AD4DEBA8A147d000C09de6465267a9047d1c217"}]"#,
        ).unwrap();
    }
}