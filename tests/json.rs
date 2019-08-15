#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use libc_strftime::{set_locale, tz_set};
use serde::Deserialize;
use serde_gettext::*;

#[derive(Deserialize, Clone)]
#[serde(untagged)]
enum CustomMessage {
    CustomBool(bool),
    Custom { custom: String },
    SerdeGetText(SerdeGetText),
}

impl CustomMessage {
    fn to_string(&self) -> String {
        match self {
            CustomMessage::SerdeGetText(x) => x.to_string().unwrap(),
            CustomMessage::Custom { custom } => format!("Custom: {}", custom),
            CustomMessage::CustomBool(x) => format!("Custom: {:?}", x),
        }
    }
}

#[test]
fn custom_message() {
    let j = json!({
        "text": "Hello!",
    });
    let message = CustomMessage::deserialize(&j).unwrap();
    assert_eq!(message.to_string(), "Hello!");

    let j = json!({
        "custom": "Hello!",
    });
    let message = CustomMessage::deserialize(&j).unwrap();
    assert_eq!(message.to_string(), "Custom: Hello!");

    let j = json!(true);
    let message = CustomMessage::deserialize(&j).unwrap();
    assert_eq!(message.to_string(), "Custom: true");
}

#[test]
fn not_translated_text() {
    let j = json!({
        "text": "Hello!",
    });
    let message = SerdeGetText::deserialize(&j).unwrap();
    assert_eq!(message.to_string().unwrap(), "Hello!");
}

#[test]
fn not_translated_text_with_args() {
    let j = json!({
        "text": "Hello %s!",
        "args": ["Grace"],
    });
    let message = SerdeGetText::deserialize(&j).unwrap();
    assert_eq!(message.to_string().unwrap(), "Hello Grace!");
}

#[test]
fn not_translated_text_with_kwargs() {
    let j = json!({
        "text": "Hello %(name)s!",
        "args": {"name": "Grace"},
    });
    let message = SerdeGetText::deserialize(&j).unwrap();
    assert_eq!(message.to_string().unwrap(), "Hello Grace!");
}

#[test]
fn datetime() {
    tz_set();
    set_locale();

    #[allow(clippy::unreadable_literal)]
    let j = json!({
        "strftime": "It is now: %c",
        "epoch": 1565854615,
    });
    let message = SerdeGetText::deserialize(&j).unwrap();
    assert_eq!(
        message.to_string().unwrap(),
        "It is now: Thu 15 Aug 2019 09:36:55 CEST"
    );
}

#[test]
fn gettext() {
    let j = json!({
        "gettext": "Hello!",
    });
    let message = SerdeGetText::deserialize(&j).unwrap();
    assert_eq!(message.to_string().unwrap(), "Hello!");
}

#[test]
fn gettext_with_args() {
    let j = json!({
        "gettext": "Hello %(name)s!",
        "args": {"name": "Grace"},
    });
    let message = SerdeGetText::deserialize(&j).unwrap();
    assert_eq!(message.to_string().unwrap(), "Hello Grace!");
}

#[test]
fn gettext_with_args_integer() {
    let j = json!({
        "gettext": "The answer is: %(answer)s",
        "args": {
            "answer": 42,
        },
    });
    let message = SerdeGetText::deserialize(&j).unwrap();
    assert_eq!(message.to_string().unwrap(), "The answer is: 42");
}

#[test]
fn gettext_with_args_float() {
    #[allow(clippy::approx_constant)]
    let j = json!({
        "gettext": "Pi is: %(pi)s",
        "args": {
            "pi": 3.14,
        },
    });
    let message = SerdeGetText::deserialize(&j).unwrap();
    assert_eq!(message.to_string().unwrap(), "Pi is: 3.14");
}

#[test]
fn gettext_with_args_bool() {
    let j = json!({
        "gettext": "The answer is: %(answer)s",
        "args": {
            "answer": true,
        },
    });
    let message = SerdeGetText::deserialize(&j).unwrap();
    assert_eq!(message.to_string().unwrap(), "The answer is: yes");
}

#[test]
fn gettext_with_args_null() {
    let j = json!({
        "gettext": "The answer is: %(answer)s",
        "args": {
            "answer": (),
        },
    });
    let message = SerdeGetText::deserialize(&j).unwrap();
    assert_eq!(message.to_string().unwrap(), "The answer is: n/a");
}

#[test]
fn gettext_with_args_array() {
    let j = json!({
        "gettext": "%(greetings)s",
        "args": {
            "greetings": [", ", "Hello", "World!"],
        },
    });
    let message = SerdeGetText::deserialize(&j).unwrap();
    assert_eq!(message.to_string().unwrap(), "Hello, World!");
}

#[test]
fn gettext_with_args_i18n() {
    let j = json!({
        "gettext": "%(greetings)s",
        "args": {
            "greetings": {
                "gettext": "Hello!",
            },
        },
    });
    let message = SerdeGetText::deserialize(&j).unwrap();
    assert_eq!(message.to_string().unwrap(), "Hello!");
}

#[test]
fn gettext_with_args_recursive() {
    let j = json!({
        "gettext": "%(greetings)s",
        "args": {
            "greetings": {
                "gettext": "Hello %(name)s!",
                "args": {"name": "Grace"},
            },
        },
    });
    let message = SerdeGetText::deserialize(&j).unwrap();
    assert_eq!(message.to_string().unwrap(), "Hello Grace!");
}

#[test]
fn ngettext_singular_with_args() {
    let j = json!({
        "ngettext": {
            "singular": "%s element",
            "plural": "%s elements",
            "n": 1,
        },
        "args": [1],
    });
    let message = SerdeGetText::deserialize(&j).unwrap();
    assert_eq!(message.to_string().unwrap(), "1 element");
}

#[test]
fn ngettext_plural_with_args() {
    let j = json!({
        "ngettext": {
            "singular": "%s element",
            "plural": "%s elements",
            "n": 2,
        },
        "args": [2],
    });
    let message = SerdeGetText::deserialize(&j).unwrap();
    assert_eq!(message.to_string().unwrap(), "2 elements");
}
