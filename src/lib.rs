//! Introduction
//! ============
//!
//! This library is only a generic deserializer/API for gettext. With this you can
//! use JSON or YAML (or "any" format handled by serde) to translate text through
//! gettext and even format. It also has an API for strftime for formatting dates.
//!
//! You can use it in an API service to have a translation endpoint or in a lambda
//! to translate the input.
//!
//!  *  Example in JSON
//!
//!     ```json
//!     {
//!         "ngettext": {
//!             "singular": "One item has been deleted",
//!             "plural": "%(n)s items have been deleted",
//!             "n": 5,
//!         }
//!     }
//!     ```
//!
//!  *  Example in YAML
//!
//!     ```yaml
//!     ngettext:
//!         singular: One item has been deleted
//!         plural: "%(n)s items have been deleted"
//!         n: 5
//!     ```
//!
//! When the structure is deserialized, you can simply convert it to a translated
//! `String`:
//!
//! ```rust
//! use serde_gettext::SerdeGetText;
//! use std::convert::TryFrom;
//!
//! let yaml = r#"---
//! ngettext:
//!     singular: One item has been deleted
//!     plural: "%(n)s items have been deleted"
//!     n: 5
//! "#;
//! let s: SerdeGetText = serde_yaml::from_str(yaml).unwrap();
//!
//! assert_eq!(String::try_from(s).unwrap(), "5 items have been deleted");
//! ```
//!
//! Formatting
//! ==========
//!
//!  *  Example in JSON
//!
//!     ```json
//!     {
//!         "gettext": "Hello %(name)s!",
//!         "args": {
//!             "name": "Grace",
//!         }
//!     }
//!     ```
//!
//!  *  Example in YAML
//!
//!     ```yaml
//!     gettext: "Hello %(name)s!"
//!     args:
//!         name: Grace
//!     ```
//!
//! List of All Available Functions
//! ===============================
//!
//!  *  gettext:
//!
//!     ```yaml
//!     gettext: "msgid"
//!     ```
//!
//!  *  ngettext:
//!
//!     ```yaml
//!     ngettext:
//!         singular: "msgid_singular"
//!         plural: "msgid_singular"
//!         n: 5
//!     ```
//!
//!  *  pgettext:
//!
//!     ```yaml
//!     pgettext:
//!         ctx: "context"
//!         msgid: "msgid"
//!     ```
//!
//!  *  dgettext:
//!
//!     ```yaml
//!     dgettext:
//!         domain: "domain"
//!         msgid: "msgid"
//!     ```
//!
//!  *  dngettext:
//!
//!     ```yaml
//!     dngettext:
//!         domain: "domain"
//!         singular: "msgid_singular"
//!         plural: "msgid_singular"
//!         n: 5
//!     ```
//!
//!  *  npgettext:
//!
//!     ```yaml
//!     npgettext:
//!         ctx: "context"
//!         singular: "msgid_singular"
//!         plural: "msgid_singular"
//!         n: 5
//!     ```
//!
//!  *  dcngettext:
//!
//!     ```yaml
//!     dcngettext:
//!         domain: "domain"
//!         singular: "msgid_singular"
//!         plural: "msgid_singular"
//!         n: 5
//!         cateogy: "ctype|numeric|time|collate|monetary|messages|all|paper|name|address|telephone|measurement|identification"
//!     ```

#![deny(missing_docs)]

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate derive_error;

use dynfmt::{Argument, Format, FormatArgs, PythonFormat};
use libc_strftime::strftime_local;
#[allow(unused_imports)]
use serde::Deserialize;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::string::ToString;

/// Runtime error that occurs when the input cannot be formatted
#[derive(Debug, Error)]
pub enum Error {
    /// Formatting error
    #[error(msg_embedded, no_from, non_std)]
    FormatError(String),
    /// Missing join separator
    #[error(non_std, no_from, display = "missing join separator")]
    MissingJoinSeparator,
}

/// A deserializable struct to translate and format
#[derive(Deserialize, Clone, Debug)]
pub struct SerdeGetText {
    #[serde(flatten)]
    value: Value,
    /// Base arguments that can be provided for keywords format
    #[serde(skip)]
    pub args: HashMap<String, String>,
}

impl TryFrom<SerdeGetText> for String {
    type Error = Error;

    fn try_from(x: SerdeGetText) -> Result<String, Error> {
        x.value.try_into_string(&x.args)
    }
}

impl TryFrom<Box<SerdeGetText>> for String {
    type Error = Error;

    fn try_from(x: Box<SerdeGetText>) -> Result<String, Error> {
        String::try_from(*x)
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
enum Value {
    Text(String),
    Integer(i64),
    Float(f64),
    Bool(bool),
    Unit(()),
    Datetime(DatetimeValue),
    Array(Vec<Value>),
    FormattedText {
        text: String,
        args: Option<Formatter>,
    },
    GetText {
        gettext: ValueGetText,
        args: Option<Formatter>,
    },
    NGetText {
        ngettext: ValueNGetText,
        args: Option<Formatter>,
    },
    PGetText {
        pgettext: ValuePGetText,
        args: Option<Formatter>,
    },
    DGetText {
        dgettext: ValueDGetText,
        args: Option<Formatter>,
    },
    DNGetText {
        dngettext: ValueDNGetText,
        args: Option<Formatter>,
    },
    NPGetText {
        npgettext: ValueNPGetText,
        args: Option<Formatter>,
    },
    DCNGetText {
        dcngettext: ValueDCNGetText,
        args: Option<Formatter>,
    },
}

macro_rules! handle_gettext {
    ($s:expr, $args:expr, $map:expr, $base_map:expr) => {{
        Self::format(&$s.to_string(), $args, $map, $base_map)
    }};
}

macro_rules! handle_plural {
    ($s:expr, $args:expr, $map:expr, $base_map:expr) => {{
        $map.reserve(match $args.as_ref() {
            Some(Formatter::KeywordArgs(args)) => args.len() + 1,
            _ => 1,
        });
        $map.insert("n".to_string(), $s.n.to_string());

        Self::format(&$s.to_string(), $args, $map, $base_map)
    }};
}

impl Value {
    fn try_into_string(self, base_map: &HashMap<String, String>) -> Result<String, Error> {
        let mut map = HashMap::new();

        match self {
            Value::Text(x) => Ok(x.to_string()),
            Value::Integer(x) => Ok(x.to_string()),
            Value::Float(x) => Ok(x.to_string()),
            Value::Bool(x) => Ok(if x {
                gettextrs::gettext(b"yes" as &[u8])
            } else {
                gettextrs::gettext(b"no" as &[u8])
            }),
            Value::Unit(()) => Ok(gettextrs::gettext(b"n/a" as &[u8])),
            Value::Datetime(x) => Ok(x.to_string()),
            Value::Array(xs) => Ok({
                let mut it = xs.into_iter();
                let sep: String = match it.next() {
                    Some(x) => x.try_into_string(base_map),
                    None => Err(Error::MissingJoinSeparator),
                }?;

                let mut vec: Vec<String> = Vec::new();

                for value in it {
                    vec.push(value.try_into_string(base_map)?);
                }

                vec.join(&sep)
            }),
            Value::FormattedText { text, args } => Self::format(text.as_ref(), args, map, base_map),
            Value::GetText { gettext, args } => handle_gettext!(gettext, args, map, base_map),
            Value::NGetText { ngettext, args } => handle_plural!(ngettext, args, map, base_map),
            Value::PGetText { pgettext, args } => handle_gettext!(pgettext, args, map, base_map),
            Value::DGetText { dgettext, args } => handle_gettext!(dgettext, args, map, base_map),
            Value::DNGetText { dngettext, args } => handle_plural!(dngettext, args, map, base_map),
            Value::NPGetText { npgettext, args } => handle_plural!(npgettext, args, map, base_map),
            Value::DCNGetText { dcngettext, args } => {
                handle_plural!(dcngettext, args, map, base_map)
            }
        }
    }

    fn format(
        message: &str,
        formatter: Option<Formatter>,
        mut map: HashMap<String, String>,
        base_map: &HashMap<String, String>,
    ) -> Result<String, Error> {
        match formatter {
            Some(Formatter::KeywordArgs(kwargs)) => {
                for (key, value) in kwargs.into_iter() {
                    map.insert(key, value.try_into_string(base_map)?);
                }

                PythonFormat
                    .format(message, UnionMap::new(&map, base_map))
                    .map_err(|err| Error::FormatError(format!("{}", err)))
                    .map(|x| x.to_string())
            }
            Some(Formatter::PositionalArgs(args)) => PythonFormat
                .format(
                    message,
                    args.into_iter()
                        .map(|x| x.try_into_string(base_map))
                        .collect::<Result<Vec<String>, _>>()?,
                )
                .map_err(|err| Error::FormatError(format!("{}", err)))
                .map(|x| x.to_string()),
            None => PythonFormat
                .format(message, UnionMap::new(&map, base_map))
                .map_err(|err| Error::FormatError(format!("{}", err)))
                .map(|x| x.to_string()),
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
enum Formatter {
    KeywordArgs(HashMap<String, Value>),
    PositionalArgs(Vec<Value>),
}

struct UnionMap<'a>(&'a HashMap<String, String>, &'a HashMap<String, String>);

impl<'a> UnionMap<'a> {
    fn new(a: &'a HashMap<String, String>, b: &'a HashMap<String, String>) -> UnionMap<'a> {
        UnionMap(a, b)
    }
}

impl FormatArgs for UnionMap<'_> {
    fn get_key(&self, key: &str) -> Result<Option<Argument<'_>>, ()> {
        Ok(self
            .0
            .get(key)
            .or_else(|| self.1.get(key))
            .map(|x| x as Argument<'_>))
    }
}

#[derive(Deserialize, Clone, Debug)]
struct DatetimeValue {
    strftime: String,
    epoch: i64,
}

impl ToString for DatetimeValue {
    fn to_string(&self) -> String {
        strftime_local(&self.strftime, self.epoch)
    }
}

#[derive(Deserialize, Clone, Debug)]
struct ValueGetText(String);

impl ToString for ValueGetText {
    fn to_string(&self) -> String {
        gettextrs::gettext(self.0.as_bytes())
    }
}

#[derive(Deserialize, Clone, Debug)]
struct ValueNGetText {
    singular: String,
    plural: String,
    n: u32,
}

impl ToString for ValueNGetText {
    fn to_string(&self) -> String {
        gettextrs::ngettext(self.singular.as_bytes(), self.plural.as_bytes(), self.n)
    }
}

#[derive(Deserialize, Clone, Debug)]
struct ValuePGetText {
    ctx: String,
    msgid: String,
}

impl ToString for ValuePGetText {
    fn to_string(&self) -> String {
        gettextrs::pgettext(self.ctx.as_bytes(), self.msgid.as_bytes())
    }
}

#[derive(Deserialize, Clone, Debug)]
struct ValueDGetText {
    domain: String,
    msgid: String,
}

impl ToString for ValueDGetText {
    fn to_string(&self) -> String {
        gettextrs::dgettext(self.domain.as_bytes(), self.msgid.as_bytes())
    }
}

#[derive(Deserialize, Clone, Debug)]
struct ValueDNGetText {
    domain: String,
    singular: String,
    plural: String,
    n: u32,
}

impl ToString for ValueDNGetText {
    fn to_string(&self) -> String {
        gettextrs::dngettext(
            self.domain.as_bytes(),
            self.singular.as_bytes(),
            self.plural.as_bytes(),
            self.n,
        )
    }
}

#[derive(Deserialize, Clone, Debug)]
struct ValueNPGetText {
    ctx: String,
    singular: String,
    plural: String,
    n: u32,
}

impl ToString for ValueNPGetText {
    fn to_string(&self) -> String {
        gettextrs::npgettext(
            self.ctx.as_bytes(),
            self.singular.as_bytes(),
            self.plural.as_bytes(),
            self.n,
        )
    }
}

#[derive(Deserialize, Clone, Debug)]
struct ValueDCNGetText {
    domain: String,
    singular: String,
    plural: String,
    n: u32,
    category: LocaleCategory,
}

#[derive(Deserialize, Debug, PartialEq, Clone, Copy)]
#[allow(clippy::enum_variant_names)]
enum LocaleCategory {
    #[serde(rename = "ctype")]
    LcCType = 0,
    #[serde(rename = "numeric")]
    LcNumeric = 1,
    #[serde(rename = "time")]
    LcTime = 2,
    #[serde(rename = "collate")]
    LcCollate = 3,
    #[serde(rename = "monetary")]
    LcMonetary = 4,
    #[serde(rename = "messages")]
    LcMessages = 5,
    #[serde(rename = "all")]
    LcAll = 6,
    #[serde(rename = "paper")]
    LcPaper = 7,
    #[serde(rename = "name")]
    LcName = 8,
    #[serde(rename = "address")]
    LcAddress = 9,
    #[serde(rename = "telephone")]
    LcTelephone = 10,
    #[serde(rename = "measurement")]
    LcMeasurement = 11,
    #[serde(rename = "identification")]
    LcIdentification = 12,
}

impl std::convert::From<LocaleCategory> for gettextrs::LocaleCategory {
    fn from(category: LocaleCategory) -> Self {
        unsafe { std::mem::transmute(category) }
    }
}

impl ToString for ValueDCNGetText {
    fn to_string(&self) -> String {
        gettextrs::dcngettext(
            self.domain.as_bytes(),
            self.singular.as_bytes(),
            self.plural.as_bytes(),
            self.n,
            self.category.into(),
        )
    }
}
