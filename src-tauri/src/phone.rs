use percent_encoding::percent_decode_str;
use rlibphonenumber::{PhoneNumber, PhoneNumberFormat, Region};

/// Returns the digits-only international number (no "+") found in a
/// `tel:` / `callto:` link, or `None` when the link holds no valid number.
pub fn parse_number(link: &str, country_codes: &[String]) -> Option<String> {
    let link = link.trim();
    if let Some(phone) = whatsapp_phone(link) {
        // WhatsApp links hold the international number without "+".
        return parse_with(None, &format!("+{phone}"));
    }
    let rest = strip_scheme(link);
    let decoded = percent_decode_str(rest).decode_utf8_lossy();

    // Drop tel: parameters such as ";ext=12" or ";phone-context=..." and any query.
    let number_part = decoded.split([';', '?']).next().unwrap_or("");

    // Keep only a leading "+" and ASCII digits. This also removes spaces,
    // punctuation and invisible format characters (zero-width space/joiner,
    // BOM, LRM...) that sneak in when copying numbers from web pages and chats.
    let mut cleaned = String::new();
    for c in number_part.chars() {
        if c.is_ascii_digit() || (c == '+' && cleaned.is_empty()) {
            cleaned.push(c);
        }
    }

    if cleaned.starts_with('+') {
        return parse_with(None, &cleaned);
    }

    country_codes.iter().find_map(|code| {
        let region = code.to_uppercase().parse::<Region>().ok()?;
        parse_with(Some(region), &cleaned)
    })
}

/// True for `whatsapp:` links, which this app can also own as the handler.
pub fn is_whatsapp_scheme(link: &str) -> bool {
    link.trim()
        .get(..9)
        .is_some_and(|scheme| scheme.eq_ignore_ascii_case("whatsapp:"))
}

/// Extracts the digits of the number from `whatsapp://send?phone=`,
/// `https://api.whatsapp.com/send?phone=`, `https://web.whatsapp.com/send?phone=`
/// and `https://wa.me/<number>` links.
fn whatsapp_phone(link: &str) -> Option<String> {
    let lower = link.to_ascii_lowercase();
    let rest = lower
        .strip_prefix("whatsapp://")
        .or_else(|| lower.strip_prefix("whatsapp:"))
        .map(|rest| ("send", rest))
        .or_else(|| {
            let rest = lower
                .strip_prefix("https://")
                .or_else(|| lower.strip_prefix("http://"))?;
            let (host, path) = rest.split_once('/').unwrap_or((rest, ""));
            match host {
                "api.whatsapp.com" | "web.whatsapp.com" => Some(("send", path)),
                "wa.me" => Some(("wa.me", path)),
                _ => None,
            }
        })?;

    let (kind, rest) = rest;
    let (path, query) = rest.split_once('?').unwrap_or((rest, ""));
    let raw = if kind == "wa.me" {
        percent_decode_str(path.trim_matches('/'))
            .decode_utf8_lossy()
            .into_owned()
    } else {
        let value = query
            .split('&')
            .find_map(|pair| pair.strip_prefix("phone="))?;
        percent_decode_str(value).decode_utf8_lossy().into_owned()
    };
    let digits: String = raw.chars().filter(char::is_ascii_digit).collect();
    (!digits.is_empty()).then_some(digits)
}

fn strip_scheme(link: &str) -> &str {
    for scheme in ["tel:", "callto:"] {
        if link.len() >= scheme.len() && link[..scheme.len()].eq_ignore_ascii_case(scheme) {
            return link[scheme.len()..].trim_start_matches("//");
        }
    }
    link
}

fn parse_with(region: Option<Region>, number: &str) -> Option<String> {
    let parsed = PhoneNumber::parse(number, region).ok()?;
    if !parsed.is_valid() {
        return None;
    }
    Some(
        parsed
            .format_as(PhoneNumberFormat::E164)
            .to_string()
            .replace('+', ""),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn de_br() -> Vec<String> {
        vec!["DE".into(), "BR".into()]
    }

    #[test]
    fn parses_international_links() {
        assert_eq!(
            parse_number("tel:+4915112345678", &de_br()).as_deref(),
            Some("4915112345678")
        );
        assert_eq!(
            parse_number("callto:+55 11 91234-5678", &de_br()).as_deref(),
            Some("5511912345678")
        );
    }

    #[test]
    fn decodes_percent_encoding() {
        assert_eq!(
            parse_number("tel:%2B4915112345678", &de_br()).as_deref(),
            Some("4915112345678")
        );
        assert_eq!(
            parse_number("tel:+49%20151%2012345678", &de_br()).as_deref(),
            Some("4915112345678")
        );
    }

    #[test]
    fn uses_configured_country_codes() {
        assert_eq!(
            parse_number("tel:01511 2345678", &["DE".into()]).as_deref(),
            Some("4915112345678")
        );
        assert_eq!(
            parse_number("tel:(11) 91234-5678", &["BR".into()]).as_deref(),
            Some("5511912345678")
        );
    }

    #[test]
    fn accepts_non_mobile_numbers() {
        assert_eq!(
            parse_number("tel:+14155552671", &de_br()).as_deref(),
            Some("14155552671")
        );
        assert_eq!(
            parse_number("tel:+4930901820", &de_br()).as_deref(),
            Some("4930901820")
        );
    }

    #[test]
    fn ignores_tel_parameters() {
        assert_eq!(
            parse_number("tel:+4915112345678;ext=12", &de_br()).as_deref(),
            Some("4915112345678")
        );
    }

    #[test]
    fn parses_whatsapp_links() {
        for link in [
            "whatsapp://send?phone=491630014758",
            "whatsapp://send/?phone=491630014758&text=hi",
            "WhatsApp://send?text=hi&phone=%2B491630014758",
            "https://api.whatsapp.com/send/?phone=491630014758",
            "https://api.whatsapp.com/send?phone=491630014758&text=hello",
            "https://web.whatsapp.com/send?phone=491630014758",
            "https://wa.me/491630014758",
            "https://wa.me/491630014758?text=hi",
        ] {
            assert_eq!(
                parse_number(link, &de_br()).as_deref(),
                Some("491630014758"),
                "{link}"
            );
        }
    }

    #[test]
    fn rejects_whatsapp_links_without_number() {
        for link in [
            "whatsapp://send?text=hi",
            "https://wa.me/",
            "https://example.com/send?phone=491630014758",
        ] {
            assert_eq!(parse_number(link, &de_br()), None, "{link}");
        }
    }

    #[test]
    fn detects_whatsapp_scheme() {
        assert!(is_whatsapp_scheme("whatsapp://send?phone=1"));
        assert!(!is_whatsapp_scheme("tel:+4915112345678"));
        assert!(!is_whatsapp_scheme("https://wa.me/1"));
    }

    #[test]
    fn returns_none_on_junk() {
        for junk in ["tel:", "tel:abc", "tel:123", ""] {
            assert_eq!(parse_number(junk, &de_br()), None, "{junk}");
        }
    }

    #[test]
    fn ignores_invisible_characters() {
        for c in [
            '\u{200B}', '\u{200C}', '\u{200D}', '\u{FEFF}', '\u{200E}', '\u{00A0}',
        ] {
            assert_eq!(
                parse_number(&format!("tel:+49{c}151{c}12345678"), &de_br()).as_deref(),
                Some("4915112345678")
            );
            assert_eq!(
                parse_number(&format!("tel:{c}+4915112345678{c}"), &de_br()).as_deref(),
                Some("4915112345678")
            );
            assert_eq!(
                parse_number(&format!("tel:{c}01511 2345678"), &["DE".into()]).as_deref(),
                Some("4915112345678")
            );
        }
        assert_eq!(
            parse_number("tel:%E2%80%8B+49151%E2%80%8B12345678", &de_br()).as_deref(),
            Some("4915112345678")
        );
    }
}
