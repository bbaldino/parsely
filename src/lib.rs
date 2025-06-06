#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

// TODO: these should be moved to a prelude file
pub use parsely_impl::anyhow::{Context, anyhow, bail};
pub use parsely_impl::error::{IntoParselyResult, IntoWritableParselyResult, ParselyResult};
pub use parsely_impl::impl_stateless_sync;
pub use parsely_impl::nsw_types::{from_bitslice::BitSliceUxExts, *};
pub use parsely_impl::{BigEndian, ByteOrder, LittleEndian, NetworkOrder};
pub use parsely_impl::{BitBuf, BitBufExts, BitBufMut, BitBufMutExts, Bits, BitsMut};
pub use parsely_impl::{BitCursor, BitRead, BitWrite};
pub use parsely_impl::{
    parsely_read::ParselyRead, parsely_write::ParselyWrite, parsely_write::StateSync,
};
pub use parsely_macro::{ParselyRead, ParselyWrite};

// These are more advanced usage: shouldn't be in prelude but should be accessible (needed to
// implement custom read/write trait types for the bitcursor type...maybe an alias would be better?)
