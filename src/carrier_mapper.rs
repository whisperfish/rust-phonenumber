//! Carrier name lookup from libphonenumber prefix data.
//!
//! This module provides carrier name resolution using the same prefix-matching
//! algorithm as Google's libphonenumber. The carrier data files in
//! `assets/carrier/` are compiled into a binary blob at build time and embedded
//! in the crate.
//!
//! Carrier data is available in multiple languages. The lookup functions accept
//! a language code (e.g. `"en"`, `"zh"`, `"zh_Hant"`) and fall back to English
//! when no translation is available (except for Chinese, Japanese, and Korean).
//!
//! # Example
//!
//! ```
//! use phonenumber::carrier_mapper;
//!
//! let number = phonenumber::parse(None, "+33788439407").unwrap();
//! assert_eq!(carrier_mapper::name_for_number(&number, "en"), Some("Orange France"));
//! ```

use crate::metadata::DATABASE;
use crate::PhoneNumber;
use crate::Type;
use fnv::FnvHashMap;
use once_cell::sync::Lazy;

const CARRIER_DATA_BIN: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/carrier_data.bin"));

struct CarrierDatabase {
    /// prefix → { lang → carrier name }
    entries: FnvHashMap<String, FnvHashMap<String, String>>,
    max_prefix_len: usize,
}

static DB: Lazy<CarrierDatabase> = Lazy::new(|| {
    type CarrierEntries = Vec<(String, Vec<(String, String)>)>;
    let (entries_vec, max_prefix_len): (CarrierEntries, usize) =
        postcard::from_bytes(CARRIER_DATA_BIN).expect("failed to deserialize carrier data");
    let entries: FnvHashMap<String, FnvHashMap<String, String>> = entries_vec
        .into_iter()
        .map(|(prefix, langs)| (prefix, langs.into_iter().collect()))
        .collect();
    CarrierDatabase {
        entries,
        max_prefix_len,
    }
});

/// Locale normalization map, matching libphonenumber behavior.
fn normalize_locale(locale: &str) -> Option<&'static str> {
    match locale {
        "zh_TW" | "zh_HK" | "zh_MO" => Some("zh_Hant"),
        _ => None,
    }
}

/// Returns `false` for Chinese, Japanese, and Korean — these languages do not
/// fall back to English when no translation is available.
fn may_fall_back_to_english(lang: &str) -> bool {
    lang != "zh" && lang != "ja" && lang != "ko"
}

/// Look up the best available translation in a per-prefix language map.
///
/// Follows the libphonenumber locale resolution order:
/// 1. Normalized full locale (e.g. `zh_TW` → `zh_Hant`)
/// 2. Full locale as provided
/// 3. Language + script (if the locale contains both)
/// 4. Language + region
/// 5. Bare language code
/// 6. English fallback (except for zh, ja, ko)
fn find_lang<'a>(lang_map: &'a FnvHashMap<String, String>, locale: &str) -> Option<&'a str> {
    // Check normalization map.
    if let Some(normalized) = normalize_locale(locale) {
        if let Some(name) = lang_map.get(normalized) {
            return Some(name.as_str());
        }
    }

    // Full locale as-is.
    if let Some(name) = lang_map.get(locale) {
        return Some(name.as_str());
    }

    // Extract the bare language code (first component before '_').
    let lang = locale.split('_').next().unwrap_or(locale);

    // If the locale has multiple parts, try progressively shorter variants.
    let parts: Vec<&str> = locale.splitn(3, '_').collect();
    if parts.len() == 3 {
        // Try lang_script (e.g. "zh_Hant" from "zh_Hant_TW").
        let lang_script = format!("{}_{}", parts[0], parts[1]);
        if let Some(name) = lang_map.get(&lang_script) {
            return Some(name.as_str());
        }
        // Try lang_region (e.g. "zh_TW" from "zh_Hant_TW").
        let lang_region = format!("{}_{}", parts[0], parts[2]);
        if let Some(name) = lang_map.get(&lang_region) {
            return Some(name.as_str());
        }
    }

    // Bare language.
    if lang != locale {
        if let Some(name) = lang_map.get(lang) {
            return Some(name.as_str());
        }
    }

    // English fallback (not for CJK).
    if may_fall_back_to_english(lang) {
        if let Some(name) = lang_map.get("en") {
            return Some(name.as_str());
        }
    }

    None
}

/// Returns the carrier name for a valid phone number in the given language.
///
/// This skips the number type check — the caller is responsible for ensuring
/// the number is suitable for carrier lookup (mobile, fixed-line-or-mobile,
/// or pager).
///
/// Returns `None` if no carrier data exists for this number's prefix.
pub fn name_for_valid_number(number: &PhoneNumber, lang: &str) -> Option<&'static str> {
    let e164 = format!("{}{}", number.code().value(), number.national());
    let db = &*DB;

    let max = db.max_prefix_len.min(e164.len());
    for len in (1..=max).rev() {
        if let Some(lang_map) = db.entries.get(&e164[..len]) {
            return find_lang(lang_map, lang);
        }
    }
    None
}

/// Returns the carrier name for a phone number in the given language.
///
/// Carrier lookup only applies to mobile, fixed-line-or-mobile, and pager
/// numbers. For other types, this returns `None`.
///
/// The carrier name is the one the number was **originally allocated to**.
/// In countries with mobile number portability, the number may have been
/// ported to a different carrier.
///
/// Falls back to English when no translation exists for the requested
/// language, except for Chinese, Japanese, and Korean.
pub fn name_for_number(number: &PhoneNumber, lang: &str) -> Option<&'static str> {
    let ntype = number.number_type(&DATABASE);
    if is_carrier_type(ntype) {
        name_for_valid_number(number, lang)
    } else {
        None
    }
}

/// Returns the carrier name only when it is safe to display to users.
///
/// A carrier name is considered safe if the number's country does **not**
/// support mobile number portability. In countries with portability (most
/// of Europe, US, Brazil, etc.), the originally allocated carrier may no
/// longer be accurate, so this function returns `None`.
///
/// See <https://en.wikipedia.org/wiki/Mobile_number_portability>.
pub fn safe_display_name(number: &PhoneNumber, lang: &str) -> Option<&'static str> {
    if let Some(meta) = number.metadata(&DATABASE) {
        if meta.is_mobile_number_portable() {
            return None;
        }
    }
    name_for_number(number, lang)
}

fn is_carrier_type(ntype: Type) -> bool {
    matches!(ntype, Type::Mobile | Type::FixedLineOrMobile | Type::Pager)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;

    fn parsed(s: &str) -> PhoneNumber {
        parser::parse(None, s).unwrap()
    }

    // ---- French carriers (basic, English) ----

    #[test]
    fn fr_orange() {
        assert_eq!(name_for_number(&parsed("+33788000000"), "en"), Some("Orange France"));
    }

    #[test]
    fn fr_sfr() {
        assert_eq!(name_for_number(&parsed("+33601000000"), "en"), Some("SFR"));
    }

    #[test]
    fn fr_free() {
        assert_eq!(name_for_number(&parsed("+33600000000"), "en"), Some("Free Mobile"));
    }

    #[test]
    fn fr_bouygues() {
        assert_eq!(name_for_number(&parsed("+33650000000"), "en"), Some("Bouygues"));
    }

    // ---- Overlapping prefixes (longest wins) ----

    #[test]
    fn overlap_afone_over_orange() {
        assert_eq!(name_for_number(&parsed("+33780123456"), "en"), Some("Afone"));
    }

    #[test]
    fn overlap_laposte_over_orange() {
        assert_eq!(
            name_for_number(&parsed("+33784612345"), "en"),
            Some("La poste telecom")
        );
    }

    #[test]
    fn overlap_lebara_over_afone_over_orange() {
        assert_eq!(
            name_for_number(&parsed("+33780712345"), "en"),
            Some("Lebara France Limited")
        );
    }

    #[test]
    fn fallback_to_shortest_when_no_longer_match() {
        assert_eq!(
            name_for_number(&parsed("+33789000000"), "en"),
            Some("Orange France")
        );
    }

    // ---- International carriers ----

    #[test]
    fn belgium_proximus() {
        assert_eq!(name_for_number(&parsed("+32470000000"), "en"), Some("Proximus"));
    }

    #[test]
    fn romania_vodafone() {
        assert_eq!(name_for_number(&parsed("+40721234567"), "en"), Some("Vodafone"));
    }

    #[test]
    fn hong_kong_china_mobile() {
        assert_eq!(
            name_for_number(&parsed("+85244012345"), "en"),
            Some("China Mobile")
        );
    }

    #[test]
    fn uk_jersey_telecom() {
        assert_eq!(name_for_number(&parsed("+447911123456"), "en"), Some("JT"));
    }

    #[test]
    fn spain_vodafone() {
        assert_eq!(name_for_number(&parsed("+34666777888"), "en"), Some("Vodafone"));
    }

    #[test]
    fn italy_tim() {
        assert_eq!(name_for_number(&parsed("+393331234567"), "en"), Some("TIM"));
    }

    #[test]
    fn brazil_vivo() {
        assert_eq!(
            name_for_number(&parsed("+5511954720000"), "en"),
            Some("Vivo")
        );
    }

    // ---- Multi-language support ----

    #[test]
    fn chinese_carrier_in_chinese() {
        // China Unicom in Simplified Chinese
        assert_eq!(
            name_for_number(&parsed("+8613000000000"), "zh"),
            Some("中国联通")
        );
    }

    #[test]
    fn chinese_carrier_in_english() {
        assert_eq!(
            name_for_number(&parsed("+8613000000000"), "en"),
            Some("China Unicom")
        );
    }

    #[test]
    fn chinese_no_english_fallback() {
        // zh does NOT fall back to English for numbers without zh data
        // FR numbers have no Chinese translation
        assert_eq!(name_for_number(&parsed("+33788000000"), "zh"), None);
    }

    #[test]
    fn korean_no_english_fallback() {
        // ko does NOT fall back to English for numbers without ko data
        assert_eq!(name_for_number(&parsed("+33788000000"), "ko"), None);
    }

    #[test]
    fn french_falls_back_to_english() {
        // "fr" is not in the carrier data — should fall back to "en"
        assert_eq!(
            name_for_number(&parsed("+33788000000"), "fr"),
            Some("Orange France")
        );
    }

    #[test]
    fn russian_carrier_in_russian() {
        assert_eq!(
            name_for_number(&parsed("+79200000000"), "ru"),
            Some("МегаФон")
        );
    }

    #[test]
    fn russian_carrier_in_english() {
        assert_eq!(
            name_for_number(&parsed("+79200000000"), "en"),
            Some("MegaFon")
        );
    }

    // ---- Locale normalization ----

    #[test]
    fn zh_tw_normalizes_to_zh_hant() {
        // zh_TW should be normalized to zh_Hant
        let n = parsed("+8613000000000");
        let result_hant = name_for_number(&n, "zh_Hant");
        let result_tw = name_for_number(&n, "zh_TW");
        assert_eq!(result_hant, result_tw);
    }

    #[test]
    fn zh_hk_normalizes_to_zh_hant() {
        let n = parsed("+8613000000000");
        let result_hant = name_for_number(&n, "zh_Hant");
        let result_hk = name_for_number(&n, "zh_HK");
        assert_eq!(result_hant, result_hk);
    }

    // ---- Type filtering ----

    #[test]
    fn fixed_line_rejected() {
        assert_eq!(name_for_number(&parsed("+4930123456"), "en"), None);
    }

    #[test]
    fn toll_free_international_rejected() {
        assert_eq!(name_for_number(&parsed("+80012340000"), "en"), None);
    }

    #[test]
    fn toll_free_french_rejected() {
        assert_eq!(name_for_number(&parsed("+33800123456"), "en"), None);
    }

    #[test]
    fn us_no_carrier_data() {
        assert_eq!(name_for_number(&parsed("+12025551234"), "en"), None);
    }

    // ---- name_for_valid_number bypasses type check ----

    #[test]
    fn valid_number_skips_type_filter() {
        let n = parsed("+4930123456");
        let _ = name_for_valid_number(&n, "en");
    }

    // ---- safe_display_name ----

    #[test]
    fn safe_display_returns_none_for_portable_country() {
        // France supports mobile portability → safe_display_name returns None
        let n = parsed("+33788000000");
        assert_eq!(safe_display_name(&n, "en"), None);
    }

    #[test]
    fn safe_display_returns_none_for_us() {
        // US supports mobile portability
        let n = parsed("+12025551234");
        assert_eq!(safe_display_name(&n, "en"), None);
    }

    // ---- Database sanity checks ----

    #[test]
    fn database_is_loaded() {
        let db = &*DB;
        assert!(
            db.entries.len() > 20_000,
            "expected >20K entries, got {}",
            db.entries.len()
        );
    }

    #[test]
    fn max_prefix_len_is_reasonable() {
        let db = &*DB;
        assert!(
            db.max_prefix_len >= 7 && db.max_prefix_len <= 12,
            "unexpected max_prefix_len: {}",
            db.max_prefix_len
        );
    }

    // ---- is_carrier_type ----

    #[test]
    fn carrier_type_mobile() {
        assert!(is_carrier_type(Type::Mobile));
    }

    #[test]
    fn carrier_type_fixed_line_or_mobile() {
        assert!(is_carrier_type(Type::FixedLineOrMobile));
    }

    #[test]
    fn carrier_type_pager() {
        assert!(is_carrier_type(Type::Pager));
    }

    #[test]
    fn not_carrier_type_fixed_line() {
        assert!(!is_carrier_type(Type::FixedLine));
    }

    #[test]
    fn not_carrier_type_toll_free() {
        assert!(!is_carrier_type(Type::TollFree));
    }

    #[test]
    fn not_carrier_type_voip() {
        assert!(!is_carrier_type(Type::Voip));
    }

    #[test]
    fn not_carrier_type_premium() {
        assert!(!is_carrier_type(Type::PremiumRate));
    }
}
