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
    /*
    PGetText,
    DGetText,
    DNGetText,
    NPGetText,
    DCNGetText,
    */
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
pub struct DatetimeValue {
    strftime: String,
    epoch: i64,
}

impl DatetimeValue {
    pub fn to_string(&self) -> String {
        strftime_local(&self.strftime, self.epoch)
    }
}
