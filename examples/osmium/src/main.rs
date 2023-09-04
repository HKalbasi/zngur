use bitflags::bitflags;

mod generated;

struct Reader(generated::ZngurCppOpaqueObject);

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

fn main() {
    let f = Flags::way | Flags::node;
    let _reader = generated::new_blob_store_client(f);
    println!("Hello, world! {}", f.bits());
}
