use alloc::sync::Arc;
use core::num::NonZeroU8;
use core::slice;
use core::{mem, ops};

#[repr(transparent)]
pub struct Buf24(Buf24Inner);

pub(crate) const INLINE_CAP: usize = 23;
const PADDING_BYTES: usize = 7;

#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
struct Buf24Inner {
    tag: NonZeroU8,
    _padding: [u8; PADDING_BYTES],
    ptr: u64,
    len: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Buf24Inline {
    tag_and_len: u8,
    buf: [u8; INLINE_CAP],
}

const _: () = {
    assert!(mem::size_of::<Buf24>() == 24);
    assert!(mem::size_of::<Option<Buf24>>() == 24);

    assert!(mem::size_of::<Buf24Inline>() == mem::size_of::<Buf24Inner>());
};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tag {
    Inline = 0b001,
    Arc = 0b010,
    Static = 0b100,
}
const TAG_SHIFT: usize = 3;
const TAG_MASK: u8 = 0b111;

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

        let tag_and_len = ((len as u8) << TAG_SHIFT) | Tag::Inline as u8;
        unsafe { mem::transmute(Buf24Inline { tag_and_len, buf }) }
    }

    #[inline]
    pub fn new_static(input: &'static [u8]) -> Self {
        let len = input.len();
        if len <= INLINE_CAP {
            Self::new_inline(input)
        } else {
            let ptr = input.as_ptr() as usize as u64;
            let tag = unsafe { NonZeroU8::new_unchecked(Tag::Static as u8) };
            Self(Buf24Inner {
                tag,
                _padding: [0; PADDING_BYTES],
                ptr,
                len: len as u64,
            })
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
        let tag = unsafe { NonZeroU8::new_unchecked(Tag::Arc as u8) };
        Self(Buf24Inner {
            tag,
            _padding: [0; PADDING_BYTES],
            ptr,
            len: len as u64,
        })
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
        unsafe { mem::transmute(self.0.tag.get() & TAG_MASK) }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        if self.tag() == Tag::Inline {
            (self.0.tag.get() >> TAG_SHIFT) as usize
        } else {
            self.0.len as usize
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
            let inline: &Buf24Inline = unsafe { mem::transmute(self) };
            return unsafe { inline.buf.get_unchecked(..len) };
        }

        let data = self.0.ptr as usize as *const u8;
        unsafe { slice::from_raw_parts(data, len) }
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
