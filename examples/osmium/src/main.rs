use bitflags::bitflags;

mod generated;

struct Reader(generated::ZngurCppOpaqueOwnedObject);
struct Way(generated::ZngurCppOpaqueOwnedObject);

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct Flags: u8 {
        const nothing   = 0x00;
        const node      = 0x01;
        const way       = 0x02;
        const relation  = 0x04;
        const nwr       = 0x07;
        const area      = 0x08;
        const object    = 0x0f;
        const changeset = 0x10;
        const ALL       = 0x1f;
    }
}

trait Handler {
    fn way(&mut self, way: &Way);
}

struct BendHandler {
    count: usize,
}

impl Handler for BendHandler {
    fn way(&mut self, way: &Way) {
        self.count += 1;
        print!("Node {}: ", self.count);
    }
}

fn main() {
    let f = Flags::way | Flags::node;
    let reader = generated::new_blob_store_client(f);
    generated::apply(&reader, BendHandler { count: 0 });
    println!("Hello, world! {}", f.bits());
}
