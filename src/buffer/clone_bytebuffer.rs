use std::cell::RefCell;
use crate::buffer::buffer::{IBuffer, Buffer, ByteBuffer};

#[derive(Debug, Clone)]
pub struct CloneByteBuffer {
    pub buffer: ByteBuffer,
    // use RefCell for multiple slice buffer to share the same underlying buf
    pub hb: RefCell<Vec<u8>>,
    pub offset: i32,
}

impl IBuffer for CloneByteBuffer {
    fn mark(&self) -> i32 {
        self.buffer.mark()
    }

    fn cap(&self) -> i32 {
        self.buffer.cap()
    }

    fn position(&self) -> i32 {
        self.buffer.position()
    }

    fn limit(&self) -> i32 {
        self.buffer.limit()
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
        self.buffer.clear();
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
        self.buffer.get()
    }
}

impl CloneByteBuffer {
    pub fn new(buf: &[u8], mark: i32, pos: i32, limit: i32, cap: i32, off: i32) -> Self {
        let buffer = ByteBuffer::new_(mark, pos, limit, cap);
        Self {
            buffer,
            hb: RefCell::new(buf.to_vec()),
            offset: 0,
        }
    }

    pub fn new2(cap: i32, limit: i32) -> Self {
        let buffer = ByteBuffer::new_(-1, 0, limit, cap);
        let mut buf = Vec::with_capacity(cap as usize);
        for _ in 0..cap {
            buf.push(0);
        }
        Self {
            buffer,
            hb: RefCell::new(buf.to_vec()),
            offset: 0,
        }
    }

    pub fn new3(buf: &[u8], off: i32, len: i32) -> Self {
        let buffer = ByteBuffer::new_(-1, off, off + len, buf.len() as i32);
        Self {
            buffer: buffer,
            hb: RefCell::new(buf.to_vec()),
            offset: 0,
        }
    }

    pub fn new_(buffer: ByteBuffer, hb: RefCell<Vec<u8>>, offset: i32) -> Self {
        Self {
            buffer, hb, offset
        }
    }

    // todo: the result of RefCell clone is not expected: we want to change the slice and also change the parent buffer.
    // but use clone() here will only change the slice hb buffer, not changing the parent buffer.
    pub fn slice(&self) -> Self {
        let buffer = ByteBuffer::new_(-1, 0, self.buffer.remaining(), self.buffer.remaining());
        Self {
            buffer,
            hb: self.hb.clone(),
            offset: self.buffer.position() + self.offset,
        }
    }

    pub fn duplicate(self) -> Self {
        Self {
            buffer: self.buffer,
            hb: self.hb,
            offset: self.offset,
        }
    }

    pub fn ix(&self, i: i32) -> i32 {
        i + self.offset
    }

    pub fn get(&mut self) -> u8 {
        let idx = self.buffer.buffer.next_get_index();
        self.get_idx_(idx)
    }

    pub fn get_i(&mut self, i: i32) -> u8 {
        let idx = self.buffer.buffer.check_index(i);
        self.get_idx_(idx)
    }

    fn get_idx_(&mut self, i: i32) -> u8 {
        let ix = self.ix(i) as usize;
        let mut hb = self.hb.get_mut();
        hb[ix]
    }

    pub fn put(&mut self, x: u8) {
        let next_get_index = self.buffer.buffer.next_put_index();
        self.put_i(x, next_get_index)
    }

    pub fn put_i(&mut self, x: u8, i: i32) {
        let idx = self.buffer.buffer.check_index(i);
        self.put_idx_(x, idx)
    }

    fn put_idx_(&mut self, x: u8, idx: i32) {
        let ix = self.ix(idx) as usize;
        let mut hb = self.hb.get_mut();
        hb[ix] = x;
    }

    // todo: batch copy?
    // System.arraycopy(hb, ix(position()), dst, offset, length);
    // buf.append(hb[src_start..src_start+length]);
    // buf[offset..offset+length] = hb[src_start..src_start+length];
    ///
    /// Get buf from HeapByteBuffer(source), copy to destination vec
    /// - source start: current HeapByteBuffer's position
    /// - destination start: offset
    ///
    pub fn get_buf(&mut self, dst: &mut Vec<u8>, offset: i32, length: i32) -> &mut Self {
        Buffer::check_bounds(offset, length, dst.len() as i32);
        if length > self.remaining() {
            panic!("buffer under flow")
        }
        let src_start = self.ix(self.position()) as usize;
        let mut hb = self.hb.get_mut();
        let mut idx = 0;
        for i in offset..offset + length {
            let id = i as usize;
            dst[id] = hb[src_start+idx];
            idx += 1;
        }
        assert_eq!(idx, length as usize);
        self.position_(self.position() + length);
        self
    }

    /// Put buf from source vector, to HeapByteBuffer
    /// - source start: offset
    /// - destination start: current HeapByteBuffer's position
    pub fn put_buf(&mut self, src: &mut Vec<u8>, offset: i32, length: i32) -> &mut Self {
        Buffer::check_bounds(offset, length, src.len() as i32);
        if length > self.remaining() {
            panic!("buffer under flow")
        }
        let dst_start = self.ix(self.position()) as usize;
        let mut hb = self.hb.get_mut();
        let mut idx = 0;
        for i in offset..offset + length {
            let id = i as usize;
            hb[dst_start +idx] = src[id];
            idx += 1;
        }
        // assert_eq!(idx+1, length as usize);
        self.position_(self.position() + length);
        self
    }

    // System.arraycopy(sb.hb, sb.ix(sb.position()), hb, ix(position()), n);
    ///
    /// Put destination HeapByteBuffer to current HeapByteBuffer
    /// - source start: destination HeapByteBuffer's position
    /// - destination start: current HeapByteBuffer's position
    pub fn put_buffer(&mut self, heap_buffer: &mut CloneByteBuffer) {
        // let mut heap_buffer = buffer as HeapByteBuffer;
        let n = heap_buffer.remaining() as usize;
        if n > self.remaining() as usize {
            panic!("buffer overflow")
        }

        // make sure immutable invoke execute first. else have conflict problem.
        let src_start = heap_buffer.ix(heap_buffer.position()) as usize;
        let dst_start = self.ix(self.position()) as usize;

        // mutable buf vector
        let mut src_hb = heap_buffer.hb.get_mut();
        let mut hb = self.hb.get_mut();

        // copy from src_hb's src_start to hb's dst_start
        let mut idx = 0;
        for i in src_start..src_start + n {
            let id = i as usize;
            hb[dst_start+idx] = src_hb[id];
            idx += 1;
        }
        // update src and dst position
        heap_buffer.position_(heap_buffer.position() + n as i32);
        self.position_(self.position() + n as i32);
    }

}