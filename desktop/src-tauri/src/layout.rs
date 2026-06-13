//! QWERTY ↔ ЙЦУКЕН (українська) символьна мапа.
//! Базується на стандартних розкладках Windows: US QWERTY та Ukrainian (Enhanced).

use std::collections::HashMap;

fn pairs() -> Vec<(char, char)> {
    // (en, ua) — нижній регістр; великі генеруються через to_uppercase нижче.
    vec![
        ('q', 'й'), ('w', 'ц'), ('e', 'у'), ('r', 'к'), ('t', 'е'),
        ('y', 'н'), ('u', 'г'), ('i', 'ш'), ('o', 'щ'), ('p', 'з'),
        ('[', 'х'), (']', 'ї'),
        ('a', 'ф'), ('s', 'і'), ('d', 'в'), ('f', 'а'), ('g', 'п'),
        ('h', 'р'), ('j', 'о'), ('k', 'л'), ('l', 'д'),
        (';', 'ж'), ('\'', 'є'),
        ('z', 'я'), ('x', 'ч'), ('c', 'с'), ('v', 'м'), ('b', 'и'),
        ('n', 'т'), ('m', 'ь'), (',', 'б'), ('.', 'ю'),
        ('/', '.'),
        ('`', '\''),
    ]
}

fn build_map(reverse: bool) -> HashMap<char, String> {
    let mut map: HashMap<char, String> = HashMap::new();
    for (en, ua) in pairs() {
        let (from, to) = if reverse { (ua, en) } else { (en, ua) };
        map.insert(from, to.to_string());
        // Великі літери
        let from_upper: String = from.to_uppercase().collect();
        let to_upper: String = to.to_uppercase().collect();
        if from_upper.chars().count() == 1 {
            map.insert(from_upper.chars().next().unwrap(), to_upper);
        }
    }
    map
}

pub fn en_to_ua(input: &str) -> String {
    let map = build_map(false);
    convert(input, &map)
}

pub fn ua_to_en(input: &str) -> String {
    let map = build_map(true);
    convert(input, &map)
}

fn convert(input: &str, map: &HashMap<char, String>) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match map.get(&ch) {
            Some(s) => out.push_str(s),
            None => out.push(ch),
        }
    }
    out
}

/// Евристика: визначити, чи текст виглядає як «крякозябри»
/// (UA текст набраний на EN розкладці) або навпаки.
pub fn detect_direction(input: &str) -> Direction {
    let mut ua_chars = 0usize;
    let mut en_letters = 0usize;
    let mut en_special = 0usize; // [ ] ; ' `
    let total: usize = input.chars().filter(|c| !c.is_whitespace()).count();

    for ch in input.chars() {
        if ch.is_alphabetic() {
            if ('а'..='я').contains(&ch.to_ascii_lowercase_unchecked())
                || matches!(ch, 'ї' | 'і' | 'є' | 'ґ' | 'Ї' | 'І' | 'Є' | 'Ґ')
                || ('а'..='я').contains(&ch)
                || ('А'..='Я').contains(&ch)
            {
                ua_chars += 1;
            } else if ch.is_ascii_alphabetic() {
                en_letters += 1;
            }
        } else if matches!(ch, '[' | ']' | ';' | '\'' | '`') {
            en_special += 1;
        }
    }

    if total == 0 {
        return Direction::EnToUa;
    }

    // Багато спецсимволів у тексті, що мав би бути словами → це EN-розкладка
    let suspicious = en_letters + en_special;
    if ua_chars > suspicious {
        Direction::UaToEn
    } else {
        Direction::EnToUa
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    EnToUa,
    UaToEn,
}

// Невеликий helper, бо to_ascii_lowercase працює тільки для ASCII.
trait AsciiLowerUnchecked {
    fn to_ascii_lowercase_unchecked(self) -> char;
}
impl AsciiLowerUnchecked for char {
    fn to_ascii_lowercase_unchecked(self) -> char {
        if self.is_ascii() {
            self.to_ascii_lowercase()
        } else {
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn en_to_ua_basic() {
        assert_eq!(en_to_ua("ghbdsn"), "привіт");
        assert_eq!(en_to_ua("Ghbdsn!"), "Привіт!");
    }

    #[test]
    fn roundtrip() {
        let original = "ghbdsn cdsne";
        let ua = en_to_ua(original);
        assert_eq!(ua_to_en(&ua), original);
    }

    #[test]
    fn detect_works() {
        assert_eq!(detect_direction("ghbdsn"), Direction::EnToUa);
        assert_eq!(detect_direction("привіт"), Direction::UaToEn);
    }
}