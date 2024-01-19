use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::sync::Arc;
use core::borrow::Borrow;
use core::cmp::Ordering;
use core::convert::Infallible;
use core::ops::Deref;
use core::str::{from_utf8_unchecked, FromStr};
use core::{fmt, hash, iter, mem};

use crate::buf16::{Buf16, INLINE_CAP};

/// A `Str16` is a string type that has the following properties:
///
/// * `size_of::<Str16>() == 16` (therefor `== size_of::<Arc<str>>()` on 64 bit platforms)
/// * `size_of::<Option<Str16>>() == size_of::<Str16>()`
/// * `Clone` is `O(1)`
/// * Strings are stack-allocated if they are up to 15 bytes long
/// * If a string does not satisfy the aforementioned conditions, it is heap-allocated
/// * Additionally, a `Str16` can be explicitly created from a `&'static str` without allocation
///
/// Unlike `String`, however, `Str16` is immutable.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Str16(Buf16);

impl Str16 {
    /// Constructs inline variant of `Str16`.
    ///
    /// Panics if `text.len() > 23`.
    #[inline]
    pub const fn new_inline(text: &str) -> Str16 {
        Self(Buf16::new_inline(text.as_bytes()))
    }

    /// Constructs a `Str16` from a statically allocated string.
    ///
    /// This never allocates.
    #[inline]
    pub fn new_static(text: &'static str) -> Str16 {
        Self(Buf16::new_static(text.as_bytes()))
    }

    pub fn new<T>(text: T) -> Str16
    where
        T: AsRef<str>,
    {
        Str16(Buf16::new(text.as_ref().as_bytes()))
    }

    #[inline(always)]
    pub fn as_str(&self) -> &str {
        unsafe { from_utf8_unchecked(self.0.as_bytes()) }
    }

    #[allow(clippy::inherent_to_string_shadow_display)]
    #[inline(always)]
    pub fn to_string(&self) -> String {
        use alloc::borrow::ToOwned;

        self.as_str().to_owned()
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline(always)]
    pub fn is_heap_allocated(&self) -> bool {
        self.0.is_heap_allocated()
    }

    fn from_arc(arc: Arc<str>) -> Self {
        let len = arc.len();
        if len <= INLINE_CAP {
            Self::new_inline(&arc)
        } else {
            Self(Buf16::from_arc(len, unsafe { mem::transmute(arc) }))
        }
    }

    fn from_char_iter<I: iter::Iterator<Item = char>>(mut iter: I) -> Str16 {
        let (min_size, _) = iter.size_hint();
        if min_size > INLINE_CAP {
            let heap: String = iter.collect();
            let arc: Arc<str> = Arc::from(heap.as_str());
            return Self::from_arc(arc);
        }
        let mut len = 0;
        let mut buf = [0u8; INLINE_CAP];
        while let Some(ch) = iter.next() {
            let size = ch.len_utf8();
            if size + len > INLINE_CAP {
                let (min_remaining, _) = iter.size_hint();
                let mut heap = String::with_capacity(size + len + min_remaining);
                heap.push_str(core::str::from_utf8(&buf[..len]).unwrap());
                heap.push(ch);
                heap.extend(iter);
                return Self::new(&heap);
            }
            ch.encode_utf8(&mut buf[len..]);
            len += size;
        }
        Str16(Buf16::new_inline(&buf[..len]))
    }

    fn from_str_iter<T>(mut iter: impl Iterator<Item = T>) -> Str16
    where
        T: AsRef<str>,
        String: iter::Extend<T>,
    {
        let mut len = 0;
        let mut buf = [0u8; INLINE_CAP];
        while let Some(slice) = iter.next() {
            let slice = slice.as_ref();
            let size = slice.len();
            if size + len > INLINE_CAP {
                let mut heap = String::with_capacity(size + len);
                heap.push_str(core::str::from_utf8(&buf[..len]).unwrap());
                heap.push_str(slice);
                heap.extend(iter);
                return Str16::new(&heap);
            }
            buf[len..][..size].copy_from_slice(slice.as_bytes());
            len += size;
        }
        Str16(Buf16::new_inline(&buf[..len]))
    }
}

impl Deref for Str16 {
    type Target = str;

    #[inline(always)]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl PartialEq<str> for Str16 {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<Str16> for str {
    fn eq(&self, other: &Str16) -> bool {
        other == self
    }
}

impl<'a> PartialEq<&'a str> for Str16 {
    fn eq(&self, other: &&'a str) -> bool {
        self == *other
    }
}

impl<'a> PartialEq<Str16> for &'a str {
    fn eq(&self, other: &Str16) -> bool {
        *self == other
    }
}

impl PartialEq<String> for Str16 {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<Str16> for String {
    fn eq(&self, other: &Str16) -> bool {
        other == self
    }
}

impl<'a> PartialEq<&'a String> for Str16 {
    fn eq(&self, other: &&'a String) -> bool {
        self == *other
    }
}

impl<'a> PartialEq<Str16> for &'a String {
    fn eq(&self, other: &Str16) -> bool {
        *self == other
    }
}

impl Ord for Str16 {
    fn cmp(&self, other: &Str16) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl PartialOrd for Str16 {
    fn partial_cmp(&self, other: &Str16) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl hash::Hash for Str16 {
    fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
        self.as_str().hash(hasher);
    }
}

impl fmt::Debug for Str16 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl fmt::Display for Str16 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

impl iter::FromIterator<char> for Str16 {
    fn from_iter<I: iter::IntoIterator<Item = char>>(iter: I) -> Str16 {
        Self::from_char_iter(iter.into_iter())
    }
}

impl iter::FromIterator<String> for Str16 {
    fn from_iter<I: iter::IntoIterator<Item = String>>(iter: I) -> Str16 {
        Self::from_str_iter(iter.into_iter())
    }
}

impl<'a> iter::FromIterator<&'a String> for Str16 {
    fn from_iter<I: iter::IntoIterator<Item = &'a String>>(iter: I) -> Str16 {
        Self::from_str_iter(iter.into_iter().map(|x| x.as_str()))
    }
}

impl<'a> iter::FromIterator<&'a str> for Str16 {
    fn from_iter<I: iter::IntoIterator<Item = &'a str>>(iter: I) -> Str16 {
        Self::from_str_iter(iter.into_iter())
    }
}

impl AsRef<str> for Str16 {
    #[inline(always)]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<&str> for Str16 {
    #[inline]
    fn from(s: &str) -> Str16 {
        Str16::new(s)
    }
}

impl From<&mut str> for Str16 {
    #[inline]
    fn from(s: &mut str) -> Str16 {
        Str16::new(s)
    }
}

impl From<&String> for Str16 {
    #[inline]
    fn from(s: &String) -> Str16 {
        Str16::new(s)
    }
}

impl From<String> for Str16 {
    #[inline(always)]
    fn from(text: String) -> Self {
        Self::new(text)
    }
}

impl From<Box<str>> for Str16 {
    #[inline]
    fn from(s: Box<str>) -> Str16 {
        Str16::new(s)
    }
}

impl From<Arc<str>> for Str16 {
    #[inline]
    fn from(s: Arc<str>) -> Str16 {
        Self::from_arc(s)
    }
}

impl<'a> From<Cow<'a, str>> for Str16 {
    #[inline]
    fn from(s: Cow<'a, str>) -> Str16 {
        Str16::new(s)
    }
}

impl From<Str16> for Arc<str> {
    #[inline(always)]
    fn from(text: Str16) -> Self {
        if let Some(arc) = text.0.as_arc() {
            mem::forget(text);
            return unsafe { mem::transmute(arc) };
        }
        Arc::from(text.as_str())
    }
}

impl From<Str16> for String {
    #[inline(always)]
    fn from(text: Str16) -> Self {
        text.as_str().into()
    }
}

impl Borrow<str> for Str16 {
    #[inline(always)]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl FromStr for Str16 {
    type Err = Infallible;

    #[inline]
    fn from_str(s: &str) -> Result<Str16, Self::Err> {
        Ok(Str16::from(s))
    }
}

#[cfg(feature = "arbitrary")]
impl<'a> arbitrary::Arbitrary<'a> for Str16 {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> Result<Self, arbitrary::Error> {
        let s = <&str>::arbitrary(u)?;
        Ok(Str16::new(s))
    }
}

#[cfg(feature = "serde")]
mod serde {
    use alloc::string::String;
    use alloc::vec::Vec;
    use core::fmt;

    use serde::de::{Deserializer, Error, Unexpected, Visitor};

    use crate::Str16;

    // https://github.com/serde-rs/serde/blob/629802f2abfd1a54a6072992888fea7ca5bc209f/serde/src/private/de.rs#L56-L125
    fn str16<'de: 'a, 'a, D>(deserializer: D) -> Result<Str16, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Str16Visitor;

        impl<'a> Visitor<'a> for Str16Visitor {
            type Value = Str16;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Str16::from(v))
            }

            fn visit_borrowed_str<E>(self, v: &'a str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Str16::from(v))
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Str16::from(v))
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: Error,
            {
                match core::str::from_utf8(v) {
                    Ok(s) => Ok(Str16::from(s)),
                    Err(_) => Err(Error::invalid_value(Unexpected::Bytes(v), &self)),
                }
            }

            fn visit_borrowed_bytes<E>(self, v: &'a [u8]) -> Result<Self::Value, E>
            where
                E: Error,
            {
                match core::str::from_utf8(v) {
                    Ok(s) => Ok(Str16::from(s)),
                    Err(_) => Err(Error::invalid_value(Unexpected::Bytes(v), &self)),
                }
            }

            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
            where
                E: Error,
            {
                match String::from_utf8(v) {
                    Ok(s) => Ok(Str16::from(s)),
                    Err(e) => Err(Error::invalid_value(
                        Unexpected::Bytes(&e.into_bytes()),
                        &self,
                    )),
                }
            }
        }

        deserializer.deserialize_str(Str16Visitor)
    }

    impl serde::Serialize for Str16 {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.as_str().serialize(serializer)
        }
    }

    impl<'de> serde::Deserialize<'de> for Str16 {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            str16(deserializer)
        }
    }
}
