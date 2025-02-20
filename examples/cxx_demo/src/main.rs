#[rustfmt::skip]
mod generated;

use generated::new_blob_store_client;

// An iterator over contiguous chunks of a discontiguous file object. Toy
// implementation uses a Vec<Vec<u8>> but in reality this might be iterating
// over some more complex Rust data structure like a rope, or maybe loading
// chunks lazily from somewhere.
pub struct MultiBuf {
    chunks: Vec<Vec<u8>>,
    pos: usize,
}

impl MultiBuf {
    pub fn next_chunk(&mut self) -> &[u8] {
        let next = self.chunks.get(self.pos);
        self.pos += 1;
        next.map_or(&[], Vec::as_slice)
    }
}

#[derive(Debug, Default)]
struct BlobMetadata {
    size: usize,
    tags: Vec<String>,
}

impl BlobMetadata {
    fn set_size(&mut self, size: usize) {
        self.size = size;
    }

    fn push_tag(&mut self, c: *const i8) {
        self.tags.push(
            String::from_utf8_lossy(unsafe { std::ffi::CStr::from_ptr(c).to_bytes() }).to_string(),
        );
    }
}

trait BlobStoreTrait {
    fn put(&self, buf: &mut MultiBuf) -> u64;
    fn tag(&self, blob_id: u64, tag: &str);
    fn metadata(&self, blob_id: u64) -> BlobMetadata;
}

fn main() {
    let client = new_blob_store_client();

    // Upload a blob_.
    let chunks = vec![b"fearless".to_vec(), b"concurrency".to_vec()];
    let mut buf = MultiBuf { chunks, pos: 0 };
    let blob_id = client.put(&mut buf);
    println!("blob_id = {}", blob_id);

    // Add a tag.
    client.tag(blob_id, "rust");

    // Read back the tags.
    let metadata = client.metadata(blob_id);
    println!("metadata = {:?}", metadata);
}
