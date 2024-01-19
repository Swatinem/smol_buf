use alloc::sync::Arc;
use core::num::NonZeroU64;
use core::slice;
use core::{mem, ops};

#[repr(transparent)]
pub struct Buf16(Buf16Inner);

pub(crate) const INLINE_CAP: usize = 15;

#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
struct Buf16Inner {
    ptr: u64,
    len_with_tag: NonZeroU64,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Buf16Inline {
    buf: [u8; INLINE_CAP],
    tag_and_len: u8,
}

const _: () = {
    assert!(mem::size_of::<Buf16>() == 16);
    assert!(mem::size_of::<Option<Buf16>>() == 16);

    assert!(mem::size_of::<Buf16Inline>() == mem::size_of::<Buf16Inner>());
};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tag {
    Inline = 0b001,
    Arc = 0b010,
    Static = 0b100,
}
const TAG_SHIFT: usize = 64 - 3;
const TAG_MASK: u64 = !(0b111 << TAG_SHIFT);
const TAG_INLINE_SHIFT: usize = 8 - 3;
const INLINE_LEN_SHIFT: usize = 64 - 8;

impl Buf16 {
    /// Constructs inline variant of `Buf16`.
    ///
    /// Panics if `input.len() > 15`.
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

        let tag_and_len = (len as u8) | ((Tag::Inline as u8) << TAG_INLINE_SHIFT);
        unsafe { mem::transmute(Buf16Inline { buf, tag_and_len }) }
    }

    #[inline]
    pub fn new_static(input: &'static [u8]) -> Self {
        let len = input.len();
        if len <= INLINE_CAP {
            Self::new_inline(input)
        } else {
            let ptr = input.as_ptr() as usize as u64;
            let len_with_tag = (len as u64 | ((Tag::Static as u64) << TAG_SHIFT)).to_le();
            let len_with_tag = unsafe { NonZeroU64::new_unchecked(len_with_tag) };
            Self(Buf16Inner { ptr, len_with_tag })
        }
    }

    pub fn new(input: &[u8]) -> Self {
        let len = input.len();
        if len <= INLINE_CAP {
            Self::new_inline(input)
        } else {
            let arc = Arc::from(input);
            Self::from_arc(arc)
        }
    }

    #[inline]
    pub(crate) fn from_arc(arc: Arc<[u8]>) -> Self {
        let len = arc.len();
        if len <= INLINE_CAP {
            return Self::new_inline(&arc);
        }

        let ptr = Arc::into_raw(arc) as *const u8 as usize as u64;
        let len_with_tag = (len as u64 | ((Tag::Arc as u64) << TAG_SHIFT)).to_le();
        let len_with_tag = unsafe { NonZeroU64::new_unchecked(len_with_tag) };
        Self(Buf16Inner { ptr, len_with_tag })
    }

    #[inline]
    pub(crate) fn as_arc(&self) -> Option<Arc<[u8]>> {
        if self.tag() != Tag::Arc {
            return None;
        }

        let arc_ptr = self.as_bytes() as *const [u8];

        Some(unsafe { Arc::from_raw(arc_ptr) })
    }

    #[inline(always)]
    fn tag(&self) -> Tag {
        let tag = (self.0.len_with_tag.get().to_le() >> TAG_SHIFT) as u8;
        unsafe { mem::transmute(tag) }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        let len_or_tag = self.0.len_with_tag.get().to_le();
        if self.tag() == Tag::Inline {
            ((len_or_tag & TAG_MASK) >> INLINE_LEN_SHIFT) as usize
        } else {
            (len_or_tag & TAG_MASK) as usize
        }
    }

    #[inline(always)]
    pub fn is_heap_allocated(&self) -> bool {
        self.tag() == Tag::Arc
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline(always)]
    pub fn as_bytes(&self) -> &[u8] {
        let len = self.len();

        if self.tag() == Tag::Inline {
            let inline: &Buf16Inline = unsafe { mem::transmute(self) };
            return unsafe { inline.buf.get_unchecked(..len) };
        }

        let data = self.0.ptr as usize as *const u8;
        unsafe { slice::from_raw_parts(data, len) }
    }
}

impl Drop for Buf16 {
    fn drop(&mut self) {
        drop(self.as_arc());
    }
}

impl Clone for Buf16 {
    fn clone(&self) -> Self {
        if let Some(arc) = self.as_arc() {
            unsafe { Arc::increment_strong_count(Arc::into_raw(arc)) };
        }

        Self(self.0)
    }
}

impl PartialEq for Buf16 {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 || self.as_bytes() == other.as_bytes()
    }
}

impl Eq for Buf16 {}

impl Default for Buf16 {
    #[inline(always)]
    fn default() -> Self {
        Self::new_inline(&[])
    }
}

impl ops::Deref for Buf16 {
    type Target = [u8];

    #[inline(always)]
    fn deref(&self) -> &[u8] {
        self.as_bytes()
    }
}

// TODO: copy over more methods
