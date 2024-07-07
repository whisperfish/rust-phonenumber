use phonenumber::parse;
use proptest::prelude::*;

proptest! {
    #[test]
    fn doesnt_crash(s in "\\PC*") {
        let _ = parse(None, &s);
    }

    #[test]
    fn parse_belgian_phonenumbers(s in "\\+32[0-9]{8,9}") {
        let parsed = parse(None, &s).expect("valid Belgian number");
        prop_assert_eq!(parsed.country().id(), phonenumber::country::BE.into());
    }
}
