use regex::Regex;
use serde_json::{self, Value};
use std::collections::HashMap;
use std::error::Error;

/// Parses JTL content into a structured vector.
pub fn parse(text: &str) -> Result<Vec<Value>, Box<dyn Error>> {
    let mut result: Vec<Value> = Vec::new();
    let lines: Vec<&str> = text.lines().collect();

    if lines.is_empty() || !lines[0].contains("DOCTYPE=JTL") {
        return Err("invalid JTL document: missing DOCTYPE".into());
    }

    let mut in_body = false;
    let mut in_env = false;
    let mut current_env: HashMap<String, String> = HashMap::new();

    for line in lines {
        let line = line.trim();
        if line.is_empty()
            || line.starts_with("/*")
            || line.starts_with("*/")
            || line.starts_with(">//>")
        {
            continue;
        }

        if line == ">>>ENV;" {
            in_env = true;
            continue;
        }
        if line == ">>>BEGIN;" {
            in_env = false;
            in_body = true;
            continue;
        }
        if line == ">>>END;" {
            in_body = false;
            continue;
        }

        // Handle multiple declarations per line.
        let declarations: Vec<&str> = line.split(';').collect();
        for decl in declarations {
            let decl = decl.trim();
            if decl.is_empty() || decl.starts_with(">//>") {
                continue;
            }

            if in_env && decl.starts_with(">>>") {
                let content = &decl[3..];
                if let Some(eq_index) = content.find('=') {
                    let var_name = content[..eq_index].trim();
                    let var_value = content[eq_index + 1..].trim();
                    current_env.insert(var_name.to_string(), var_value.to_string());
                }
            } else if in_body && decl.starts_with('>') {
                if decl.len() < 5 {
                    return Err("invalid element format: too short".into());
                }
                let element_map = parse_element(decl, &current_env)?;
                result.push(Value::Object(element_map));
            }
        }
    }

    Ok(result)
}

/// Converts a vector to a JSON string.
pub fn stringify(data: &Vec<Value>) -> Result<String, serde_json::Error> {
    serde_json::to_string(data)
}

/// Parses a single JTL element.
fn parse_element(line: &str, env: &HashMap<String, String>) -> Result<serde_json::Map<String, Value>, Box<dyn Error>> {
    let line = line
        .strip_prefix('>')
        .ok_or("invalid element format: missing '>' prefix")?;

    if !line.contains('>') {
        return Err("invalid element format: missing separator".into());
    }

    let attr_regex = Regex::new(r#"(\w+)="([^"]+)""#)?;
    let captures: Vec<_> = attr_regex.captures_iter(line).collect();
    if captures.is_empty() {
        return Err("invalid element format: no attributes found".into());
    }

    let mut element_map = serde_json::Map::new();
    for cap in captures {
        let key = cap.get(1).unwrap().as_str();
        let value = cap.get(2).unwrap().as_str();
        element_map.insert(key.to_string(), Value::String(value.to_string()));
    }

    // Find the first occurrence of '>' to separate attributes from content.
    let content_start = line
        .find('>')
        .ok_or("invalid element format: missing content separator")?;
    let mut content_part = &line[content_start + 1..];

    // Remove a trailing semicolon, if present.
    if content_part.ends_with(';') {
        content_part = &content_part[..content_part.len() - 1];
    }

    let parts: Vec<&str> = content_part.splitn(2, '>').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return Err("invalid element format: malformed content".into());
    }
    let id = parts[0].to_string();
    let mut content = parts[1].to_string();

    // Replace environment variable if needed.
    if content.starts_with("$env:") {
        let env_var = content.trim_start_matches("$env:");
        if let Some(val) = env.get(env_var) {
            content = val.clone();
        }
    }
    element_map.insert("KEY".to_string(), Value::String(id));
    element_map.insert("Content".to_string(), Value::String(content.clone()));
    element_map.insert("Contents".to_string(), Value::String(content));

    Ok(element_map)
}

/// Extracts environment variables from JTL text.
pub fn parse_env(text: &str) -> Result<HashMap<String, Value>, Box<dyn Error>> {
    let mut env_map: HashMap<String, Value> = HashMap::new();
    let lines: Vec<&str> = text.lines().collect();

    if lines.is_empty() || !lines[0].contains("DOCTYPE=JTL") {
        return Err("invalid JTL document: missing DOCTYPE".into());
    }

    let mut in_env = false;
    for line in lines {
        let line = line.trim();
        if line.is_empty()
            || line.starts_with("/*")
            || line.starts_with("*/")
            || line.starts_with(">//>")
        {
            continue;
        }

        if line == ">>>ENV;" {
            in_env = true;
            continue;
        }
        if line == ">>>BEGIN;" {
            break;
        }

        if in_env && line.starts_with(">>>") {
            let declarations: Vec<&str> = line.split(';').collect();
            for decl in declarations {
                let decl = decl.trim();
                if decl.starts_with(">>>") {
                    let content = &decl[3..];
                    if let Some(eq_index) = content.find('=') {
                        let var_name = content[..eq_index].trim();
                        let var_value = content[eq_index + 1..].trim();
                        env_map.insert(var_name.to_string(), Value::String(var_value.to_string()));
                    }
                }
            }
        }
    }

    Ok(env_map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    const SAMPLE_JTL: &str = r#"DOCTYPE=JTL
>>>ENV;
>>>foo=bar;
>>>BEGIN;
>element_id key="value">element_id>$env:foo;
>>>END;"#;

    #[test]
    fn test_parse() {
        let parsed = parse(SAMPLE_JTL).expect("Parsing should succeed");
        assert!(!parsed.is_empty());

        // Check that the parsed element contains the expected fields.
        let element = parsed.get(0).unwrap();
        let obj = element.as_object().expect("Element should be an object");
        assert_eq!(obj.get("key").unwrap(), "value");
        assert_eq!(obj.get("Content").unwrap(), "bar");
        assert_eq!(obj.get("Contents").unwrap(), "bar");
    }

    #[test]
    fn test_parse_env() {
        let env_vars = parse_env(SAMPLE_JTL).expect("Parsing env should succeed");
        assert!(env_vars.contains_key("foo"));
        assert_eq!(env_vars.get("foo").unwrap(), "bar");
    }

    #[test]
    fn test_stringify() {
        // Create a sample vector.
        let mut element = serde_json::Map::new();
        element.insert("key".to_string(), Value::String("value".to_string()));
        element.insert("content".to_string(), Value::String("bar".to_string()));
        let vec = vec![Value::Object(element)];

        let json_str = stringify(&vec).expect("Stringify should succeed");
        // Parse the JSON string back and check the field.
        let parsed_json: Value = serde_json::from_str(&json_str).expect("JSON should be valid");
        assert!(parsed_json.get(0).is_some());
    }

    #[test]
    fn test_missing_doctype() {
        let invalid_jtl = r#"No DOCTYPE here
>>>ENV;
>>>foo=bar;
>>>BEGIN;
>element_id key="value">element_id>$env:foo;
>>>END;"#;

        let err = parse(invalid_jtl).unwrap_err();
        assert_eq!(err.to_string(), "invalid JTL document: missing DOCTYPE");
    }

    #[test]
    fn test_element_too_short() {
        // An element line that is too short.
        let jtl = r#"DOCTYPE=JTL
>>>BEGIN;
>a;
>>>END;"#;
        let err = parse(jtl).unwrap_err();
        assert_eq!(err.to_string(), "invalid element format: too short");
    }
}
