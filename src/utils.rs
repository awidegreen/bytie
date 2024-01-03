use bytesize::ByteSize;

pub fn parse_to_vec(v: &str) -> Result<Vec<u8>, String> {
    Ok(v.as_bytes().to_vec())
}

pub fn parse_pos(p: &str) -> Result<ByteSize, String> {
    if let Ok(v) = p.parse() {
        return Ok(v);
    };

    let s = p.trim_start_matches("0x");
    match u64::from_str_radix(s, 16) {
        Ok(v) => Ok(ByteSize::b(v)),
        Err(_) => Err("not a valid position".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pos() {
        assert_eq!(parse_pos("15"), Ok(ByteSize::b(15)));
        assert_eq!(parse_pos("015"), Ok(ByteSize::b(15)));
        assert_eq!(parse_pos("15 B"), Ok(ByteSize::b(15)));
        assert_eq!(parse_pos("15 b"), Ok(ByteSize::b(15)));
        assert_eq!(parse_pos("3445 Kib"), Ok(ByteSize::kib(3445)));

        assert_eq!(parse_pos("0x15"), Ok(ByteSize::b(0x15)));
        assert!(parse_pos("0xg5").is_err());
        assert!(parse_pos("xg5").is_err());
        assert!(parse_pos("-5").is_err());
    }
}
