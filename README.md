Introduction
============

This library is only a generic deserializer/API for gettext. With this you can
use JSON or YAML (or "any" format handled by serde) to translate text through
gettext and even format. It also has an API for strftime for formatting dates.

You can use it in an API service to have a translation endpoint or in a lambda
to translate the input.

 *  Example in JSON

    ```json
    {
        "ngettext": {
            "singular": "One item has been deleted",
            "plural": "%(n)s items have been deleted",
            "n": 5,
        }
    }
    ```

 *  Example in YAML

    ```yaml
    ngettext:
        singular: One item has been deleted
        plural: "%(n)s items have been deleted"
        n: 5
    ```

When the structure is deserialized, you can simply convert it to a translated
`String`:

```rust
use serde_gettext::SerdeGetText;
use std::convert::TryFrom;

let yaml = r#"---
ngettext:
    singular: One item has been deleted
    plural: "%(n)s items have been deleted"
    n: 5
"#;
let s: SerdeGetText = serde_yaml::from_str(yaml).unwrap();

assert_eq!(String::try_from(s).unwrap(), "5 items have been deleted");
```

Formatting
==========

 *  Example in JSON

    ```json
    {
        "gettext": "Hello %(name)s!",
        "args": {
            "name": "Grace",
        }
    }
    ```

 *  Example in YAML

    ```yaml
    gettext: "Hello %(name)s!"
    args:
        name: Grace
    ```

List of All Available Functions
===============================

 *  gettext:

    ```yaml
    gettext: "msgid"
    ```

 *  ngettext:

    ```yaml
    ngettext:
        singular: "msgid_singular"
        plural: "msgid_singular"
        n: 5
    ```

 *  pgettext:

    ```yaml
    pgettext:
        ctx: "context"
        msgid: "msgid"
    ```

 *  dgettext:

    ```yaml
    dgettext:
        domain: "domain"
        msgid: "msgid"
    ```

 *  dngettext:

    ```yaml
    dngettext:
        domain: "domain"
        singular: "msgid_singular"
        plural: "msgid_singular"
        n: 5
    ```

 *  npgettext:

    ```yaml
    npgettext:
        ctx: "context"
        singular: "msgid_singular"
        plural: "msgid_singular"
        n: 5
    ```

 *  dcngettext:

    ```yaml
    dcngettext:
        domain: "domain"
        singular: "msgid_singular"
        plural: "msgid_singular"
        n: 5
        cateogy: "ctype|numeric|time|collate|monetary|messages|all|paper|name|address|telephone|measurement|identification"
    ```
