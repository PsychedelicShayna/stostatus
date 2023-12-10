use std::{collections::HashMap, str::Chars};

// JSON
//
// Root can be any JSON Value, but is typically an Object or Array.
// Let's call the root X
//
// 1.) What is X?
// 2.) Turn X into Rust counterpart.
// 3.) Give X to container Y (in case of an empty file with the word null in
//                            it, that makes X null and the file itself Y)
//
// Note: since X can be an object or array, that means X can contain another X
//       in which case, we should derive its value by recursively calling self
//       which turns X into Y for that iteration. Effectively, whenever we
//       encounter an array or object, we should always beign another layer
//       of recursion. The top of the file is the first layer of recursion.
//
// What to expect of an object: opening brace {
//  Keys must be strings, values can be any JSON value, delimiter is colon,
//  double quotes only, coma separating each pair, and an eventual }
//
//  Keys MUST be unique. Duplicates are errors.
//  The last pair cannot have a coma; a coma expects next to be a string.
//
//  Example: {"key": "value"}
//  Example Empty: {}
//
// What to expect of an array: opening bracket [
//  Values can be any JSON value, delimiter is comma, and an eventual ]
//
//  Example: ["value1", "value2"]
//  Example Empty: []
//
// What to expect of a string: double quotes "
// Any character except double quotes, backslash, and control characters.
// Backslash is used to escape double quotes, backslash, and control characters.
//
// Example: "value"
// Example Empty: ""
// Example Escaped: "\"value\""
// Example Escaped Empty: "\"\""
//
// What to expect of a number: effectively any decimal integer or float.
// Example: 123
// Example: 123.456
//
// Example: 123.1e-456  (apparently this is possible)
// Example: 12.1e+45   (wtf)
#[derive(Clone, Debug)]
pub enum JsonToken {
    Boolean(bool),
    CloseBrace,
    CloseBracket,
    Colon,
    Comma,
    DobuleQuote,
    Float(f64),
    Integer(i64),
    Null,
    OpenBrace,
    OpenBracket,

    /// Represents any character that may appear in a JSON string.
    /// This is a catch-all for characters that may not be tokens.
    Character(char),
}

/// Various errors that could arise while parsing JSON.
#[derive(Clone, Debug)]
pub enum JsonError {
    /// Encountered a token that should not have been encountered
    /// in that context if the JSON were valid. This is the most
    /// Generic JsonError. Example, expecting "`:`" in "`{ "foo", 2 }`"
    UnexpectedToken,

    /// Variant of `UnexpectedToken` containing the offending character.
    UnexpectedTokenCh(char),

    /// Variant of `UnexpectedToken` containing the offending string.
    UnexpectedTokenSt(String),

    /// Exhausted the input without having having formed a complete token,
    /// or without having met an expected condition. Example: `[3, 4`
    UnexpectedEndOfInput,

    /// A number cannot have a leading zero, unless it is a floating point
    /// number. Example: `0123` is invalid, but `0.123` is valid.
    IntegerWithLeadingZero,

    /// The decimal point was placed in an invalid location, such as after
    /// a non-digit. Invalid example: `-.10.` Valid example: `-1.0`
    BadDecimalPointPlacement,

    /// More than one decimal point was encountered in a floating point
    /// number. Invalid example: `5.141.24` Valid example: `5.14124`
    OverOneDecimalPoint,

    /// Encountered a decimal point placed after an illegal character.
    /// Similar to `BadDecimalPointPlacement`, but may be more specific.
    DecimalPointPlacedAfter(char),

    // The internally built-up string presumed to represent a JSON number,
    // failed at being parsed into a float via `parse::<f64>()`.
    InconvertibleToFloat(String, std::num::ParseFloatError),

    // The internally built-up string presumed to represent a JSON number,
    // failed at being parsed into an integer via `parse::<i64>()`.
    InconvertibleToInt(String, std::num::ParseIntError),

    /// Expected a JsonValue to contain a type that it didn't contain.
    /// The first argument is the JsonValue, the second is the expected type,
    /// which the JsonValue did not contain.
    JsonValueNotType(JsonValue, String),
}

/// Various variants that model JSON's data types into Rust counterparts.
/// A new type distinction is made for floats and integers, which JSON
/// does not do on its own..
#[derive(Clone, Debug)]
pub enum JsonValue {
    /// The JSON arraylist `[]`, represented as a Rust vector of JsonValue
    Array(Vec<JsonValue>),

    /// A JSON boolean, represented as a Rust boolean. What did you expect?
    Boolean(bool),

    /// A floating point JSON number represented as a 64 bit Rust float.
    /// The distinction exists on this end; JSON itself doesn't differentiate
    Float(f64),

    /// A non-fractional JSON number represented as a signed 64 bit Rust int.
    /// The distinction exists on this end; JSON itself doesn't differentiate
    Integer(i64),

    /// The billion dollar mistake.
    Null,

    /// A JSON object `{}`, represented as a Rust `HashMap` with `JsonValue` values.
    Object(HashMap<String, JsonValue>),

    /// A standard JSON string `""`, represented as an owned Rust `String`
    String(String),
}

impl JsonValue {
    pub fn get_string(&self) -> Option<String> {
        match self {
            JsonValue::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    pub fn get_object(&self) -> Option<HashMap<String, JsonValue>> {
        match self {
            JsonValue::Object(o) => Some(o.clone()),
            _ => None,
        }
    }

    pub fn get_array(&self) -> Option<Vec<JsonValue>> {
        match self {
            JsonValue::Array(a) => Some(a.clone()),
            _ => None,
        }
    }

    pub fn get_integer(&self) -> Option<i64> {
        match self {
            JsonValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    pub fn get_float(&self) -> Option<f64> {
        match self {
            JsonValue::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub fn get_boolean(&self) -> Option<bool> {
        match self {
            JsonValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn get_null(&self) -> Option<()> {
        match self {
            JsonValue::Null => Some(()),
            _ => None,
        }
    }

    pub fn is_string(&self) -> bool {
        self.get_string().is_some()
    }

    pub fn is_object(&self) -> bool {
        self.get_object().is_some()
    }

    pub fn is_array(&self) -> bool {
        self.get_array().is_some()
    }

    pub fn is_integer(&self) -> bool {
        self.get_integer().is_some()
    }

    pub fn is_float(&self) -> bool {
        self.get_float().is_some()
    }

    pub fn is_boolean(&self) -> bool {
        self.get_boolean().is_some()
    }

    pub fn is_null(&self) -> bool {
        self.get_null().is_some()
    }

    pub fn is_number(&self) -> bool {
        self.is_integer() || self.is_float()
    }

    pub fn is_primitive(&self) -> bool {
        self.is_string() || self.is_number() || self.is_boolean() || self.is_null()
    }

    pub fn is_container(&self) -> bool {
        self.is_object() || self.is_array()
    }
}

fn parse_number(head: char, tail: &mut Chars) -> Result<JsonValue, JsonError> {
    let mut buffer: String = String::new();

    let mut floating_point: bool = false;
    let mut complete_float: bool = false;

    if !head.is_digit(10) && head != '-' {
        return Err(JsonError::UnexpectedTokenCh(head));
    }

    buffer.push(head);

    let mut previous = head;

    while let Some(character) = tail.next() {
        if character == ',' || character == '}' || character == ']' {
            break;
        } else if character == '.' {
            if floating_point {
                return Err(JsonError::OverOneDecimalPoint);
            } else if !floating_point && !previous.is_digit(10) {
                return Err(JsonError::DecimalPointPlacedAfter(previous));
            }

            floating_point = true;
            buffer.push(character);
        } else if character.is_digit(10) {
            buffer.push(character);
            previous = character;
        } else {
            // This and the above else if **must** come after the first two
            // since the first two cases qualify as non-digit characters, but
            // are deliberately exempt. Preserve the order if refactoring.

            return Err(JsonError::UnexpectedTokenCh(character));
        }
    }

    if floating_point {
        buffer
            .parse::<f64>()
            .map_err(|e| JsonError::InconvertibleToFloat(buffer, e))
            .map(|f| JsonValue::Float(f))
    } else {
        buffer
            .parse::<i64>()
            .map_err(|e| JsonError::InconvertibleToInt(buffer, e))
            .map(|f| JsonValue::Integer(f))
    }
}

fn parse_boolean(head: char, tail: &mut Chars) -> Result<JsonValue, JsonError> {
    match head {
        't' if tail.take(3).collect::<String>() == "rue" => {
            return Ok(JsonValue::Boolean(true));
        }

        'f' if tail.take(4).collect::<String>() == "alse" => {
            return Ok(JsonValue::Boolean(false));
        }

        _ => {
            return Err(JsonError::UnexpectedToken);
        }
    }
}

fn parse_string(head: char, tail: &mut Chars) -> Result<JsonValue, JsonError> {
    let mut buffer: String = String::new();

    if head != '"' {
        return Err(JsonError::UnexpectedToken);
    }

    while let Some(character) = tail.next() {
        match character {
            '"' => {
                return Ok(JsonValue::String(format!("{}{}", head, buffer)));
            }

            '\\' => match tail.next() {
                Some('n') => buffer.push('\n'),
                Some('r') => buffer.push('\r'),
                Some('t') => buffer.push('\t'),
                Some('b') => buffer.push('\u{0008}'),
                Some('f') => buffer.push('\u{000C}'),
                Some(c) => buffer.push(c),
                None => return Err(JsonError::UnexpectedEndOfInput),
            },

            c => buffer.push(c),
        }
    }

    Ok(JsonValue::Null)
}

fn parse_json(json: String) -> Result<JsonValue, JsonError> {
    let mut iter = json.chars();
    parse_value(&mut iter)
}

/// Deduces the type of JSON value from the first character, and delegates
/// to the appropriate function to parse the rest of the value.
fn parse_value(iter: &mut Chars) -> Result<JsonValue, JsonError> {
    while let Some(character) = iter.next() {
        match character {
            c if c.is_whitespace() || c.is_control() => continue,
            't' | 'f' => return parse_boolean(character, iter),
            '"' => return parse_string(character, iter),
            '0'..='9' | '-' => return parse_number(character, iter),
            '{' => return parse_object(character, iter),
            '[' => return parse_array(character, iter),
            _ => return Err(JsonError::UnexpectedTokenCh(character)),
        }
    }

    return Err(JsonError::UnexpectedEndOfInput);
}

// ,[1, 2, {'foo': 'bar'}, true, false, null, 3.14, -3.1, [[4], 1]]
fn parse_array(head: char, tail: &mut Chars) -> Result<JsonValue, JsonError> {
    let mut buffer: Vec<JsonValue> = Vec::new();

    while let Some(c) = tail.next() {
        match c {
            _ if c.is_whitespace() || c.is_control() => continue,
            ']' => return Ok(JsonValue::Array(buffer)),
            ',' => continue,
            _ => buffer.push(parse_value(tail)?),
        }
    }

    Ok(JsonValue::Array(buffer))
}

fn parse_object(head: char, tail: &mut Chars) -> Result<JsonValue, JsonError> {
    let mut buffer: HashMap<String, JsonValue> = HashMap::new();

    if head != '{' {
        return Err(JsonError::UnexpectedToken);
    }

    let mut key: Option<String> = None;
    let mut value: Option<JsonValue> = None;

    let mut awaiting_val = false;
    let mut awiating_col = false;

    while let Some(character) = tail.next() {
        if character.is_whitespace() || character.is_control() {
            continue;
        }

        match (character, &key, &value) {
            (':', Some(k), None) => {
                buffer.insert(k.clone(), parse_value(tail)?);
                key = None;
            }

            (',' | '}', Some(k), Some(v)) => {
                buffer.insert(k.clone(), v.clone());

                key = None;
                value = None;
            }

            ('"', None, None) => {
                if let Some(s) = parse_string(head, tail)?.get_string() {
                    key = Some(s);
                    awaiting_val = true;
                }
            }

            _ => {}
        }
    }

    todo!();
}

//
// pub fn json_parse(json: String) -> Result<String, JsonError> {
//     // There are only a finite number of characters that
//     // can be encountered at the root level, given the
//     // context.
//     //
//     // t = true
//     // f = false
//     // n = null
//     //
//     // : = key-value separator
//     // , = value separator
//     //
//     // { = object
//     // [ = array
//     // " = string start/end
//     //      \ = escape character
//     //
//     // 0-9 = number
//     // - = negative number
//     // . = decimal number
//     // e = scientific notation
//     // + = positive scientific notation
//     // - = negative scientific notation
//
//     struct Context {
//         flag_in_boolean: bool,
//         flag_in_number: bool,
//         flag_in_array: bool,
//         flag_in_object: bool,
//         flag_in_string_escaped: bool,
//         flag_in_string: bool,
//     }
//
//     let mut context = Context {
//         flag_in_boolean: false,
//         flag_in_number: false,
//         flag_in_array: false,
//         flag_in_object: false,
//         flag_in_string_escaped: false,
//         flag_in_string: false,
//     };
//
//
//     let mut value = JsonValue::Null;
//     let mut buf_string = String::new();
//
//     let iter = json.chars();
//
//     fn parse_string(json: mut String, ctx: &mut Context) -> Result<JsonValue, JsonError> {
//         let mut buffer: String = String::new();
//         let mut iter = json.chars();
//
//
//
//
//     }
//
//     fn parse_object(json: mut String, ctx: &mut Context) -> (String, Result<JsonValue, JsonError>) {
//         let mut buffer: HashMap<String, JsonValue> = HashMap::new();
//         let mut iter = json.chars();
//
//
//     }
//
//     while let Some(character) = iter.next() {
//
//         if flag_in_string {
//             if flag_in_string_escaped {
//                 match character {
//                     'n' => buf_string.push('\n'),
//                     'r' => buf_string.push('\r'),
//                     't' => buf_string.push('\t'),
//                     'b' => buf_string.push('\u{0008}'),
//                     'f' => buf_string.push('\u{000C}'),
//                     c => buf_string.push(c),
//                 }
//
//                 flag_in_string_escaped = false;
//                 continue;
//             }
//
//             match character {
//                 '"' => {
//                     flag_in_string = false;
//                 }
//             }
//
//             continue;
//         }
//         //     } else {
//         //         match character {
//         //             '"' => {
//         //                 flag_in_string = false;
//         //                 value = JsonValue::String(buf_string);
//         //                 buf_string = String::new();
//         //             },
//         //
//         //             '\\' => {
//         //                 flag_in_string_escaped = true;
//         //             }
//         //         }
//         //     }
//         //
//         //         flag_in_string_escaped = false;
//         //     } else if character == '"' {
//         //         flag_in_string = false;
//         //         value = JsonValue::String(buf_string);
//         //         buf_string = String::new();
//         //     } else if character == '\\' {
//         //         flag_in_string_escaped = true;
//         //     } else {
//         //         buf_string.push(character);
//         //     }
//         //
//         //
//         // }
//
//
//         match character {
//             '{' => {
//                 if flag_in_string {
//                     buf_string.push(character);
//                     continue;
//                 }
//
//                 if flag_in_object {}
//             }
//         }
//     }
//
//     for (index, character) in json.chars() {
//         match character {
//             '{' => {2
//                 if flag_in_string {
//                     buf_string.push(character);
//                     continue;
//                 }
//             }
//         }
//     }
//     todo!();
// }
//

//  Unit tests

#[cfg(test)]
mod tests {
    use super::*;

    // Why macros are pogu, part 5.
    //
    // I have no plan in mind, I'm literally just recording and winging it.
    //
    // The problem with all these test cases is that they should actually be
    // formatted like this

    #[test]
    fn test_parse_number() {
        assert!(match parse_number('1', &mut "234".chars()) {
            Ok(JsonValue::Integer(s)) if s == 1234 => true,
            _ => false,
        });

        assert!(match parse_number('-', &mut "123".chars()) {
            Ok(JsonValue::Integer(v)) if -123 == v => true,
            _ => false,
        });

        assert!(match parse_number('1', &mut "23.4".chars()) {
            Ok(JsonValue::Float(v)) if 123.4 == v => true,
            _ => false,
        });

        assert!(match parse_number('-', &mut "123.4".chars()) {
            Ok(JsonValue::Float(v)) if -123.4 == v => true,
            _ => false,
        });
    }

    #[test]
    fn test_parse_string() {

        assert!(match parse_string('"', &mut "test\"".chars()) {
            Ok(JsonValue::String(s)) => {
                println!("S Is: {}", s);
                s == "test".to_string()
            }
            _ => false,
        });

        // assert!(match parse_string('"', &mut "\\\"test\\\"".chars()) {
        //     Ok(JsonValue::String(s)) => s == "\"test\"".to_string(),
        //     _ => false,
        // });
    }

    #[test]
    fn test_parse_boolean() {
        assert!(match parse_boolean('t', &mut "rue".chars()) {
            Ok(JsonValue::Boolean(s)) => s == true,
            _ => false,
        });
        assert!(match parse_boolean('f', &mut "alse".chars()) {
            Ok(JsonValue::Boolean(s)) => s == false,
            _ => false,
        });
    }

    #[test]
    fn test_parse_array() {
        assert!(match parse_array('[', &mut "[1, 2, 3]".chars()) {
            Ok(JsonValue::Array(vjv)) => vjv
                .into_iter()
                .zip(vec![1, 2, 3])
                .all(|(a, b)| matches!(a.get_integer(), Some(b))),
            _ => false,
        });
    }

    #[test]
    fn test_parse_object1_strings() {
        let mut test_map: HashMap<String, String> = HashMap::new();
        test_map.insert("key".into(), "value".into());

        assert!(match parse_object('{', &mut "\"key\": \"value\" }".chars()) {
            Ok(JsonValue::Object(mjv)) => {
                mjv.into_iter().all( |(k, v)| matches!(v.get_string(), Some(s) if s == *test_map.get(&k).unwrap()))
            },

            _ => false
        });
    }

    #[test]
    fn test_parse_object2_integers() {
        let mut test_map: HashMap<String, i64> = HashMap::new();
        test_map.insert("key".into(), 1234i64);

        assert!(match parse_object('{', &mut "\"key\": 1234 }".chars()) {
            Ok(JsonValue::Object(mjv)) => {
                mjv.into_iter().all(
                    |(k, v)| matches!(v.get_integer(), Some(s) if s == *test_map.get(&k).unwrap()),
                )
            }

            _ => false,
        });
    }

    #[test]
    fn test_parse_object3_floats() {
        let mut test_map: HashMap<String, f64> = HashMap::new();
        test_map.insert("key".into(), 12.34f64);

        assert!(match parse_object('{', &mut "\"key\": 12.34 }".chars()) {
            Ok(JsonValue::Object(mjv)) => {
                mjv.into_iter().all(
                    |(k, v)| matches!(v.get_float(), Some(s) if s == *test_map.get(&k).unwrap()),
                )
            }

            _ => false,
        });
    }


    #[test]
    fn test_parse_object4_bools() {
        let mut test_map: HashMap<String, bool> = HashMap::new();
        test_map.insert("key1".into(), true);
        test_map.insert("key2".into(), false);

        assert!(match parse_object('{', &mut "\"key1\": true, \"key2\": false }".chars()) {
            Ok(JsonValue::Object(mjv)) => {
                mjv.into_iter().all(
                    |(k, v)| matches!(v.get_boolean(), Some(s) if s == *test_map.get(&k).unwrap()),
                )
            }

            _ => false,
        });
    }

    #[test]
    fn test_parse_object5_arrays_int() {
        let mut test_map: HashMap<String, Vec<i64>> = HashMap::new();
        let test_vec = vec![1, 2, 3];

        test_map.insert("key".into(), test_vec.clone());

        assert!(match parse_object('{', &mut "\"key\": [1, 2, 3] }".chars()) {
            Ok(JsonValue::Object(mjv)) => {
                mjv.into_iter().all( |(_, v)| -> bool {
                    let v = match v {
                        JsonValue::Array(v) if v.len() == test_vec.len() => v,
                        _ => return false,
                    };
                    
                    v.into_iter().zip(test_vec.clone()).all(|(a, b)| matches!(a.get_integer(), Some(b)))
                })
            }

            _ => false,
        });
    }
}
// const NEG: i32 = 0b1000_0000_0000_0000_0000_0000_0000_0000_u32 as i32;
//
//         assert!(matches!(parse_number('1', &mut "2".chars()), Ok(JsonValue::Integer(12))), "12");
//
//         let sample3 = "\"-123.456";
//         let sampler = "\"-123.456";
//
//     }
//
//
// }
