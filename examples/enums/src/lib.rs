#[rustfmt::skip]
mod generated;

#[derive(Debug)]
pub struct NonPrimitive(pub i32);

pub enum Merged {
    First(i32, i32),
    Second(i32, NonPrimitive),
    Third,
}

impl Merged {
    pub fn first() -> Self {
        Self::First(42, 24)
    }

    pub fn second() -> Self {
        Self::Second(24, NonPrimitive(42))
    }
}

// Test that enums behave properly inside of fields.
pub struct Container(pub Merged);
