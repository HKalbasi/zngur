#[rustfmt::skip]
mod generated;

pub enum Merged {
    First(i32, i32),
    Second(i32, i32),
    Third,
}

impl Merged {
    pub fn first() -> Self {
        Self::First(42, 24)
    }

    pub fn second() -> Self {
        Self::Second(24, 42)
    }
}
