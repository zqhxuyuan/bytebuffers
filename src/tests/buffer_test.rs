use std::ops::Range;
use std::cell::RefCell;
use crate::buffer::buffer::{Buffer, IBuffer};
use crate::buffer::bytebuffer::ByteBuffer;
use crate::buffer::heap_bytebuffer::HeapByteBuffer;

#[test]
fn test_buffer_new() {
    let mut buffer = Buffer::default();
    buffer.flip().clear().mark_();
    buffer.flip().clear().mark();

    let mut buffer = ByteBuffer::new_(0, 0, 1, 1);
    buffer.flip().clear().mark_();
    buffer.flip().clear().mark();

    let mut buffer = HeapByteBuffer::new2(10, 10);
    buffer.flip().clear().mark_();
    buffer.flip().clear().mark();

    for i in 0..5 {
        buffer.put(i);
    }
    buffer.flip();
    for i in 0..5 {
        let j = buffer.get();
        assert_eq!(i, j);
    }
}

#[test]
fn test_ops_vec() {
    let mut v = Vec::new();
    v.push(1);

    let mut v1 = v.clone();
    v1.push(2);

    println!("v1:{:?}", v1);
    println!("v:{:?}", v);

    let mut v2 = v1;
    v2.push(3);
    println!("v2:{:?}", v2);
    // println!("v1:{:?}", &v1);

    let mut v = vec![1,2,3,4,5];
    let mut v1 = &v[0..2];
    // v1[0] = 11;
    let mut v2 = v1.to_vec();
    v2[0] = 11;
    println!("v:{:?}", v);
    println!("v1:{:?}", v1);
    println!("v2:{:?}", v2);

}

#[test]
fn test_buffer_slice() {
    let mut buffer = HeapByteBuffer::new2(10, 10);
    for i in 0..5 {
        buffer.put(i);
    }
    // println!("buffer  puts {:?}", &buffer);
    assert_eq!(buffer.position(), 5);
    assert_eq!(buffer.offset, 0);
    assert_eq!(buffer.hb, RefCell::new(vec![0, 1, 2, 3, 4, 0, 0, 0, 0, 0]));

    let mut slice = buffer.slice();
    // println!("buffer slice {:?}", &slice);
    assert_eq!(slice.position(), 0);
    assert_eq!(slice.offset, 5);
    assert_eq!(slice.limit(), 5);
    assert_eq!(slice.cap(), 5);
    assert_eq!(slice.hb, RefCell::new(vec![0, 1, 2, 3, 4, 0, 0, 0, 0, 0]));

    // slice.flip();
    for i in 10..12 {
        slice.put(i);
    }
    // println!("slice   puts {:?}", &slice);
    assert_eq!(slice.position(), 2);
    assert_eq!(slice.offset, 5);
    assert_eq!(slice.hb, RefCell::new(vec![0, 1, 2, 3, 4, 10, 11, 0, 0, 0]));

    for i in 20..23 {
        buffer.put(i);
    }
    // println!("buffer ##### {:?}", &buffer);
    assert_eq!(buffer.position(), 8);
    assert_eq!(buffer.offset, 0);
    assert_eq!(buffer.hb, RefCell::new(vec![0, 1, 2, 3, 4, 20, 21, 22, 0, 0]));
}

#[test]
fn test_buffer_slice_get() {
    let mut buffer = HeapByteBuffer::new2(10, 10);
    for i in 0..5 {
        buffer.put(i);
    }
    let mut slice = buffer.slice();
    assert_eq!(buffer.position(), 5);
    assert_eq!(slice.position(), 0);
    assert_eq!(slice.hb, RefCell::new(vec![0, 1, 2, 3, 4, 0, 0, 0, 0, 0]));

    // let s1 = slice.get();
    // assert_eq!(buffer.position(), 5);
    // assert_eq!(s1, 0);
    // assert_eq!(slice.position(), 1);
    // assert_eq!(slice.hb, RefCell::new(vec![0, 1, 2, 3, 4, 0, 0, 0, 0, 0]));
    //
    // slice.flip();
    // let s1 = slice.get();
    // assert_eq!(s1, 0);
    // assert_eq!(slice.position(), 1);
    // assert_eq!(buffer.position(), 5);

    // buffer.put(5);
    // buffer.put(6);
    // assert_eq!(slice.position(), 1);
    // assert_eq!(buffer.position(), 7);

    // slice.flip();
    // let s1 = slice.get();
    // println!("{}", s1);
    println!("{:?}", slice);
    println!("{:?}", buffer);


    // slice.put(10);
    // slice.put(11);
    // slice.put(12);
    // println!("{:?}", slice);
    // println!("{:?}", buffer);

}

#[test]
fn test_buffer_get_put() {
    let mut v: Vec<u8> = Vec::default();
    for _ in 0..5 {
        v.push(0);
    }

    // get_buf: 从HeapByteBuffer中读取数据，放入目标vec
    let mut buffer = HeapByteBuffer::new2(10, 10);
    for i in 0..5 {
        buffer.put(i);
    }

    buffer.flip();
    buffer.get_buf(&mut v, 0, 5);
    assert_eq!(v, vec![0,1,2,3,4]);

    // put_buf: 将源vec的内容，放入新创建/当前的HeapByteBuffer中
    let mut buffer = HeapByteBuffer::new2(5, 5);
    buffer.put_buf(&mut v, 0, 5);
    assert_eq!(buffer.position(), 5);
    assert_eq!(buffer.offset, 0);
    assert_eq!(buffer.hb, RefCell::new(v));

    // put_buffer: 将源HeapByteBuffer的内容，放入当前的HeapByteBuffer中

    // if not flip, the new buffer will not copy data
    buffer.flip();
    let mut buffer2 = HeapByteBuffer::new2(5, 5);
    buffer2.put_buffer(&mut buffer);
    println!("{:?}", buffer2);
}