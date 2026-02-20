mod generated;

pub(crate) struct CharPrinter;

impl CharPrinter {
    pub(crate) fn print(c: char) {
        println!("Rust received char: '{}' (U+{:04X})", c, c as u32);
    }

    pub(crate) fn is_alphabetic(c: char) -> bool {
        c.is_alphabetic()
    }

    pub(crate) fn to_uppercase(c: char) -> char {
        c.to_uppercase().next().unwrap_or(c)
    }
}
