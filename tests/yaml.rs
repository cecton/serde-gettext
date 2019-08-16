#[macro_use]
extern crate serde_derive;

use libc_strftime::{set_locale, tz_set};
#[allow(unused_imports)]
use serde::Deserialize;
use serde_gettext::*;
use std::convert::TryFrom;

#[derive(Deserialize, Clone)]
#[serde(untagged)]
enum CustomMessage {
    CustomBool(bool),
    Custom { custom: String },
    SerdeGetText(Box<SerdeGetText>),
}

impl CustomMessage {
    fn into_string(self) -> String {
        match self {
            CustomMessage::SerdeGetText(x) => String::try_from(x).unwrap(),
            CustomMessage::Custom { custom } => format!("Custom: {}", custom),
            CustomMessage::CustomBool(x) => format!("Custom: {:?}", x),
        }
    }
}

fn to_string(s: &str) -> String {
    String::try_from(serde_yaml::from_str::<SerdeGetText>(s).expect("could not parse yaml"))
        .expect("could not translate")
}

#[test]
fn custom_message() {
    let message = serde_yaml::from_str::<CustomMessage>(
        r#"---
text: Hello!
"#,
    )
    .unwrap();
    assert_eq!(message.into_string(), "Hello!");

    let message = serde_yaml::from_str::<CustomMessage>(
        r#"---
custom: Hello!
"#,
    )
    .unwrap();
    assert_eq!(message.into_string(), "Custom: Hello!");

    let message = serde_yaml::from_str::<CustomMessage>(
        r#"---
true
"#,
    )
    .unwrap();
    assert_eq!(message.into_string(), "Custom: true");
}

#[test]
fn not_translated_text() {
    assert_eq!(
        to_string(
            r#"---
text: Hello!
"#
        ),
        "Hello!"
    );
}

#[test]
fn not_translated_text_with_args() {
    assert_eq!(
        to_string(
            r#"---
text: "Hello %s!"
args:
    - Grace
"#
        ),
        "Hello Grace!"
    );
}

#[test]
fn not_translated_text_with_kwargs() {
    assert_eq!(
        to_string(
            r#"---
text: "Hello %(name)s!"
args:
    name: Grace
"#
        ),
        "Hello Grace!"
    );
}

#[test]
fn datetime() {
    tz_set();
    set_locale();

    assert_eq!(
        to_string(
            r#"---
strftime: "It is now: %c"
epoch: 1565854615
"#
        ),
        "It is now: Thu 15 Aug 2019 09:36:55 CEST"
    );
}

#[test]
fn gettext() {
    assert_eq!(
        to_string(
            r#"---
gettext: "Hello!"
"#
        ),
        "Hello!"
    );
}

#[test]
fn gettext_with_args() {
    assert_eq!(
        to_string(
            r#"---
gettext: "Hello %(name)s!"
args:
    name: Grace
"#
        ),
        "Hello Grace!"
    );
}

#[test]
fn gettext_with_args_integer() {
    assert_eq!(
        to_string(
            r#"---
gettext: "The answer is: %(answer)s"
args:
    answer: 42
"#
        ),
        "The answer is: 42"
    );
}

#[test]
fn gettext_with_args_float() {
    assert_eq!(
        to_string(
            r#"---
gettext: "Pi is: %(pi)s"
args:
    pi: 3.14
"#
        ),
        "Pi is: 3.14"
    );
}

#[test]
fn gettext_with_args_bool() {
    assert_eq!(
        to_string(
            r#"---
gettext: "The answer is: %(answer)s"
args:
    answer: true
"#
        ),
        "The answer is: yes"
    );
}

#[test]
fn gettext_with_args_null() {
    assert_eq!(
        to_string(
            r#"---
gettext: "The answer is: %(answer)s"
args:
    answer:
"#
        ),
        "The answer is: n/a"
    );
}

#[test]
fn gettext_with_args_array() {
    assert_eq!(
        to_string(
            r#"---
gettext: "%(greetings)s"
args:
    greetings:
        - ", "
        - Hello
        - World!
"#
        ),
        "Hello, World!"
    );
}

#[test]
fn gettext_with_args_i18n() {
    assert_eq!(
        to_string(
            r#"---
gettext: "%(greetings)s"
args:
    greetings:
        gettext: Hello!
"#
        ),
        "Hello!"
    );
}

#[test]
fn gettext_with_args_recursive() {
    assert_eq!(
        to_string(
            r#"---
gettext: "%(greetings)s"
args:
    greetings:
        gettext: "Hello %(name)s!"
        args:
            name: Grace
"#
        ),
        "Hello Grace!"
    );
}

#[test]
fn ngettext_singular_with_args() {
    assert_eq!(
        to_string(
            r#"---
ngettext:
    singular: "%(n)s element"
    plural: "%(n)s elements"
    n: 1
"#
        ),
        "1 element"
    );
}

#[test]
fn ngettext_plural_with_args() {
    assert_eq!(
        to_string(
            r#"---
ngettext:
    singular: "%(n)s element"
    plural: "%(n)s elements"
    n: 2
"#,
        ),
        "2 elements"
    );
}

#[test]
fn base_args() {
    let s = r#"---
text: "Hello %(name)s!"
"#;
    let mut message = serde_yaml::from_str::<SerdeGetText>(s).expect("could not parse yaml");
    message.args.insert("name".to_string(), "Grace".to_string());
    assert_eq!(
        String::try_from(message).expect("could not translate"),
        "Hello Grace!"
    );
}
