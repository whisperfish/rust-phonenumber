use std::env;

use phonenumber::Mode;

fn main() -> Result<(), String> {
    let mut args = env::args().skip(1).collect::<Vec<_>>();

    let (country, number_string) = match (args.pop(), args.pop()) {
        (Some(number), None) => (None, number),
        (Some(number), Some(country)) => {
            let country: phonenumber::country::Id = country
                .parse()
                .map_err(|e| format!("invalid country code: {e}"))?;

            (Some(country), number)
        }
        _ => return Err("usage: test [country] <number>".to_owned()),
    };

    let number =
        phonenumber::parse(country, number_string).map_err(|e| format!("could not parse: {e}"))?;

    let valid = phonenumber::is_valid(&number);

    if valid {
        println!("\x1b[32m{:#?}\x1b[0m", number);
        println!();
        println!(
            "International: {}",
            number.format().mode(Mode::International)
        );
        println!("     National: {}", number.format().mode(Mode::National));
        println!("      RFC3966: {}", number.format().mode(Mode::Rfc3966));
        println!("        E.164: {}", number.format().mode(Mode::E164));
    } else {
        println!("\x1b[31m{:#?}\x1b[0m", number);
    }

    Ok(())
}
