use std::env;

extern crate phonenumber;
use phonenumber::Mode;

fn main() {
    let mut args = env::args().skip(1).collect::<Vec<_>>();

    if args.len() < 1 {
        panic!("not enough arguments");
    }

    let number = args.pop().unwrap();
    let country = args.pop().map(|c| c.parse().unwrap());

    let number = phonenumber::parse(country, number).unwrap();
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
}
