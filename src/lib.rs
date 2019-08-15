#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate derive_error;

use dynfmt::{Format, PythonFormat};
use libc_strftime::strftime_local;
#[allow(unused_imports)]
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Error)]
pub enum Error {
    /// Formatting error
    #[error(msg_embedded, no_from, non_std)]
    FormatError(String),
    /// Missing join separator
    #[error(non_std, no_from, display = "missing join separator")]
    MissingJoinSeparator,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum SerdeGetText {
    Text(String),
    Integer(i64),
    Float(f64),
    Bool(bool),
    Unit(()),
    Array(Vec<SerdeGetText>),
    FormattedText {
        text: SerdeGetTextText,
        args: Option<Formatter>,
    },
    Datetime(DatetimeValue),
    GetText {
        gettext: SerdeGetTextGetText,
        args: Option<Formatter>,
    },
    NGetText {
        ngettext: SerdeGetTextNGetText,
        args: Option<Formatter>,
    },
    PGetText {
        pgettext: SerdeGetTextPGetText,
        args: Option<Formatter>,
    },
    DGetText {
        dgettext: SerdeGetTextDGetText,
        args: Option<Formatter>,
    },
    DNGetText {
        dngettext: SerdeGetTextDNGetText,
        args: Option<Formatter>,
    },
    NPGetText {
        npgettext: SerdeGetTextNPGetText,
        args: Option<Formatter>,
    },
    DCNGetText {
        dcngettext: SerdeGetTextDCNGetText,
        args: Option<Formatter>,
    },
}

impl SerdeGetText {
    pub fn to_string(&self) -> Result<String, Error> {
        let (s, args) = match self {
            SerdeGetText::Text(x) => (x.to_string(), &None),
            SerdeGetText::Integer(x) => (x.to_string(), &None),
            SerdeGetText::Float(x) => (x.to_string(), &None),
            SerdeGetText::Bool(x) => (
                if *x {
                    gettextrs::gettext(b"yes" as &[u8])
                } else {
                    gettextrs::gettext(b"no" as &[u8])
                },
                &None,
            ),
            SerdeGetText::Unit(()) => (gettextrs::gettext(b"n/a" as &[u8]), &None),
            SerdeGetText::Array(xs) => (
                {
                    let sep = match xs.get(0) {
                        Some(x) => x.to_string(),
                        None => Err(Error::MissingJoinSeparator),
                    }?;

                    let mut vec = Vec::new();

                    for value in xs.iter().skip(1) {
                        vec.push(value.to_string()?);
                    }

                    vec.join(&sep)
                },
                &None,
            ),
            SerdeGetText::FormattedText { text, args } => (text.translate(), args),
            SerdeGetText::Datetime(x) => (x.to_string(), &None),
            SerdeGetText::GetText { gettext, args } => (gettext.translate(), args),
            SerdeGetText::NGetText { ngettext, args } => (ngettext.translate(), args),
            SerdeGetText::PGetText { pgettext, args } => (pgettext.translate(), args),
            SerdeGetText::DGetText { dgettext, args } => (dgettext.translate(), args),
            SerdeGetText::DNGetText { dngettext, args } => (dngettext.translate(), args),
            SerdeGetText::NPGetText { npgettext, args } => (npgettext.translate(), args),
            SerdeGetText::DCNGetText { dcngettext, args } => (dcngettext.translate(), args),
        };

        Ok(args
            .as_ref()
            .map(|x| x.format(&s))
            .transpose()?
            .unwrap_or(s))
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum Formatter {
    KeywordArgs(HashMap<String, SerdeGetText>),
    PositionalArgs(Vec<SerdeGetText>),
}

impl Formatter {
    pub fn format(&self, message: &str) -> Result<String, Error> {
        match self {
            Formatter::KeywordArgs(kwargs) => {
                let mut args = HashMap::new();

                for (key, value) in kwargs.iter() {
                    args.insert(key.clone(), value.to_string()?);
                }

                PythonFormat
                    .format(message, args)
                    .map_err(|err| Error::FormatError(format!("{}", err)))
                    .map(|x| x.to_string())
            }
            Formatter::PositionalArgs(args) => PythonFormat
                .format(
                    message,
                    args.iter()
                        .map(|x| x.to_string())
                        .collect::<Result<Vec<_>, _>>()?,
                )
                .map_err(|err| Error::FormatError(format!("{}", err)))
                .map(|x| x.to_string()),
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct DatetimeValue {
    strftime: String,
    epoch: i64,
}

impl DatetimeValue {
    pub fn to_string(&self) -> String {
        strftime_local(&self.strftime, self.epoch)
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct SerdeGetTextText(String);

impl SerdeGetTextText {
    pub fn translate(&self) -> String {
        self.0.clone()
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct SerdeGetTextGetText(String);

impl SerdeGetTextGetText {
    pub fn translate(&self) -> String {
        gettextrs::gettext(self.0.as_bytes())
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct SerdeGetTextNGetText {
    singular: String,
    plural: String,
    n: u32,
}

impl SerdeGetTextNGetText {
    pub fn translate(&self) -> String {
        gettextrs::ngettext(self.singular.as_bytes(), self.plural.as_bytes(), self.n)
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct SerdeGetTextPGetText {
    ctx: String,
    msgid: String,
}

impl SerdeGetTextPGetText {
    pub fn translate(&self) -> String {
        gettextrs::pgettext(self.ctx.as_bytes(), self.msgid.as_bytes())
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct SerdeGetTextDGetText {
    domain: String,
    msgid: String,
}

impl SerdeGetTextDGetText {
    pub fn translate(&self) -> String {
        gettextrs::dgettext(self.domain.as_bytes(), self.msgid.as_bytes())
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct SerdeGetTextDNGetText {
    domain: String,
    singular: String,
    plural: String,
    n: u32,
}

impl SerdeGetTextDNGetText {
    pub fn translate(&self) -> String {
        gettextrs::dngettext(
            self.domain.as_bytes(),
            self.singular.as_bytes(),
            self.plural.as_bytes(),
            self.n,
        )
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct SerdeGetTextNPGetText {
    ctx: String,
    singular: String,
    plural: String,
    n: u32,
}

impl SerdeGetTextNPGetText {
    pub fn translate(&self) -> String {
        gettextrs::npgettext(
            self.ctx.as_bytes(),
            self.singular.as_bytes(),
            self.plural.as_bytes(),
            self.n,
        )
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct SerdeGetTextDCNGetText {
    domain: String,
    singular: String,
    plural: String,
    n: u32,
    category: LocaleCategory,
}

/// Locale category enum ported from locale.h
#[derive(Deserialize, Debug, PartialEq, Clone, Copy)]
pub enum LocaleCategory {
    /// Character classification and case conversion.
    LcCType = 0,
    /// Non-monetary numeric formats.
    LcNumeric = 1,
    /// Date and time formats.
    LcTime = 2,
    /// Collation order.
    LcCollate = 3,
    /// Monetary formats.
    LcMonetary = 4,
    /// Formats of informative and diagnostic messages and interactive responses.
    LcMessages = 5,
    /// For all.
    LcAll = 6,
    /// Paper size.
    LcPaper = 7,
    /// Name formats.
    LcName = 8,
    /// Address formats and location information.
    LcAddress = 9,
    /// Telephone number formats.
    LcTelephone = 10,
    /// Measurement units (Metric or Other).
    LcMeasurement = 11,
    /// Metadata about the locale information.
    LcIdentification = 12,
}

impl std::convert::From<LocaleCategory> for gettextrs::LocaleCategory {
    fn from(category: LocaleCategory) -> Self {
        unsafe { std::mem::transmute(category) }
    }
}

impl SerdeGetTextDCNGetText {
    pub fn translate(&self) -> String {
        gettextrs::dcngettext(
            self.domain.as_bytes(),
            self.singular.as_bytes(),
            self.plural.as_bytes(),
            self.n,
            self.category.into(),
        )
    }
}
