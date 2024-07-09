use phonenumber::parse;
use proptest::prelude::*;

proptest! {
    #[test]
    fn rfc3966_crash_test(
        tel_prefix: bool,
        use_plus: bool,
        s: String,
        phone_context: Option<String>,
    ) {
        let context = if let Some(phone_context) = &phone_context { format!(";phone-context={phone_context}") } else { "".to_string() };
        let tel_prefix = if tel_prefix { "tel:" } else { "" };
        let plus = if use_plus { "+" } else { "" };
        let s = format!("{}{}{}{}", tel_prefix, plus, s, context);
        let _ = parse(None, &s);
    }

    #[test]
    fn doesnt_crash(s in "\\PC*") {
        let _ = parse(None, &s);
    }

    #[test]
    fn doesnt_crash_2(s in "\\+\\PC*") {
        let _ = parse(None, &s);
    }

    #[test]
    fn parse_belgian_phonenumbers(s in "\\+32[0-9]{8,9}") {
        let parsed = parse(None, &s).expect("valid Belgian number");
        prop_assert_eq!(parsed.country().id(), phonenumber::country::BE.into());
    }
}
