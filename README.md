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
>element_id key="value">element_id>$env:foo;
>>>END;"#;
        let parsed = parse(SAMPLE_JTL).expect("Parsing should succeed");
        assert!(parsed.contains_key("element_id"));

        // Check that the parsed element contains the expected fields.
        let element = parsed.get("element_id").unwrap();
        let obj = element.as_object().expect("Element should be an object");
        assert_eq!(obj.get("key").unwrap(), "value");
        assert_eq!(obj.get("content").unwrap(), "bar");
    }
```

Latest Commit: Init the project.
Tests Passed: As of 2/22/2025
