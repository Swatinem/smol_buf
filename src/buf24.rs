use alloc::sync::Arc;
use core::num::NonZeroU8;
use core::{mem, ops, ptr, slice};

/// A small-data optimized byte buffer.
///
/// See [`Str24`](crate::Str24) for all the properties.
#[repr(transparent)]
pub struct Buf24(Buf24Inner);

pub(crate) const INLINE_CAP: usize = 23;
const PADDING_BYTES: usize = 7;

#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
struct Buf24Inner {
    ptr: u64,
    len: u64,
    _padding: [u8; PADDING_BYTES],
    tag: NonZeroU8,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Buf24Inline {
    buf: [u8; INLINE_CAP],
    tag_and_len: u8,
}

const _: () = {
    assert!(mem::size_of::<Buf24>() == 24);
    assert!(mem::align_of::<Buf24>() == 8);
    assert!(mem::size_of::<Option<Buf24>>() == 24);

    assert!(mem::size_of::<Buf24Inline>() == mem::size_of::<Buf24Inner>());
};

const TAG_INLINE: u8 = 0b001 << 5;
const TAG_ARC: u8 = 0b010 << 5;
const TAG_STATIC: u8 = 0b100 << 5;
const TAG_MASK: u8 = !(0b111 << 5);

impl Buf24 {
    /// Constructs inline variant of `Buf24`.
    ///
    /// Panics if `input.len() > 23`.
    #[inline]
    pub const fn new_inline(input: &[u8]) -> Self {
        let len = input.len();
        assert!(len <= INLINE_CAP); // avoids checks in loop

        let mut buf = [0; INLINE_CAP];

        let mut i = 0;
        while i < len {
            buf[i] = input[i];
            i += 1
        }

        let tag_and_len = len as u8 | TAG_INLINE;
        unsafe { mem::transmute(Buf24Inline { buf, tag_and_len }) }
    }

    #[inline]
    pub fn new_static(input: &'static [u8]) -> Self {
        let len = input.len();
        if len <= INLINE_CAP {
            Self::new_inline(input)
        } else {
            let ptr = input.as_ptr() as usize as u64;
            let tag = unsafe { NonZeroU8::new_unchecked(TAG_STATIC) };
            Self(Buf24Inner {
                ptr,
                len: len as u64,
                _padding: [0; PADDING_BYTES],
                tag,
            })
        }
    }

    #[inline(always)]
    pub fn new(input: &[u8]) -> Self {
        let len = input.len();
        if len <= INLINE_CAP {
            Self::new_inline(input)
        } else {
            Self::new_arc(input)
        }
    }

    #[cold]
    fn new_arc(input: &[u8]) -> Self {
        let arc = Arc::from(input);
        Self::from_arc(input.len(), arc)
    }

    #[inline]
    pub(crate) fn from_arc(len: usize, arc: Arc<[u8]>) -> Self {
        let ptr = Arc::into_raw(arc) as *const u8 as usize as u64;
        let tag = unsafe { NonZeroU8::new_unchecked(TAG_ARC) };
        Self(Buf24Inner {
            ptr,
            len: len as u64,
            _padding: [0; PADDING_BYTES],
            tag,
        })
    }

    #[inline]
    pub(crate) fn as_arc(&self) -> Option<Arc<[u8]>> {
        if self.tag_byte() & TAG_ARC == 0 {
            return None;
        }

        let (ptr, len) = (self.0.ptr as usize as *const u8, self.0.len as usize);
        let arc_ptr = ptr::slice_from_raw_parts(ptr, len);
        Some(unsafe { Arc::from_raw(arc_ptr) })
    }

    #[inline(always)]
    fn tag_byte(&self) -> u8 {
        unsafe { mem::transmute::<&Buf24, &Buf24Inline>(self) }.tag_and_len
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        let tag_byte = self.tag_byte();
        if tag_byte & TAG_INLINE > 0 {
            (tag_byte & TAG_MASK) as usize
        } else {
            self.0.len as usize
        }
    }

    #[inline(always)]
    pub fn is_heap_allocated(&self) -> bool {
        self.tag_byte() & TAG_ARC > 0
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline(always)]
    pub fn as_bytes(&self) -> &[u8] {
        let tag_byte = self.tag_byte();
        let (ptr, len) = if tag_byte & TAG_INLINE > 0 {
            (
                self as *const _ as *const u8,
                (tag_byte & TAG_MASK) as usize,
            )
        } else {
            (self.0.ptr as usize as *const u8, self.0.len as usize)
        };
        unsafe { slice::from_raw_parts(ptr, len) }
    }
}

impl Drop for Buf24 {
    fn drop(&mut self) {
        drop(self.as_arc());
    }
}

impl Clone for Buf24 {
    fn clone(&self) -> Self {
        if let Some(arc) = self.as_arc() {
            unsafe { Arc::increment_strong_count(Arc::into_raw(arc)) };
        }

        Self(self.0)
    }
}

impl PartialEq for Buf24 {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 || self.as_bytes() == other.as_bytes()
    }
}

impl Eq for Buf24 {}

impl Default for Buf24 {
    #[inline(always)]
    fn default() -> Self {
        Self::new_inline(&[])
    }
}

impl ops::Deref for Buf24 {
    type Target = [u8];

    #[inline(always)]
    fn deref(&self) -> &[u8] {
        self.as_bytes()
    }
}

// TODO: copy over more methods
