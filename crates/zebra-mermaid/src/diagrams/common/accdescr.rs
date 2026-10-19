//! The accDescr terminal for grammars where `accDescr` is also a name. Ported from Pele
//! `src/diagrams/common/accDescr.ts`.

use super::tokens::{blanks, line_rest_before_comment, spaces, Pattern, Scanner, TokenType, ACC_DESCR};

/// `/[\t ]*accDescr(?:[\t ]*:(?:[^\n\r]*?(?=%%)|[^\n\r]*)|\s*({))/y` with the brace scan: once
/// a braced description finds no closing brace there is none further on, so later braced
/// descriptions fail without scanning the rest of the text again.
fn scanner() -> Scanner {
    let mut unclosed = false;
    Box::new(move |src: &str, at: usize| {
        let q = blanks(src, at);
        if !src[q..].starts_with("accDescr") {
            return None;
        }
        let q = q + 8;
        let c = blanks(src, q);
        if src[c..].starts_with(':') {
            return Some(line_rest_before_comment(src, c + 1));
        }
        let o = spaces(src, q);
        if !src[o..].starts_with('{') {
            return None;
        }
        if unclosed {
            return None;
        }
        let end = o + 1;
        match src[end..].find('}') {
            Some(k) => Some(end + k + 1),
            None => {
                unclosed = true;
                None
            }
        }
    })
}

/// The accDescr token under `name`.
pub fn acc_descr(name: &'static str) -> TokenType {
    TokenType { name, pattern: Pattern::Scanner(scanner), ..ACC_DESCR }
}
