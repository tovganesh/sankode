/// Utilities for working with Devanagari numerals (०, १, २, ३, ४, ५, ६, ७, ८, ९)

pub const DEVANAGARI_DIGITS: [char; 10] = ['०', '१', '२', '३', '४', '५', '६', '७', '८', '९'];

/// Checks if a character is a Devanagari digit
pub fn is_devanagari_digit(c: char) -> bool {
    ('०'..='९').contains(&c)
}

/// Converts a Devanagari digit character to its integer value (0-9)
pub fn devanagari_digit_to_u32(c: char) -> Option<u32> {
    if is_devanagari_digit(c) {
        Some((c as u32) - ('०' as u32))
    } else {
        None
    }
}

/// Converts an integer (0-9) to a Devanagari digit character
pub fn u32_to_devanagari_digit(d: u32) -> Option<char> {
    if d < 10 {
        char::from_u32(('०' as u32) + d)
    } else {
        None
    }
}

/// Parses a string of Devanagari digits into an i64 integer
pub fn parse_devanagari_i64(s: &str) -> Result<i64, String> {
    let mut result: i64 = 0;
    let mut is_negative = false;
    let mut chars = s.chars().peekable();

    if let Some(&'-') = chars.peek() {
        is_negative = true;
        chars.next();
    }

    let mut count = 0;
    for c in chars {
        if let Some(digit) = devanagari_digit_to_u32(c) {
            result = result
                .checked_mul(10)
                .and_then(|r| r.checked_add(digit as i64))
                .ok_or_else(|| format!("संख्या अतिविशाला अस्ति (Integer overflow): {}", s))?;
            count += 1;
        } else {
            return Err(format!("अमान्यः देवनागरी-अङ्कः (Invalid Devanagari digit): '{}'", c));
        }
    }

    if count == 0 {
        return Err("रिक्ता संख्या (Empty number)".to_string());
    }

    if is_negative {
        result = -result;
    }

    Ok(result)
}

/// Formats an i64 integer into a Devanagari numeral string
pub fn format_i64_devanagari(mut n: i64) -> String {
    if n == 0 {
        return "०".to_string();
    }

    let is_negative = n < 0;
    if is_negative {
        n = -n;
    }

    let mut digits = Vec::new();
    let mut val = n as u64;
    while val > 0 {
        let d = (val % 10) as u32;
        digits.push(u32_to_devanagari_digit(d).unwrap());
        val /= 10;
    }

    if is_negative {
        digits.push('-');
    }

    digits.into_iter().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_digit_conversion() {
        assert_eq!(devanagari_digit_to_u32('०'), Some(0));
        assert_eq!(devanagari_digit_to_u32('५'), Some(5));
        assert_eq!(devanagari_digit_to_u32('९'), Some(9));
        assert_eq!(devanagari_digit_to_u32('a'), None);
    }

    #[test]
    fn test_parse_devanagari_i64() {
        assert_eq!(parse_devanagari_i64("०").unwrap(), 0);
        assert_eq!(parse_devanagari_i64("१०").unwrap(), 10);
        assert_eq!(parse_devanagari_i64("१२३४५६७८९").unwrap(), 123456789);
        assert_eq!(parse_devanagari_i64("-४२").unwrap(), -42);
    }

    #[test]
    fn test_format_devanagari() {
        assert_eq!(format_i64_devanagari(0), "०");
        assert_eq!(format_i64_devanagari(10), "१०");
        assert_eq!(format_i64_devanagari(12345), "१२३४५");
        assert_eq!(format_i64_devanagari(-42), "-४२");
    }
}
