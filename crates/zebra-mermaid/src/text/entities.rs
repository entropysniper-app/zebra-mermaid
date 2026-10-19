//! Entity codes: Mermaid's `#name;` placeholders and HTML character references.
//! Ported from Pele `src/text/entities.ts`.

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::OnceLock;

use crate::util::js;

const NAMED: &str = "amp:38,lt:60,gt:62,quot:34,apos:39,nbsp:160,iexcl:161,cent:162,pound:163,curren:164,yen:165,sect:167,\
copy:169,laquo:171,not:172,reg:174,deg:176,plusmn:177,sup2:178,sup3:179,micro:181,para:182,middot:183,\
sup1:185,raquo:187,frac14:188,frac12:189,frac34:190,iquest:191,times:215,divide:247,ndash:8211,mdash:8212,\
lsquo:8216,rsquo:8217,sbquo:8218,ldquo:8220,rdquo:8221,bdquo:8222,dagger:8224,Dagger:8225,bull:8226,\
hellip:8230,permil:8240,prime:8242,Prime:8243,lsaquo:8249,rsaquo:8250,euro:8364,trade:8482,larr:8592,\
uarr:8593,rarr:8594,darr:8595,harr:8596,lArr:8656,uArr:8657,rArr:8658,dArr:8659,hArr:8660,forall:8704,\
part:8706,exist:8707,empty:8709,nabla:8711,isin:8712,notin:8713,ni:8715,prod:8719,sum:8721,minus:8722,\
radic:8730,infin:8734,and:8743,or:8744,cap:8745,cup:8746,int:8747,there4:8756,sim:8764,cong:8773,\
asymp:8776,ne:8800,equiv:8801,le:8804,ge:8805,sub:8834,sup:8835,sube:8838,supe:8839,oplus:8853,otimes:8855,\
perp:8869,sdot:8901,loz:9674,spades:9824,clubs:9827,hearts:9829,diams:9830,check:10003,cross:10007,\
equals:61,colon:58,semi:59,num:35,percnt:37,lpar:40,rpar:41,lbrack:91,rbrack:93,lbrace:123,rbrace:125,\
vert:124,sol:47,bsol:92,quest:63,excl:33,ast:42,plus:43,comma:44,period:46,dollar:36,commat:64,grave:96,\
hat:94,lowbar:95,tilde:126,Alpha:913,Beta:914,Gamma:915,Delta:916,Epsilon:917,Zeta:918,Eta:919,Theta:920,\
Iota:921,Kappa:922,Lambda:923,Mu:924,Nu:925,Xi:926,Omicron:927,Pi:928,Rho:929,Sigma:931,Tau:932,\
Upsilon:933,Phi:934,Chi:935,Psi:936,Omega:937,alpha:945,beta:946,gamma:947,delta:948,epsilon:949,zeta:950,\
eta:951,theta:952,iota:953,kappa:954,lambda:955,mu:956,nu:957,xi:958,omicron:959,pi:960,rho:961,sigmaf:962,\
sigma:963,tau:964,upsilon:965,phi:966,chi:967,psi:968,omega:969";

fn named() -> &'static HashMap<&'static str, char> {
    static MAP: OnceLock<HashMap<&'static str, char>> = OnceLock::new();
    MAP.get_or_init(|| {
        NAMED
            .split(',')
            .map(|pair| {
                let (name, code) = pair.split_once(':').expect("pair");
                (name, char::from_u32(code.parse().expect("code")).expect("char"))
            })
            .collect()
    })
}

/// The character a named entity stands for, as Pele's table has it.
pub fn lookup(name: &str) -> Option<char> {
    named().get(name).copied()
}

/// `String.fromCodePoint(n)` for a valid scalar value above zero, else `None`.
fn code_point(n: f64) -> Option<char> {
    if n > 0.0 && n <= 1_114_111.0 && !(55296.0..=57343.0).contains(&n) && n.fract() == 0.0 {
        char::from_u32(n as u32)
    } else {
        // Pele passes a non-integral value to String.fromCodePoint, which throws; Rust keeps
        // the text as written instead. Only a hand-typed placeholder can reach this.
        None
    }
}

pub(crate) const OPEN: &str = "\u{fb02}\u{b0}";
pub(crate) const CLOSE: &str = "\u{b6}\u{df}";

/// Mermaid's placeholders run from `ﬂ°` to `¶ß`, with a second `°` marking a numeric code.
fn decode_placeholders(text: &str) -> String {
    let mut out = String::new();
    let mut last = 0;
    let mut from = text.find(OPEN);
    while let Some(f) = from {
        let after_open = f + OPEN.len();
        // `text.indexOf('¶', from + 2)`.
        let Some(stop) = text[after_open..].find('\u{b6}').map(|k| k + after_open) else {
            break;
        };
        let after_stop = stop + '\u{b6}'.len_utf8();
        if text[after_stop..].starts_with('\u{df}') {
            let numeric = text[after_open..].starts_with('\u{b0}');
            let body_start = if numeric { after_open + '\u{b0}'.len_utf8() } else { after_open };
            let end = after_stop + '\u{df}'.len_utf8();
            // A numeric placeholder whose degree sign is the `¶`'s neighbour has no body.
            let body = if body_start <= stop { &text[body_start..stop] } else { "" };
            let raw = &text[f..end];
            out.push_str(&text[last..f]);
            if numeric {
                match code_point(js::number(body)) {
                    Some(c) => out.push(c),
                    None => out.push_str(raw),
                }
            } else {
                match lookup(body) {
                    Some(c) => out.push(c),
                    None => {
                        out.push('&');
                        out.push_str(body);
                        out.push(';');
                    }
                }
            }
            last = end;
        }
        from = text[after_stop..].find(OPEN).map(|k| k + after_stop);
    }
    if last == 0 {
        text.to_string()
    } else {
        out.push_str(&text[last..]);
        out
    }
}

/// Replaces `&(?:#(\d+)|#[xX]([0-9a-fA-F]+)|(\w+));` matches.
fn decode_references(text: &str) -> String {
    let b = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    let mut i = 0;
    while i < b.len() {
        if b[i] != b'&' {
            i += 1;
            continue;
        }
        let start = i;
        let mut j = i + 1;
        let mut replaced: Option<Cow<'_, str>> = None;
        if j < b.len() && b[j] == b'#' {
            j += 1;
            if j < b.len() && (b[j] == b'x' || b[j] == b'X') {
                // `#(\d+)` is tried first; "x" is not a digit, so only the hex branch can match.
                let k0 = j + 1;
                let mut k = k0;
                while k < b.len() && b[k].is_ascii_hexdigit() {
                    k += 1;
                }
                if k > k0 && k < b.len() && b[k] == b';' {
                    let value = js::parse_int(&text[k0..k], 16);
                    let m = &text[start..=k];
                    replaced = Some(match code_point(value) {
                        Some(c) => Cow::Owned(c.to_string()),
                        None => Cow::Borrowed(m),
                    });
                    j = k + 1;
                }
            } else {
                let k0 = j;
                let mut k = k0;
                while k < b.len() && b[k].is_ascii_digit() {
                    k += 1;
                }
                if k > k0 && k < b.len() && b[k] == b';' {
                    let value = js::number(&text[k0..k]);
                    let m = &text[start..=k];
                    replaced = Some(match code_point(value) {
                        Some(c) => Cow::Owned(c.to_string()),
                        None => Cow::Borrowed(m),
                    });
                    j = k + 1;
                }
            }
        } else {
            let k0 = j;
            let mut k = k0;
            while k < b.len() && (b[k].is_ascii_alphanumeric() || b[k] == b'_') {
                k += 1;
            }
            if k > k0 && k < b.len() && b[k] == b';' {
                let m = &text[start..=k];
                replaced = Some(match lookup(&text[k0..k]) {
                    Some(c) => Cow::Owned(c.to_string()),
                    None => Cow::Borrowed(m),
                });
                j = k + 1;
            }
        }
        match replaced {
            Some(r) => {
                out.push_str(&text[last..start]);
                out.push_str(&r);
                last = j;
                i = j;
            }
            None => i += 1,
        }
    }
    if last == 0 {
        return text.to_string();
    }
    out.push_str(&text[last..]);
    out
}

/// Turns Mermaid's `#35;` / `#quot;` codes (as hidden by `encode_entities`) and HTML entities
/// into characters.
pub fn decode_entities(text: &str) -> String {
    let mut text: Cow<'_, str> = Cow::Borrowed(text);
    if text.contains(CLOSE) {
        text = Cow::Owned(decode_placeholders(&text));
    }
    if text.contains('&') {
        text = Cow::Owned(decode_references(&text));
    }
    text.into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_references() {
        assert_eq!(decode_entities("a &amp; b"), "a & b");
        assert_eq!(decode_entities("&#35;&#x41;&#X42;"), "#AB");
        assert_eq!(decode_entities("&unknown; &amp"), "&unknown; &amp");
        assert_eq!(decode_entities("&#0;"), "&#0;");
        assert_eq!(decode_entities("&#55296;"), "&#55296;");
        assert_eq!(decode_entities("&#1114112;"), "&#1114112;");
        assert_eq!(decode_entities("&#128512;"), "😀");
        assert_eq!(decode_entities("&&amp;"), "&&");
    }

    #[test]
    fn placeholders() {
        assert_eq!(decode_entities("\u{fb02}\u{b0}quot\u{b6}\u{df}"), "\"");
        assert_eq!(decode_entities("\u{fb02}\u{b0}\u{b0}35\u{b6}\u{df}x"), "#x");
        assert_eq!(decode_entities("\u{fb02}\u{b0}nope\u{b6}\u{df}"), "&nope;");
        assert_eq!(decode_entities("\u{fb02}\u{b0}\u{b0}0\u{b6}\u{df}"), "\u{fb02}\u{b0}\u{b0}0\u{b6}\u{df}");
        // A `¶` without `ß` is skipped and the search goes on after it.
        assert_eq!(decode_entities("\u{fb02}\u{b0}a\u{b6}x \u{fb02}\u{b0}amp\u{b6}\u{df}"), "\u{fb02}\u{b0}a\u{b6}x &");
    }
}
