#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use libc_strftime::{set_locale, tz_set};
use serde::Deserialize;
use serde_gettext::*;
use std::convert::TryFrom;

#[derive(Deserialize, Clone)]
#[serde(untagged)]
enum CustomMessage {
    CustomBool(bool),
    Custom { custom: String },
    SerdeGetText(SerdeGetText),
}

impl CustomMessage {
    fn to_string(self) -> String {
        match self {
            CustomMessage::SerdeGetText(x) => String::try_from(x).unwrap(),
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
    assert_eq!(String::try_from(message).unwrap(), "Hello!");
}

#[test]
fn not_translated_text_with_args() {
    let j = json!({
        "text": "Hello %s!",
        "args": ["Grace"],
    });
    let message = SerdeGetText::deserialize(&j).unwrap();
    assert_eq!(String::try_from(message).unwrap(), "Hello Grace!");
}

#[test]
fn not_translated_text_with_kwargs() {
    let j = json!({
        "text": "Hello %(name)s!",
        "args": {"name": "Grace"},
    });
    let message = SerdeGetText::deserialize(&j).unwrap();
    assert_eq!(String::try_from(message).unwrap(), "Hello Grace!");
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
        String::try_from(message).unwrap(),
        "It is now: Thu 15 Aug 2019 09:36:55 CEST"
    );
}

#[test]
fn gettext() {
    let j = json!({
        "gettext": "Hello!",
    });
    let message = SerdeGetText::deserialize(&j).unwrap();
    assert_eq!(String::try_from(message).unwrap(), "Hello!");
}

#[test]
fn gettext_with_args() {
    let j = json!({
        "gettext": "Hello %(name)s!",
        "args": {"name": "Grace"},
    });
    let message = SerdeGetText::deserialize(&j).unwrap();
    assert_eq!(String::try_from(message).unwrap(), "Hello Grace!");
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
    assert_eq!(String::try_from(message).unwrap(), "The answer is: 42");
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
    assert_eq!(String::try_from(message).unwrap(), "Pi is: 3.14");
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
    assert_eq!(String::try_from(message).unwrap(), "The answer is: yes");
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
    assert_eq!(String::try_from(message).unwrap(), "The answer is: n/a");
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
    assert_eq!(String::try_from(message).unwrap(), "Hello, World!");
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
    assert_eq!(String::try_from(message).unwrap(), "Hello!");
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
    assert_eq!(String::try_from(message).unwrap(), "Hello Grace!");
}

#[test]
fn ngettext_singular_without_args() {
    let j = json!({
        "ngettext": {
            "singular": "%(n)s element",
            "plural": "%(n)s elements",
            "n": 1,
        },
    });
    let message = SerdeGetText::deserialize(&j).unwrap();
    assert_eq!(String::try_from(message).unwrap(), "1 element");
}

#[test]
fn ngettext_plural_with_args() {
    let j = json!({
        "ngettext": {
            "singular": "%(n)s element (success: %(success)s)",
            "plural": "%(n)s elements (success: %(success)s)",
            "n": 2,
        },
        "args": {
            "success": true,
        },
    });
    let message = SerdeGetText::deserialize(&j).unwrap();
    assert_eq!(
        String::try_from(message).unwrap(),
        "2 elements (success: yes)"
    );
}

#[test]
fn dcngettext() {
    let j = json!({
        "dcngettext": {
            "domain": "some_domain",
            "singular": "%(n)s element",
            "plural": "%(n)s elements",
            "n": 2,
            "category": "measurement",
        },
    });
    let message = SerdeGetText::deserialize(&j).unwrap();
    assert_eq!(String::try_from(message).unwrap(), "2 elements");
}

#[test]
fn base_args() {
    let j = json!({
        "text": "Hello %(name)s!",
    });
    let mut message = SerdeGetText::deserialize(&j).unwrap();
    message.args.insert("name".to_string(), "Grace".to_string());
    assert_eq!(String::try_from(message).unwrap(), "Hello Grace!");
}

#[test]
fn base_args_union_order() {
    let j = json!({
        "text": "Hello %(name)s!",
        "args": {"name": "Marie"},
    });
    let mut message = SerdeGetText::deserialize(&j).unwrap();
    message.args.insert("name".to_string(), "Grace".to_string());
    assert_eq!(String::try_from(message).unwrap(), "Hello Marie!");
}
