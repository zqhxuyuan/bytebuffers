use std::{marker, mem::{transmute, size_of}, slice, borrow::Cow};
use std::marker::PhantomData;

// #[repr(C)]
struct Slice<T> {
    offset: u32,
    len: u32,
    phantom: marker::PhantomData<T>,
}

impl <T> Slice<T> {
    pub fn new(offset: u32, len: u32) -> Self {
        Self {
            offset: offset,
            len: len,
            phantom: PhantomData
        }
    }
}

#[repr(C)]
struct Header {
    targets: Slice<u32>,
}

pub struct Data<'a> {
    bytes: Cow<'a, [u8]>,
}

impl<'a> Data<'a> {
    pub fn new<B: Into<Cow<'a, [u8]>>>(bytes: B) -> Data<'a> {
        Data { bytes: bytes.into() }
    }

    pub fn get_target(&self, idx: usize) -> u32 {
        self.slice(&self.header().targets)[idx]
    }

    fn bytes(&self, start: usize, len: usize) -> *const u8 {
        println!("start:{}, len:{}", start, len);
        self.bytes[start..start + len].as_ptr()
    }

    fn header(&self) -> &Header {
        unsafe { transmute(self.bytes(0, size_of::<Header>())) }
    }

    fn slice<T>(&self, s: &Slice<T>) -> &[T] {
        let size = size_of::<T>() * s.len as usize;
        let bytes = self.bytes(s.offset as usize, size);
        unsafe { slice::from_raw_parts(bytes as *const T, s.len as usize) }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::marker::PhantomData;

    #[test]
    fn test_work() {
        let slice: Slice<u32> = Slice::new(0, 3);

        // let header = Header {
        //     targets: slice
        // };

        let v = [0,1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,8,9];
        let data = Data::new(&v[..]);

        let slice1 = data.slice(&slice);
        println!("{:?}", slice1);

        let slice2 = data.slice(&slice);
        println!("{:?}", slice2);

        assert_eq!(slice1, slice2);
    }

    #[test]
    fn test_slice1() {
        use std::slice;

        // manifest a slice for a single element
        // let x = 42;
        // let ptr = &x as *const _;
        // let slice = unsafe { slice::from_raw_parts(ptr, 1) };
        // assert_eq!(slice[0], 42);

        let vec = vec![0,1,2,3,4];
        let ptr1 = &vec as *const _;
        let slice1 = unsafe { slice::from_raw_parts(ptr1, 1) };
        println!("{:?}", slice1);

        let mut a = A {
            vec: vec![1,2,3,4,5],
        };
        let ptr1 = &a as *const _;
        let slice1 = unsafe { slice::from_raw_parts(ptr1, 1) };
        println!("{:?}", slice1);

        a.vec[0] = 0;
        println!("{:?}", slice1);

    }

    #[derive(Debug)]
    struct A {
        vec: Vec<u8>
    }
}