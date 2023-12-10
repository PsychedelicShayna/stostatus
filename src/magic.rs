use std::slice::from_raw_parts;

use crate::Error;

//-----------------------------------------------------------------------------
//       \r\n\r\n --->  End of HTTP Headers         GZip Magic Number
//                             vvvv  ....  ....  ....  vvvv  vvvv
pub const MAGIC_PATTERN: [u8; 6] = [0x0d, 0x0a, 0x0d, 0x0a, 0x1f, 0x8b];

pub fn find_pattern(needle: &Vec<u8>, haystack: &Vec<u8>) -> Option<(usize, usize)> {
    let mut found = false;

    let mut needle_idx = 0;

    let mut found_beg_idx: Option<usize> = None;
    let mut found_end_idx: Option<usize> = None;

    for (idx, byte) in haystack.iter().enumerate() {
        if *byte == needle[needle_idx] {
            if needle_idx == 0 && found_beg_idx.is_none() {
                found_beg_idx = Some(idx);
            }

            needle_idx += 1;

            if needle_idx == needle.len() {
                found_end_idx = Some(idx);
                break;
            }
        } else {
            needle_idx = 0;
            found_beg_idx = None;
        }
    }

    if let (Some(beg), Some(end)) = (found_beg_idx, found_end_idx) {
        return Some((beg, end));
    }

    None
}

pub fn find_magic_pattern(haystack: &Vec<u8>) -> Option<(usize, usize)> {
    find_pattern(&MAGIC_PATTERN.to_vec(), haystack)
}

pub fn extract_gzip_payload(haystack: &Vec<u8>) -> Result<Vec<u8>, Error> {
    if let Some((_, end)) = find_magic_pattern(haystack) {
        let gzip_start = end - 1;
        let gzip_data = &haystack[gzip_start..];

        return Ok(gzip_data.to_vec());
    }

    Err(Error::NoData)
}

pub fn extract_server_status(haystack: &Vec<u8>) -> Result<String, Error> {
    let server_status_pattern = "\"server_status\":";
    let server_status_pattern_bytes = server_status_pattern.as_bytes();

    if let Some((beg, end)) = find_pattern(&server_status_pattern_bytes.to_vec(), haystack) {
        let server_status: Vec<u8> = haystack[beg..]
            .iter()
            .take_while(|&&b| b != b'}')
            .map(|br| *br)
            .collect();

        return Ok(String::from_utf8_lossy(&server_status).to_string());
    }

    Err(Error::NoData)
}
