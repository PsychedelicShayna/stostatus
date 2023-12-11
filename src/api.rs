use crate::gzip::gzip_inflate;
use crate::http;
use crate::Error;

use crate::search::find_pattern;

#[derive(Debug, PartialEq, Clone)]
pub enum ServerStatus {
    Online,
    Offline,
    Unknown(String),
}

/// Performs some basic JSON validation and eliminates whitespace outside of
/// JSON strings. Ensures correct parity for braces, brackets, and quotes,
/// with escaped characters ignored,, so strings with escaped quotes are 
/// accounted for.

pub fn sanitize_json(data: Vec<u8>) -> Option<Vec<u8>> {
    let mut ordering_stack: Vec<u8> = Vec::new();

    let mut escaping: bool = false;
    let mut quoting: bool = false;

    let mut stripped: Vec<u8> = Vec::new();

    for byte in data.iter() {
        match byte {
            _ if escaping => {
                escaping = false;
            }

            b'\\' => {
                escaping = true;
            }

            b'"' => {
                quoting ^= true;
            }

            _ if quoting => (),

            b'{' | b'[' => ordering_stack.push(*byte),
            b'}' | b']' => {
                if let Some(stack_byte) = ordering_stack.pop() {
                    if (byte - 2) != stack_byte {
                        return None;
                    }
                } else {
                    return None;
                }
            }
            _ => (),
        }

        if !byte.is_ascii_whitespace() {
            stripped.push(*byte);
        }
    }

    (ordering_stack.is_empty() && !quoting && !escaping).then_some(stripped)
}

/// Sarches for the first occurence of a JSON key, and extracts its value with
/// the assumption that the value is a string.
pub fn extract_json_str(json_data: &Vec<u8>, key: &str) -> Result<String, Error> {
    let pattern = format!("\"{}\":\"", key);

    let json_data = sanitize_json(json_data.clone()).ok_or(Error::InvalidJson)?;

    let (beg, _) = find_pattern(&pattern.as_bytes().to_vec(), &json_data.clone())
        .ok_or(Error::NoPattern(json_data.clone()))?;

    let remaining = json_data.iter().skip(beg + pattern.len());

    let value: String = remaining
        .take_while(|&&b| b != b'"')
        .map(|&b| b as char)
        .collect();

    Ok(value)
}

/// Checks the server status of the Star Trek Online game server.
pub fn check_server_status() -> Result<ServerStatus, Error> {
    let domain = "startreklauncher.crypticstudios.com";

    let headers: Vec<(String, String)> = vec![
        ("Host", "startreklauncher.crypticstudios.com"),
        ("Connection", "keep-alive"),
        ("Content-Length", "0"),
        ("Accept", "application/json, text/javascript, */*, q=0.01"),
        ("User-Agent", "Mozilla/4.0 (compatible, CrypticLauncher)"),
        ("X-Accept-Language-Cryptic", "en-US"),
        ("X-Cryptic-Affiliate", "appid=9900"),
        ("X-Cryptic-Version", "3"),
        ("X-Requested-With", "XMLHttpRequest"),
        ("Origin", "http://startreklauncher.crypticstudios.com"),
        (
            "Referer",
            "http://startreklauncher.crypticstudios.com/launcher",
        ),
        ("Accept-Encoding", "gzip, deflate"),
        ("Accept-Language", "en-US,en,q=0.9"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect();

    let request = http::Request::new(
        domain.to_string(),
        "/server_status/".to_string(),
        http::Method::GET,
        headers,
        None,
    );

    // Eliminate all whitespace, and downcase the response data, to ensure
    // consistency when searching for the relevant data.
    let data = request
        .send()?
        .gz_extract()
        .map(|mut gz| unsafe { gzip_inflate(&mut gz) })?
        .map_err(|_| Error::InvalidGZip)?;


    let server_status = extract_json_str(&data, "server_status")?;

    match server_status.as_str() {
        "up" => Ok(ServerStatus::Online),
        "down" => Ok(ServerStatus::Offline),
        s => Ok(ServerStatus::Unknown(s.into())),
    }
}
