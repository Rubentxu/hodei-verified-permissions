//! Advanced claim value transformations
//!
//! This module provides various transformations that can be applied to claim values
//! during JWT validation and claims mapping.

use regex::Regex;
use serde::{Deserialize, Serialize};

/// Value transformation operations
///
/// These transformations can be chained together to extract and manipulate
/// claim values in flexible ways.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum ValueTransform {
    /// No transformation
    None,
    /// Split by separator and take the last part
    SplitLast(String),
    /// Capture group from regex match
    RegexCapture {
        pattern: String,
        group: usize,
    },
    /// Replace using regex pattern
    RegexReplace {
        pattern: String,
        replacement: String,
    },
    /// Add prefix to value
    Prefix(String),
    /// Add suffix to value
    Suffix(String),
    /// Convert to lowercase
    Lowercase,
    /// Convert to uppercase
    Uppercase,
    /// Trim whitespace
    Trim,
    /// Chain multiple transformations
    Chain(Vec<ValueTransform>),
}

impl ValueTransform {
    /// Apply the transformation to a value
    ///
    /// # Arguments
    ///
    /// * `value` - The input value to transform
    ///
    /// # Returns
    ///
    /// The transformed value or an error if transformation fails
    pub fn apply(&self, value: &str) -> Result<String, String> {
        match self {
            ValueTransform::None => Ok(value.to_string()),

            ValueTransform::SplitLast(sep) => {
                Ok(value
                    .split(sep)
                    .last()
                    .unwrap_or(value)
                    .to_string())
            }

            ValueTransform::RegexCapture { pattern, group } => {
                let re = Regex::new(pattern)
                    .map_err(|e| format!("Invalid regex pattern: {}", e))?;

                re.captures(value)
                    .and_then(|caps| caps.get(*group).map(|m| m.as_str().to_string()))
                    .ok_or_else(|| {
                        format!("No match found for pattern '{}' in value '{}'", pattern, value)
                    })
            }

            ValueTransform::RegexReplace { pattern, replacement } => {
                let re = Regex::new(pattern)
                    .map_err(|e| format!("Invalid regex pattern: {}", e))?;

                Ok(re.replace_all(value, replacement).to_string())
            }

            ValueTransform::Prefix(prefix) => {
                Ok(format!("{}{}", prefix, value))
            }

            ValueTransform::Suffix(suffix) => {
                Ok(format!("{}{}", value, suffix))
            }

            ValueTransform::Lowercase => {
                Ok(value.to_lowercase())
            }

            ValueTransform::Uppercase => {
                Ok(value.to_uppercase())
            }

            ValueTransform::Trim => {
                Ok(value.trim().to_string())
            }

            ValueTransform::Chain(transforms) => {
                let mut result = value.to_string();
                for transform in transforms {
                    result = transform.apply(&result)?;
                }
                Ok(result)
            }
        }
    }

    /// Apply transformation to an optional value
    pub fn apply_optional(&self, value: Option<&str>) -> Result<Option<String>, String> {
        match value {
            Some(v) => self.apply(v).map(Some),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_none() {
        let transform = ValueTransform::None;
        assert_eq!(transform.apply("test").unwrap(), "test");
    }

    #[test]
    fn test_transform_split_last() {
        let transform = ValueTransform::SplitLast("/".to_string());
        assert_eq!(
            transform.apply("path/to/resource").unwrap(),
            "resource"
        );
    }

    #[test]
    fn test_transform_split_last_no_separator() {
        let transform = ValueTransform::SplitLast("/".to_string());
        assert_eq!(transform.apply("resource").unwrap(), "resource");
    }

    #[test]
    fn test_transform_regex_capture() {
        let transform = ValueTransform::RegexCapture {
            pattern: r"(\w+)@(\w+\.\w+)".to_string(),
            group: 1,
        };
        assert_eq!(
            transform.apply("user@example.com").unwrap(),
            "user"
        );
    }

    #[test]
    fn test_transform_regex_capture_group_2() {
        let transform = ValueTransform::RegexCapture {
            pattern: r"(\w+)@(\w+\.\w+)".to_string(),
            group: 2,
        };
        assert_eq!(
            transform.apply("user@example.com").unwrap(),
            "example.com"
        );
    }

    #[test]
    fn test_transform_regex_capture_no_match() {
        let transform = ValueTransform::RegexCapture {
            pattern: r"(\d+)".to_string(),
            group: 1,
        };
        assert!(transform.apply("no-numbers").is_err());
    }

    #[test]
    fn test_transform_regex_replace() {
        let transform = ValueTransform::RegexReplace {
            pattern: r"@.*".to_string(),
            replacement: "".to_string(),
        };
        assert_eq!(
            transform.apply("user@example.com").unwrap(),
            "user"
        );
    }

    #[test]
    fn test_transform_prefix() {
        let transform = ValueTransform::Prefix("User::".to_string());
        assert_eq!(
            transform.apply("user123").unwrap(),
            "User::user123"
        );
    }

    #[test]
    fn test_transform_suffix() {
        let transform = ValueTransform::Suffix("@example.com".to_string());
        assert_eq!(
            transform.apply("user").unwrap(),
            "user@example.com"
        );
    }

    #[test]
    fn test_transform_lowercase() {
        let transform = ValueTransform::Lowercase;
        assert_eq!(
            transform.apply("USER@EXAMPLE.COM").unwrap(),
            "user@example.com"
        );
    }

    #[test]
    fn test_transform_uppercase() {
        let transform = ValueTransform::Uppercase;
        assert_eq!(
            transform.apply("user@example.com").unwrap(),
            "USER@EXAMPLE.COM"
        );
    }

    #[test]
    fn test_transform_trim() {
        let transform = ValueTransform::Trim;
        assert_eq!(
            transform.apply("  user@example.com  ").unwrap(),
            "user@example.com"
        );
    }

    #[test]
    fn test_transform_chain() {
        let transform = ValueTransform::Chain(vec![
            ValueTransform::Trim,
            ValueTransform::Lowercase,
            ValueTransform::Prefix("User::".to_string()),
        ]);

        assert_eq!(
            transform.apply("  USER@EXAMPLE.COM  ").unwrap(),
            "User::user@example.com"
        );
    }

    #[test]
    fn test_transform_chain_complex() {
        let transform = ValueTransform::Chain(vec![
            ValueTransform::RegexCapture {
                pattern: r"(\w+)@(\w+\.\w+)".to_string(),
                group: 1,
            },
            ValueTransform::Prefix("Principal::".to_string()),
        ]);

        assert_eq!(
            transform.apply("john.doe@example.com").unwrap(),
            "Principal::john"
        );
    }

    #[test]
    fn test_transform_chain_split_and_uppercase() {
        let transform = ValueTransform::Chain(vec![
            ValueTransform::SplitLast("/".to_string()),
            ValueTransform::Uppercase,
        ]);

        assert_eq!(
            transform.apply("path/to/resource").unwrap(),
            "RESOURCE"
        );
    }

    #[test]
    fn test_transform_optional_some() {
        let transform = ValueTransform::Uppercase;
        assert_eq!(
            transform.apply_optional(Some("test")).unwrap(),
            Some("TEST".to_string())
        );
    }

    #[test]
    fn test_transform_optional_none() {
        let transform = ValueTransform::Uppercase;
        assert_eq!(transform.apply_optional(None).unwrap(), None);
    }

    #[test]
    fn test_transform_regex_replace_multiple() {
        let transform = ValueTransform::RegexReplace {
            pattern: r"\s+".to_string(),
            replacement: "_".to_string(),
        };
        assert_eq!(
            transform.apply("user  with   spaces").unwrap(),
            "user_with_spaces"
        );
    }

    #[test]
    fn test_transform_invalid_regex() {
        let transform = ValueTransform::RegexCapture {
            pattern: "[invalid(".to_string(),
            group: 1,
        };
        assert!(transform.apply("test").is_err());
    }
}
