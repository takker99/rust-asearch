#![warn(missing_docs)]
//! Approximate pattern matching crate

const INITPAT: u32 = 0x80000000; // 0100,0000,0000,0000,0000,0000,0000,0000
const MAXCHAR: usize = 0x10000;
const INITSTATE: [u32; 4] = [INITPAT, 0, 0, 0];

/// Approximate pattern matching engine
pub struct Asearch {
    shiftpat: [u32; MAXCHAR],
    acceptpat: u32,
    epsilon: u32,
}

impl Asearch {
    /// Create a new approximate pattern matching engine
    ///
    /// * `source` - text which is searched for.
    pub fn new(source: impl Into<String>) -> Asearch {
        let mut shiftpat: [u32; MAXCHAR] = [0; MAXCHAR];
        let mut mask = INITPAT;
        let mut epsilon: u32 = 0;
        for item in &unpack(source.into()) {
            // 0x20 is a space
            if *item == 0x20 {
                epsilon |= mask;
            } else {
                shiftpat[*item] |= mask;
                shiftpat[to_upper(*item)] |= mask;
                shiftpat[to_lower(*item)] |= mask;
                mask >>= 1;
            }
        }
        Asearch {
            acceptpat: mask,
            shiftpat,
            epsilon,
        }
    }

    fn state(&self, text: impl Into<String>) -> [u32; 4] {
        let mut i0 = INITSTATE[0];
        let mut i1 = INITSTATE[1];
        let mut i2 = INITSTATE[2];
        let mut i3 = INITSTATE[3];
        for item in &unpack(text.into()) {
            let mask = self.shiftpat[*item];
            i3 = (i3 & self.epsilon) | ((i3 & mask) >> 1) | (i2 >> 1) | i2;
            i2 = (i2 & self.epsilon) | ((i2 & mask) >> 1) | (i1 >> 1) | i1;
            i1 = (i1 & self.epsilon) | ((i1 & mask) >> 1) | (i0 >> 1) | i0;
            i0 = (i0 & self.epsilon) | ((i0 & mask) >> 1);
            i1 |= i0 >> 1;
            i2 |= i1 >> 1;
            i3 |= i2 >> 1;
        }
        [i0, i1, i2, i3]
    }

    /// Do approximate pattern matching
    ///
    /// * `text` - text which is searched.
    /// * `ambig` - Levenshtein distance.
    pub fn find(&self, text: impl Into<String>, ambig: u8) -> bool {
        let ambig_ = if (ambig as usize) < INITSTATE.len() {
            ambig as usize
        } else {
            INITSTATE.len() - 1
        };

        let s = self.state(text.into());
        (s[ambig_] & self.acceptpat) != 0
    }
}

/// convert each char to a code point
/// They are used for indice of the finite automaton
fn unpack(text: impl Into<String>) -> Vec<usize> {
    text.into()
        .chars()
        .into_iter()
        .map(|c| c as usize)
        .collect()
}

fn is_upper(c: usize) -> bool {
    (0x41..=0x5a).contains(&c)
}
fn is_lower(c: usize) -> bool {
    (0x61..=0x7a).contains(&c)
}
fn to_lower(c: usize) -> usize {
    if is_upper(c) {
        c + 0x20
    } else {
        c
    }
}
fn to_upper(c: usize) -> usize {
    if is_lower(c) {
        c - 0x20
    } else {
        c
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pattern_abcde() {
        let asearch = Asearch::new("abcde");

        assert!(asearch.find("abcde", 0));
        assert!(asearch.find("aBCDe", 0));
        assert!(asearch.find("abXcde", 1));
        assert!(asearch.find("ab?de", 1));
        assert!(asearch.find("abXXde", 2));
        assert!(!asearch.find("abXcde", 0));
        assert!(!asearch.find("ab?de", 0));
        assert!(!asearch.find("abde", 0));
        assert!(!asearch.find("abXXde", 1));
        assert!(asearch.find("abcde", 1));
        assert!(!asearch.find("abcd", 0));
        assert!(asearch.find("abcd", 1));
        assert!(asearch.find("bcde", 2)); // TODO: 1で通るようにcodeを修正する
    }

    #[test]
    fn pattern_ab_de() {
        let asearch = Asearch::new("ab de");

        assert!(asearch.find("abcde", 0));
        assert!(asearch.find("abccde", 0));
        assert!(asearch.find("abXXXXXXXde", 0));
        assert!(asearch.find("ababcccccxede", 1));
        assert!(!asearch.find("abcccccxe", 0));
    }

    #[test]
    fn pattern_unicode() {
        let asearch = Asearch::new("漢字文字列");

        assert!(asearch.find("漢字文字列", 0));
        assert!(!asearch.find("漢字の文字列", 0));
        assert!(asearch.find("漢字の文字列", 1));
        assert!(!asearch.find("漢字文字", 0));
        assert!(asearch.find("漢字文字", 1));
        assert!(!asearch.find("漢字文字烈", 0));
        assert!(asearch.find("漢字文字烈", 1));
        assert!(!asearch.find("漢和辞典", 2));
    }
}
