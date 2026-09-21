use unicode_normalization::UnicodeNormalization;

/// Phonetic Transliteration Engine: Converts Roman input (ITRANS / Harvard-Kyoto style) into Devanagari.
pub struct Transliterator;

impl Transliterator {
    /// Transliterates digits from Western (0-9) to Devanagari (०-९)
    pub fn transliterate_digits(input: &str) -> String {
        input
            .chars()
            .map(|c| match c {
                '0' => '०',
                '1' => '१',
                '2' => '२',
                '3' => '३',
                '4' => '४',
                '5' => '५',
                '6' => '६',
                '7' => '७',
                '8' => '८',
                '9' => '९',
                other => other,
            })
            .collect()
    }

    /// Transliterates an entire string from phonetic Roman script to Devanagari.
    pub fn to_devanagari(input: &str) -> String {
        let mut output = String::new();
        let chars: Vec<char> = input.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            // Check double danda
            if i + 1 < chars.len() && chars[i] == '|' && chars[i + 1] == '|' {
                output.push('॥');
                i += 2;
                continue;
            }
            if chars[i] == '|' {
                output.push('।');
                i += 1;
                continue;
            }

            // Check digit
            if chars[i].is_ascii_digit() {
                output.push(Self::transliterate_digit(chars[i]));
                i += 1;
                continue;
            }

            // Check if within quotes (string literals)
            if chars[i] == '"' {
                output.push('"');
                i += 1;
                while i < chars.len() && chars[i] != '"' {
                    if chars[i] == '\\' && i + 1 < chars.len() {
                        output.push('\\');
                        output.push(chars[i + 1]);
                        i += 2;
                    } else {
                        output.push(chars[i]);
                        i += 1;
                    }
                }
                if i < chars.len() && chars[i] == '"' {
                    output.push('"');
                    i += 1;
                }
                continue;
            }

            // Check non-alphabetic characters
            if !chars[i].is_alphabetic() {
                output.push(chars[i]);
                i += 1;
                continue;
            }

            // Check if current word is a known Sankode keyword or core word
            let mut word = String::new();
            let mut w_idx = i;
            while w_idx < chars.len() && chars[w_idx].is_alphabetic() {
                word.push(chars[w_idx]);
                w_idx += 1;
            }
            if let Some(kw) = Self::match_common_keyword(&word) {
                output.push_str(kw);
                i = w_idx;
                continue;
            }

            // Parse a syllable: [consonants]* [vowel]
            let (consumed, dev_syllable) = Self::match_syllable(&chars[i..]);
            if consumed > 0 {
                output.push_str(&dev_syllable);
                i += consumed;
            } else {
                output.push(chars[i]);
                i += 1;
            }
        }

        output.nfkc().collect()
    }

    fn transliterate_digit(c: char) -> char {
        match c {
            '0' => '०',
            '1' => '१',
            '2' => '२',
            '3' => '३',
            '4' => '४',
            '5' => '५',
            '6' => '६',
            '7' => '७',
            '8' => '८',
            '9' => '९',
            _ => c,
        }
    }

    fn match_common_keyword(word: &str) -> Option<&'static str> {
        match word {
            "kriya" | "kriyaa" | "kriyA" => Some("क्रिया"),
            "mana" | "mAna" => Some("मान"),
            "vikarya" | "vikArya" => Some("विकार्य"),
            "sthira" => Some("स्थिर"),
            "samracana" | "samracanA" => Some("संरचना"),
            "vikalpa" => Some("विकल्प"),
            "guna" => Some("गुण"),
            "vidhana" | "vidhAna" => Some("विधान"),
            "yadi" => Some("यदि"),
            "anyatha" | "anyathA" => Some("अन्यथा"),
            "yavat" | "yAvat" => Some("यावत्"),
            "pratyeka" => Some("प्रत्येक"),
            "iti" => Some("इति"),
            "prati" => Some("प्रति"),
            "bhanga" => Some("भङ्ग"),
            "anuvrtta" => Some("अनुवृत्त"),
            "satyam" => Some("सत्यम्"),
            "mithya" | "mithyA" => Some("मिथ्या"),
            "mudraya" => Some("मुद्रय"),
            "mukhya" => Some("मुख्य"),
            "rikta" => Some("रिक्त"),
            "purna" | "pUrNa" => Some("पूर्ण"),
            "sutra" | "sUtra" => Some("सूत्र"),
            "varna" | "varNa" => Some("वर्ण"),
            "dvaidha" => Some("द्वैध"),
            _ => None,
        }
    }

    fn match_syllable(slice: &[char]) -> (usize, String) {
        // Try independent vowel first
        if let Some((len, v)) = Self::match_independent_vowel(slice) {
            return (len, v.to_string());
        }

        // Try consonant cluster
        let mut idx = 0;
        let mut result = String::new();
        let mut had_consonant = false;

        while idx < slice.len() {
            if let Some((c_len, cons)) = Self::match_consonant(&slice[idx..]) {
                if had_consonant {
                    result.push('्'); // virama
                }
                result.push_str(cons);
                idx += c_len;
                had_consonant = true;
            } else {
                break;
            }
        }

        if had_consonant {
            // Check following vowel sign (matra)
            if let Some((v_len, matra)) = Self::match_matra(&slice[idx..]) {
                idx += v_len;
                if !matra.is_empty() {
                    result.push_str(matra);
                }
            } else {
                // If no vowel followed, append halant if it's followed by word end or punctuation
                if idx < slice.len() && !slice[idx].is_alphabetic() {
                    result.push('्');
                } else if idx == slice.len() {
                    result.push('्');
                }
            }
            return (idx, result);
        }

        (0, String::new())
    }

    fn match_consonant(slice: &[char]) -> Option<(usize, &'static str)> {
        let text: String = slice.iter().take(3).collect();
        // 3-char
        if text.starts_with("kSh") || text.starts_with("ksh") {
            return Some((3, "क्ष"));
        }
        if text.starts_with("jny") {
            return Some((3, "ज्ञ"));
        }

        // 2-char
        let text2: String = slice.iter().take(2).collect();
        match text2.as_str() {
            "kh" | "Kh" => return Some((2, "ख")),
            "gh" | "Gh" => return Some((2, "घ")),
            "ng" => return Some((2, "ङ")),
            "ch" => return Some((2, "च")),
            "Ch" => return Some((2, "छ")),
            "jh" | "Jh" => return Some((2, "झ")),
            "ny" => return Some((2, "ञ")),
            "Th" => return Some((2, "ठ")),
            "Dh" => return Some((2, "ढ")),
            "th" => return Some((2, "थ")),
            "dh" => return Some((2, "ध")),
            "ph" => return Some((2, "फ")),
            "bh" => return Some((2, "भ")),
            "sh" => return Some((2, "श")),
            "Sh" => return Some((2, "ष")),
            "gy" => return Some((2, "ज्ञ")),
            "tr" => return Some((2, "त्र")),
            _ => {}
        }

        // 1-char
        match slice.first()? {
            'k' | 'K' => Some((1, "क")),
            'g' | 'G' => Some((1, "ग")),
            'c' => Some((1, "च")),
            'j' | 'J' => Some((1, "ज")),
            'T' => Some((1, "ट")),
            'D' => Some((1, "ड")),
            'N' => Some((1, "ण")),
            't' => Some((1, "त")),
            'd' => Some((1, "द")),
            'n' => Some((1, "न")),
            'p' | 'P' => Some((1, "प")),
            'f' => Some((1, "फ")),
            'b' | 'B' => Some((1, "ब")),
            'm' | 'M' => Some((1, "म")),
            'y' | 'Y' => Some((1, "य")),
            'r' | 'R' => Some((1, "र")),
            'l' | 'L' => Some((1, "ल")),
            'v' | 'w' | 'V' | 'W' => Some((1, "व")),
            'S' | 'z' => Some((1, "ष")),
            's' => Some((1, "स")),
            'h' | 'H' => Some((1, "ह")),
            'x' => Some((1, "क्ष")),
            _ => None,
        }
    }

    fn match_independent_vowel(slice: &[char]) -> Option<(usize, &'static str)> {
        let text2: String = slice.iter().take(2).collect();
        match text2.as_str() {
            "aa" => return Some((2, "आ")),
            "ii" | "ee" => return Some((2, "ई")),
            "uu" | "oo" => return Some((2, "ऊ")),
            "ai" => return Some((2, "ऐ")),
            "au" => return Some((2, "औ")),
            "ri" => return Some((2, "ऋ")),
            _ => {}
        }

        match slice.first()? {
            'a' => Some((1, "अ")),
            'A' => Some((1, "आ")),
            'i' => Some((1, "इ")),
            'I' => Some((1, "ई")),
            'u' => Some((1, "उ")),
            'U' => Some((1, "ऊ")),
            'e' => Some((1, "ए")),
            'o' => Some((1, "ओ")),
            _ => None,
        }
    }

    fn match_matra(slice: &[char]) -> Option<(usize, &'static str)> {
        let text2: String = slice.iter().take(2).collect();
        match text2.as_str() {
            "aa" => return Some((2, "ा")),
            "ii" | "ee" => return Some((2, "ी")),
            "uu" | "oo" => return Some((2, "ू")),
            "ai" => return Some((2, "ै")),
            "au" => return Some((2, "ौ")),
            "ri" => return Some((2, "ृ")),
            _ => {}
        }

        match slice.first()? {
            'a' => Some((1, "")), // inherent 'a' leaves consonant without matra or virama
            'A' => Some((1, "ा")),
            'i' => Some((1, "ि")),
            'I' => Some((1, "ी")),
            'u' => Some((1, "ु")),
            'U' => Some((1, "ू")),
            'e' => Some((1, "े")),
            'o' => Some((1, "ो")),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transliterate_digits() {
        assert_eq!(Transliterator::transliterate_digits("1024"), "१०२४");
        assert_eq!(Transliterator::transliterate_digits("3.14"), "३.१४");
    }

    #[test]
    fn test_transliterate_keywords() {
        assert_eq!(Transliterator::to_devanagari("kriya"), "क्रिया");
        assert_eq!(Transliterator::to_devanagari("yadi"), "यदि");
        assert_eq!(Transliterator::to_devanagari("iti"), "इति");
        assert_eq!(Transliterator::to_devanagari("mukhya"), "मुख्य");
    }

    #[test]
    fn test_transliterate_danda() {
        assert_eq!(Transliterator::to_devanagari("kriya mukhya() |"), "क्रिया मुख्य() ।");
    }
}
