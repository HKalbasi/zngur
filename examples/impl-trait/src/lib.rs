#![allow(dead_code)]

#[rustfmt::skip]
mod generated;

use std::fmt::Display;

// A simple trait for demonstration
pub trait Greeter {
    fn greet(&self) -> String;
}

// Concrete type implementing the trait
#[derive(Debug, Clone)]
pub struct Person {
    name: String,
}

impl Person {
    pub fn new(name: String) -> Self {
        Person { name }
    }
}

impl Greeter for Person {
    fn greet(&self) -> String {
        format!("Hello, I'm {}!", self.name)
    }
}

// Another concrete type implementing the trait
#[derive(Debug, Clone)]
pub struct Robot {
    id: u32,
}

impl Robot {
    pub fn new(id: u32) -> Self {
        Robot { id }
    }
}

impl Greeter for Robot {
    fn greet(&self) -> String {
        format!("Beep boop! Robot #{}", self.id)
    }
}

// Function using impl Trait in argument position
// For FFI, this is converted to Box<dyn Trait> which implements Greeter via deref coercion
pub fn print_greeting(greeter: impl Greeter) {
    println!("{}", greeter.greet());
}

// Concrete implementations for testing impl Trait in argument position
pub fn print_greeting_person(person: Person) {
    print_greeting(person);
}

pub fn print_greeting_robot(robot: Robot) {
    print_greeting(robot);
}

// Function returning impl Trait
// This should fail initially, then we'll make it work with Box<dyn Trait>
pub fn create_person(name: String) -> impl Greeter {
    Person::new(name)
}

// Another function returning impl Trait
pub fn create_robot(id: u32) -> impl Greeter {
    Robot::new(id)
}

// Function that conditionally returns different types implementing the trait
// This is the tricky case that requires Box<dyn Trait>
pub fn create_greeter_by_type(is_person: bool, name: String, id: u32) -> Box<dyn Greeter> {
    if is_person {
        Box::new(Person::new(name))
    } else {
        Box::new(Robot::new(id))
    }
}

impl Display for Person {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Person({})", self.name)
    }
}

impl Display for Robot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Robot(#{})", self.id)
    }
}
