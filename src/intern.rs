use std::collections::HashSet;
use std::sync::Mutex;

use alloc::sync::Arc;

use crate::{buf16, buf24, Str16, Str24};

/// [`Intern16`] is an interner storing and yielding [`Str16`] string types.
///
/// The [`intern`](Self::intern) method can be used to intern a string.
#[derive(Clone, Default)]
pub struct Intern16 {
    set: Arc<Mutex<HashSet<Str16>>>,
}

impl Intern16 {
    /// Construct a new empty interner.
    pub fn new() -> Self {
        Self::default()
    }

    /// Intern a given string.
    ///
    /// This will return the canonical inline representation for small strings,
    /// and will otherwise return an interned [`Str16`] shared with the interner.
    pub fn intern(&self, text: impl AsRef<str>) -> Str16 {
        self.intern_str(text.as_ref())
    }

    fn intern_str(&self, text: &str) -> Str16 {
        if text.len() <= buf16::INLINE_CAP {
            return Str16::from(text);
        }

        let mut set = self.set.lock().unwrap();
        if let Some(str) = set.get(text) {
            return str.clone();
        }

        let str = Str16::from(text);
        set.insert(str.clone());
        str
    }
}

/// [`Intern24`] is an interner storing and yielding [`Str24`] string types.
///
/// The [`intern`](Self::intern) method can be used to intern a string.
#[derive(Clone, Default)]
pub struct Intern24 {
    set: Arc<Mutex<HashSet<Str24>>>,
}

impl Intern24 {
    /// Construct a new empty interner.
    pub fn new() -> Self {
        Self::default()
    }

    /// Intern a given string.
    ///
    /// This will return the canonical inline representation for small strings,
    /// and will otherwise return an interned [`Str24`] shared with the interner.
    pub fn intern(&self, text: impl AsRef<str>) -> Str24 {
        self.intern_str(text.as_ref())
    }

    fn intern_str(&self, text: &str) -> Str24 {
        if text.len() <= buf24::INLINE_CAP {
            return Str24::from(text);
        }

        let mut set = self.set.lock().unwrap();
        if let Some(str) = set.get(text) {
            return str.clone();
        }

        let str = Str24::from(text);
        set.insert(str.clone());
        str
    }
}

#[cfg(test)]
mod tests {
    use core::ptr;

    use super::*;

    #[test]
    fn test_intern16() {
        let interner = Intern16::new();

        let interned = interner.intern("smol");
        assert!(!interned.is_heap_allocated());

        let heap1 = interner.intern("some text that is not so smol anymore");
        let heap2 = interner.intern("some text that is not so smol anymore");

        assert!(ptr::eq(heap1.as_str(), heap2.as_str()));
    }

    #[test]
    fn test_intern24() {
        let interner = Intern24::new();

        let interned = interner.intern("smol but more than 16");
        assert!(!interned.is_heap_allocated());

        let heap1 = interner.intern("some text that is not so smol anymore");
        let heap2 = interner.intern("some text that is not so smol anymore");

        assert!(ptr::eq(heap1.as_str(), heap2.as_str()));
    }
}
