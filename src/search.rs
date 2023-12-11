pub fn find_pattern(needle: &Vec<u8>, haystack: &[u8]) -> Option<(usize, usize)> {
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
