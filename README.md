# smol_buf

[![CI](https://github.com/Swatinem/smol_buf/workflows/CI/badge.svg)](https://github.com/Swatinem/smol_buf/actions?query=branch%3Amaster+workflow%3ACI)
[![Crates.io](https://img.shields.io/crates/v/smol_buf.svg)](https://crates.io/crates/smol_buf)
[![API reference](https://docs.rs/smol_buf/badge.svg)](https://docs.rs/smol_buf/)

The `smol_buf` crate offers the following types, each offering inline stack-allocated storage,
and falling back to heap-allocation otherwise.

| ty      | Deref Target | `size_of::<T>` | `size_of::<Option<T>>` | inline bytes | Clone  |
| ------- | ------------ | -------------- | ---------------------- | ------------ | ------ |
| `Str24` | `&str`       | 24             | 24                     | 23           | `O(1)` |
| `Str16` | `&str`       | 16             | 16                     | 15           | `O(1)` |
| `Buf24` | `&[u8]`      | 24             | 24                     | 23           | `O(1)` |
| `Buf16` | `&[u8]`      | 16             | 16                     | 15           | `O(1)` |

Unlike `String` and `Vec`, however, the types are immutable.
They are thus replacements for `Arc<str>` and `Arc<[u8]>` respectively.

## MSRV Policy

Minimal Supported Rust Version: latest stable.

Bumping MSRV is not considered a semver-breaking change.
