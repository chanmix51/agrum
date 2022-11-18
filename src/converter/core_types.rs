use super::{ConversionError, FromSQL};

impl FromSQL<i64> for i64 {
    fn from_sql(value: &str) -> Result<Option<i64>, ConversionError> {
        let value = value.trim();

        if value.is_empty() {
            return Ok(None);
        }

        let int = value.parse::<i64>().map_err(|e| {
            ConversionError::nest(&format!("Could not parse '{}' to i64", value), Box::new(e))
        })?;

        Ok(Some(int))
    }
}

impl FromSQL<bool> for bool {
    fn from_sql(value: &str) -> Result<Option<bool>, ConversionError> {
        let value = value.trim();

        match value {
            "" => Ok(None),
            "t" => Ok(Some(true)),
            "f" => Ok(Some(false)),
            _ => Err(ConversionError::raise(&format!(
                "Could not parse '{}' to boolean",
                value
            ))),
        }
    }
}

impl FromSQL<String> for String {
    fn from_sql(value: &str) -> Result<Option<String>, ConversionError> {
        let value = value.trim();

        Ok(Some(value.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_i64_from() {
        let values = &[
            ("0", Some(0)),
            ("1", Some(1)),
            ("-1", Some(-1)),
            ("9223372036854775807", Some(i64::MAX)),
            ("-9223372036854775808", Some(i64::MIN)),
            (" 100 ", Some(100)),
            (" -100 ", Some(-100)),
            ("", None),
            ("  ", None),
        ];

        for (i, o) in values {
            assert_eq!(*o, i64::from_sql(i).unwrap())
        }

        let values = ["0 1", "1A2", "1,2", "1.2", "'123'"];

        for v in values {
            assert!(i64::from_sql(v).is_err());
        }
    }

    #[test]
    fn boolean_from() {
        let values = [
            ("t", Some(true)),
            ("f", Some(false)),
            (" t ", Some(true)),
            (" f ", Some(false)),
            ("", None),
            ("  ", None),
        ];

        for (i, o) in values {
            assert_eq!(o, bool::from_sql(i).unwrap())
        }

        let values = [
            "tf", "T", "F", "true", "false", "'true'", "'false'", "0", "1",
        ];

        for v in values {
            assert!(bool::from_sql(v).is_err());
        }
    }

    #[test]
    fn string_from() {
        let values = [
            ("'ab cd'", Some(String::from("ab cd"))),
            ("'ab cd'", Some(String::from("ab cd"))),
            (" ' ab cd ' ", Some(String::from(" ab cd "))),
            ("'I ❤️ 🦀'", Some(String::from("I ❤️ 🦀"))),
            ("''", Some(String::from(""))),
            ("", None),
            ("  ", None),
        ];

        for (i, o) in values {
            assert_eq!(o, String::from_sql(i).unwrap())
        }

        let values = ["'ab cd", "ab cd'", "''ab cd''", "a'bcd'", "'ab cd'e"];

        for v in values {
            assert!(bool::from_sql(v).is_err());
        }
    }
}
