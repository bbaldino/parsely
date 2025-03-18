use bit_cursor::nsw_types::*;
use bit_cursor::{bit_write::BitWrite, bit_write_exts::BitWriteExts, byte_order::ByteOrder};

use crate::error::ParselyResult;

pub trait ParselyWrite<B, Ctx>: Sized {
    fn write<T: ByteOrder>(&self, buf: &mut B, ctx: Ctx) -> ParselyResult<()>;
}

macro_rules! impl_parsely_write_builtin {
    ($type:ty) => {
        impl<B: BitWrite> ParselyWrite<B, ()> for $type {
            fn write<T: ByteOrder>(&self, buf: &mut B, _: ()) -> ParselyResult<()> {
                ::paste::paste! {
                    Ok(buf.[<write_ $type>](*self)?)
                }
            }
        }
    };
}

macro_rules! impl_parsely_write_builtin_bo {
    ($type:ty) => {
        impl<B: BitWrite> ParselyWrite<B, ()> for $type {
            fn write<T: ByteOrder>(&self, buf: &mut B, _: ()) -> ParselyResult<()> {
                ::paste::paste! {
                    Ok(buf.[<write_ $type>]::<T>(*self)?)
                }
            }
        }
    };
}

impl<B: BitWrite> ParselyWrite<B, ()> for bool {
    fn write<T: ByteOrder>(&self, buf: &mut B, _: ()) -> ParselyResult<()> {
        Ok(buf.write_bool(*self)?)
    }
}

impl_parsely_write_builtin!(u1);
impl_parsely_write_builtin!(u2);
impl_parsely_write_builtin!(u3);
impl_parsely_write_builtin!(u4);
impl_parsely_write_builtin!(u5);
impl_parsely_write_builtin!(u6);
impl_parsely_write_builtin!(u7);
impl_parsely_write_builtin!(u8);
impl_parsely_write_builtin_bo!(u9);
impl_parsely_write_builtin_bo!(u10);
impl_parsely_write_builtin_bo!(u11);
impl_parsely_write_builtin_bo!(u12);
impl_parsely_write_builtin_bo!(u13);
impl_parsely_write_builtin_bo!(u14);
impl_parsely_write_builtin_bo!(u15);
impl_parsely_write_builtin_bo!(u16);
impl_parsely_write_builtin_bo!(u17);
impl_parsely_write_builtin_bo!(u18);
impl_parsely_write_builtin_bo!(u19);
impl_parsely_write_builtin_bo!(u20);
impl_parsely_write_builtin_bo!(u21);
impl_parsely_write_builtin_bo!(u22);
impl_parsely_write_builtin_bo!(u23);
impl_parsely_write_builtin_bo!(u24);
impl_parsely_write_builtin_bo!(u25);
impl_parsely_write_builtin_bo!(u26);
impl_parsely_write_builtin_bo!(u27);
impl_parsely_write_builtin_bo!(u28);
impl_parsely_write_builtin_bo!(u29);
impl_parsely_write_builtin_bo!(u30);
impl_parsely_write_builtin_bo!(u31);
impl_parsely_write_builtin_bo!(u32);
