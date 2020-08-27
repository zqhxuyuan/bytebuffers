/// Copyright 2018-2019 Mozilla Foundation
/// https://github.com/mozilla/ffi-support/blob/main/src/lib.rs
///
/// ByteBuffer is a struct that represents an array of bytes to be sent over the FFI boundaries.
/// There are several cases when you might want to use this, but the primary one for us
/// is for returning protobuf-encoded data to Swift and Java. The type is currently rather
/// limited (implementing almost no functionality), however in the future it may be
/// more expanded.
///
/// ## Caveats
///
/// Note that the order of the fields is `len` (an i64) then `data` (a `*mut u8`), getting
/// this wrong on the other side of the FFI will cause memory corruption and crashes.
/// `i64` is used for the length instead of `u64` and `usize` because JNA has interop
/// issues with both these types.
///
/// ### `Drop` is not implemented
///
/// ByteBuffer does not implement Drop. This is intentional. Memory passed into it will
/// be leaked if it is not explicitly destroyed by calling [`ByteBuffer::destroy`], or
/// [`ByteBuffer::destroy_into_vec`]. This is for two reasons:
///
/// 1. In the future, we may allow it to be used for data that is not managed by
///    the Rust allocator\*, and `ByteBuffer` assuming it's okay to automatically
///    deallocate this data with the Rust allocator.
///
/// 2. Automatically running destructors in unsafe code is a
///    [frequent footgun](https://without.boats/blog/two-memory-bugs-from-ringbahn/)
///    (among many similar issues across many crates).
///
/// Note that calling `destroy` manually is often not needed, as usually you should
/// be passing these to the function defined by [`define_bytebuffer_destructor!`] from
/// the other side of the FFI.
///
/// Because this type is essentially *only* useful in unsafe or FFI code (and because
/// the most common usage pattern does not require manually managing the memory), it
/// does not implement `Drop`.
///
/// \* Note: in the case of multiple Rust shared libraries loaded at the same time,
/// there may be multiple instances of "the Rust allocator" (one per shared library),
/// in which case we're referring to whichever instance is active for the code using
/// the `ByteBuffer`. Note that this doesn't occur on all platforms or build
/// configurations, but treating allocators in different shared libraries as fully
/// independent is always safe.
///
/// ## Layout/fields
///
/// This struct's field are not `pub` (mostly so that we can soundly implement `Send`, but also so
/// that we can verify rust users are constructing them appropriately), the fields, their types, and
/// their order are *very much* a part of the public API of this type. Consumers on the other side
/// of the FFI will need to know its layout.
///
/// If this were a C struct, it would look like
///
/// ```c,no_run
/// struct ByteBuffer {
///     // Note: This should never be negative, but values above
///     // INT64_MAX / i64::MAX are not allowed.
///     int64_t len;
///     // Note: nullable!
///     uint8_t *data;
/// };
/// ```
///
/// In rust, there are two fields, in this order: `len: i64`, and `data: *mut u8`.
///
/// For clarity, the fact that the data pointer is nullable means that `Option<ByteBuffer>` is not
/// the same size as ByteBuffer, and additionally is not FFI-safe (the latter point is not
/// currently guaranteed anyway as of the time of writing this comment).
///
/// ### Description of fields
///
/// `data` is a pointer to an array of `len` bytes. Note that data can be a null pointer and therefore
/// should be checked.
///
/// The bytes array is allocated on the heap and must be freed on it as well. Critically, if there
/// are multiple rust shared libraries using being used in the same application, it *must be freed
/// on the same heap that allocated it*, or you will corrupt both heaps.
///
/// Typically, this object is managed on the other side of the FFI (on the "FFI consumer"), which
/// means you must expose a function to release the resources of `data` which can be done easily
/// using the [`define_bytebuffer_destructor!`] macro provided by this crate.
#[repr(C)]
pub struct ByteBuffer {
    len: i64,
    data: *mut u8,
}

impl From<Vec<u8>> for ByteBuffer {
    #[inline]
    fn from(bytes: Vec<u8>) -> Self {
        Self::from_vec(bytes)
    }
}

impl ByteBuffer {
    /// Creates a `ByteBuffer` of the requested size, zero-filled.
    ///
    /// The contents of the vector will not be dropped. Instead, `destroy` must
    /// be called later to reclaim this memory or it will be leaked.
    ///
    /// ## Caveats
    ///
    /// This will panic if the buffer length (`usize`) cannot fit into a `i64`.
    #[inline]
    pub fn new_with_size(size: usize) -> Self {
        // Note: `Vec` requires this internally on 64 bit platforms (and has a
        // stricter requirement on 32 bit ones), so this is just to be explicit.
        assert!(size < i64::MAX as usize);
        let mut buf = vec![];
        buf.reserve_exact(size);
        buf.resize(size, 0);
        ByteBuffer::from_vec(buf)
    }

    /// Creates a `ByteBuffer` instance from a `Vec` instance.
    ///
    /// The contents of the vector will not be dropped. Instead, `destroy` must
    /// be called later to reclaim this memory or it will be leaked.
    ///
    /// ## Caveats
    ///
    /// This will panic if the buffer length (`usize`) cannot fit into a `i64`.
    #[inline]
    pub fn from_vec(bytes: Vec<u8>) -> Self {
        use std::convert::TryFrom;
        let mut buf = bytes.into_boxed_slice();
        let data = buf.as_mut_ptr();
        let len = i64::try_from(buf.len()).expect("buffer length cannot fit into a i64.");
        std::mem::forget(buf);
        Self { data, len }
    }

    /// View the data inside this `ByteBuffer` as a `&[u8]`.
    // TODO: Is it worth implementing `Deref`? Patches welcome if you need this.
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        if self.data.is_null() {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(self.data, self.len()) }
        }
    }

    #[inline]
    fn len(&self) -> usize {
        use std::convert::TryInto;
        self.len
            .try_into()
            .expect("ByteBuffer length negative or overflowed")
    }

    /// View the data inside this `ByteBuffer` as a `&mut [u8]`.
    // TODO: Is it worth implementing `DerefMut`? Patches welcome if you need this.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        if self.data.is_null() {
            &mut []
        } else {
            unsafe { std::slice::from_raw_parts_mut(self.data, self.len()) }
        }
    }

    /// Deprecated alias for [`ByteBuffer::destroy_into_vec`].
    #[inline]
    #[deprecated = "Name is confusing, please use `destroy_into_vec` instead"]
    pub fn into_vec(self) -> Vec<u8> {
        self.destroy_into_vec()
    }

    /// Convert this `ByteBuffer` into a Vec<u8>, taking ownership of the
    /// underlying memory, which will be freed using the rust allocator once the
    /// `Vec<u8>`'s lifetime is done.
    ///
    /// If this is undesirable, you can do `bb.as_slice().to_vec()` to get a
    /// `Vec<u8>` containing a copy of this `ByteBuffer`'s underlying data.
    ///
    /// ## Caveats
    ///
    /// This is safe so long as the buffer is empty, or the data was allocated
    /// by Rust code, e.g. this is a ByteBuffer created by
    /// `ByteBuffer::from_vec` or `Default::default`.
    ///
    /// If the ByteBuffer were allocated by something other than the
    /// current/local Rust `global_allocator`, then calling `destroy` is
    /// fundamentally broken.
    ///
    /// For example, if it were allocated externally by some other language's
    /// runtime, or if it were allocated by the global allocator of some other
    /// Rust shared object in the same application, the behavior is undefined
    /// (and likely to cause problems).
    ///
    /// Note that this currently can only happen if the `ByteBuffer` is passed
    /// to you via an `extern "C"` function that you expose, as opposed to being
    /// created locally.
    #[inline]
    pub fn destroy_into_vec(self) -> Vec<u8> {
        if self.data.is_null() {
            vec![]
        } else {
            let len = self.len();
            // Safety: This is correct because we convert to a Box<[u8]> first,
            // which is a design constraint of RawVec.
            unsafe { Vec::from_raw_parts(self.data, len, len) }
        }
    }

    /// Reclaim memory stored in this ByteBuffer.
    ///
    /// You typically should not call this manually, and instead expose a
    /// function that does so via [`define_bytebuffer_destructor!`].
    ///
    /// ## Caveats
    ///
    /// This is safe so long as the buffer is empty, or the data was allocated
    /// by Rust code, e.g. this is a ByteBuffer created by
    /// `ByteBuffer::from_vec` or `Default::default`.
    ///
    /// If the ByteBuffer were allocated by something other than the
    /// current/local Rust `global_allocator`, then calling `destroy` is
    /// fundamentally broken.
    ///
    /// For example, if it were allocated externally by some other language's
    /// runtime, or if it were allocated by the global allocator of some other
    /// Rust shared object in the same application, the behavior is undefined
    /// (and likely to cause problems).
    ///
    /// Note that this currently can only happen if the `ByteBuffer` is passed
    /// to you via an `extern "C"` function that you expose, as opposed to being
    /// created locally.
    #[inline]
    pub fn destroy(self) {
        // Note: the drop is just for clarity, of course.
        drop(self.destroy_into_vec())
    }
}

impl Default for ByteBuffer {
    #[inline]
    fn default() -> Self {
        Self {
            len: 0 as i64,
            data: std::ptr::null_mut(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_bb_access() {
        let mut bb = ByteBuffer::from(vec![1u8, 2, 3]);
        assert_eq!(bb.as_slice(), &[1u8, 2, 3]);
        assert_eq!(bb.as_mut_slice(), &mut [1u8, 2, 3]);
        bb.as_mut_slice()[2] = 4;

        // Use into_vec to cover both into_vec and destroy_into_vec.
        #[allow(deprecated)]
            {
                assert_eq!(bb.into_vec(), &[1u8, 2, 4]);
            }
    }

    #[test]
    fn test_bb_empty() {
        let mut bb = ByteBuffer::default();
        assert_eq!(bb.as_slice(), &[]);
        assert_eq!(bb.as_mut_slice(), &[]);
        assert_eq!(bb.destroy_into_vec(), &[]);
    }

    #[test]
    fn test_bb_new() {
        let bb = ByteBuffer::new_with_size(5);
        assert_eq!(bb.as_slice(), &[0u8, 0, 0, 0, 0]);
        bb.destroy();

        let bb = ByteBuffer::new_with_size(0);
        assert_eq!(bb.as_slice(), &[]);
        assert!(!bb.data.is_null());
        bb.destroy();

        let bb = ByteBuffer::from_vec(vec![]);
        assert_eq!(bb.as_slice(), &[]);
        assert!(!bb.data.is_null());
        bb.destroy();
    }
}