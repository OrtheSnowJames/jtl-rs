This project is licensed under the MIT License. See the [LICENSE](./LICENSE) file for more details.

# JTL

This is a port from the go version of jtl. JTL is a simple tag language with a "nicer" syntax. I made this because I thought xml was too boilerplate. Efficency. 

To start with this, first add jtl to your project.
```sh
cargo add jtl
```

Second, use it.
```rust
use jtl;
```

Third, do something with it.

```rust
fn test_parse() {
    const SAMPLE_JTL: &str = r#"DOCTYPE=JTL
>>>ENV;
>>>foo=bar;
>>>BEGIN;
>key="value">element_id>$env:foo;
>>>END;"#;
    let parsed = parse(SAMPLE_JTL).expect("Parsing should succeed");
    assert!(!parsed.is_empty());

    // Check that the parsed element contains the expected fields.
    let element = parsed.get(0).unwrap();
    let obj = element.as_object().expect("Element should be an object");
    assert_eq!(obj.get("key").unwrap(), "value");
    assert_eq!(obj.get("KEY").unwrap(), "element_id");
    assert_eq!(obj.get("Content").unwrap(), "bar");
    assert_eq!(obj.get("Contents").unwrap(), "bar");
}
```

Latest Commit: Refactored to use vec with all of them instead of using key value inside a hashmap ex:
from:
HashMap String, Value {
    "foo": {
        "Content (or Contents)": "20"
    }
}

to:
Vec Value [
    {
        "key": "foo",
        "Content (or Contents)": "20"
    }
]
.

Tests Passed: As of 2/22/2025, 2nd commit
