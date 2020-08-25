use crate::buffer::buffer::{Buffer, IBuffer};

#[derive(Debug, Clone)]
pub struct ByteBuffer {
    pub buffer: Buffer,
    pub read_only: bool,
}

impl ByteBuffer {
    #[deprecated]
    pub fn default(&mut self, mark: i32, pos: i32, limit: i32, cap: i32) {
        self.new(mark, pos, limit, cap, 0)
    }

    #[deprecated]
    pub fn new(&mut self, mark: i32, pos: i32, limit: i32, cap: i32, offset: i32) {
        let mut buffer = Buffer::default();
        buffer.new(mark, pos, limit, cap);
        self.buffer = buffer;
        // self.offset = offset;
    }

    pub fn new_(mark: i32, pos: i32, limit: i32, cap: i32) -> Self {
        let mut buffer = Buffer::new_(mark, pos, limit, cap);
        buffer.init();
        Self {
            buffer,
            read_only: false,
        }
    }
}

impl IBuffer for ByteBuffer {
    fn mark(&self) -> i32 {
        self.buffer.mark
    }

    fn cap(&self) -> i32 {
        self.buffer.cap
    }

    fn position(&self) -> i32 {
        self.buffer.position
    }

    fn limit(&self) -> i32 {
        self.buffer.limit
    }

    fn reset(&mut self) -> &mut Self {
        self.buffer.reset();
        self
    }

    fn limit_(&mut self, limit: i32) -> &mut Self {
        self.buffer.limit_(limit);
        self
    }

    fn position_(&mut self, position: i32) -> &mut Self {
        self.buffer.position_(position);
        self
    }

    fn mark_(&mut self) -> &mut Self {
        self.buffer.mark_();
        self
    }

    fn clear(&mut self) -> &mut Self {
        self.buffer.clear();
        self
    }

    fn truncate(&mut self) {
        self.buffer.truncate()
    }

    fn flip(&mut self) -> &mut Self {
        self.buffer.flip();
        self
    }

    fn rewind(&mut self) -> &mut Self {
        self.buffer.rewind();
        self
    }

    fn slice(&self) -> &Self {
        self.buffer.slice();
        self
    }

    fn get(&mut self) -> u8 {
        unimplemented!()
    }
}