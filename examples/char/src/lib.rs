#[rustfmt::skip]
mod generated;

struct CharPrinter;

impl CharPrinter {
    fn print(c: char) {
        println!("Rust received char: '{}' (U+{:04X})", c, c as u32);
    }

    fn is_alphabetic(c: char) -> bool {
        c.is_alphabetic()
    }

    fn to_uppercase(c: char) -> char {
        c.to_uppercase().next().unwrap_or(c)
    }
}
