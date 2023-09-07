use std::f64::consts::PI;

use bitflags::bitflags;
use iter_tools::dependency::itertools::iproduct;

mod generated;

struct Reader(generated::ZngurCppOpaqueOwnedObject);
struct Way(generated::ZngurCppOpaqueBorrowedObject);
struct WayNodeList(generated::ZngurCppOpaqueBorrowedObject);
struct Node(generated::ZngurCppOpaqueBorrowedObject);
struct TagList(generated::ZngurCppOpaqueBorrowedObject);

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
        if !matches!(
            way.tags().get_value_by_key("highway"),
            Some("trunk" | "primary" | "secondary" | "tertiary")
        ) {
            return;
        }
        println!("node {}", self.count);
        let nodes = way.nodes();
        for middle_id in 0..nodes.len() {
            let middle_node = nodes.get(middle_id);
            let left_bound = (0..middle_id)
                .rev()
                .find(|x| nodes.get(*x).distance(middle_node) > 50.)
                .map(|x| x + 1)
                .unwrap_or(0);
            let right_bound = (middle_id..nodes.len())
                .find(|x| nodes.get(*x).distance(middle_node) > 50.)
                .unwrap_or(nodes.len());
            let min_angle = iproduct!(left_bound..middle_id, middle_id + 1..right_bound)
                .map(|(left_id, right_id)| {
                    let dist_right = nodes.get(left_id).distance(middle_node);
                    let dist_left = nodes.get(right_id).distance(middle_node);
                    let dist_middle = nodes.get(right_id).distance(nodes.get(left_id));
                    let cos_angle = (dist_right * dist_right + dist_left * dist_left
                        - dist_middle * dist_middle)
                        / (2. * dist_right * dist_left);
                    (cos_angle.acos() * 180. / PI).round() as i32
                })
                .min()
                .unwrap_or(180);
            if min_angle < 135 {
                println!(
                    "Dangerous node {} with angle {}",
                    middle_node.href(),
                    min_angle
                );
            }
        }
    }
}

fn main() {
    let f = Flags::way | Flags::node;
    let reader = generated::new_blob_store_client(f);
    generated::apply(&reader, BendHandler { count: 0 });
}
