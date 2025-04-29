use bits_io::prelude::*;

use crate::error::ParselyResult;

pub trait ParselyRead<B>: Sized {
    type Ctx;
    fn read<T: ByteOrder>(buf: &mut B, ctx: Self::Ctx) -> ParselyResult<Self>;
}

macro_rules! impl_parsely_read_builtin {
    ($type:ty) => {
        impl<B: BitBuf> ParselyRead<B> for $type {
            type Ctx = ();
            fn read<T: ByteOrder>(buf: &mut B, _: Self::Ctx) -> ParselyResult<Self> {
                ::paste::paste! {
                    Ok(buf.[<get_ $type>]()?)
                }
            }
        }
    };
}

macro_rules! impl_parsely_read_builtin_bo {
    ($type:ty) => {
        impl<B: BitBuf> ParselyRead<B> for $type {
            type Ctx = ();
            fn read<T: ByteOrder>(buf: &mut B, _: Self::Ctx) -> ParselyResult<Self> {
                ::paste::paste! {
                    Ok(buf.[<get_ $type>]::<T>()?)
                }
            }
        }
    };
}

impl<B: BitBuf> ParselyRead<B> for bool {
    type Ctx = ();
    fn read<T: ByteOrder>(buf: &mut B, _ctx: Self::Ctx) -> ParselyResult<Self> {
        Ok(buf.get_bool()?)
    }
}

impl_parsely_read_builtin!(u1);
impl_parsely_read_builtin!(u2);
impl_parsely_read_builtin!(u3);
impl_parsely_read_builtin!(u4);
impl_parsely_read_builtin!(u5);
impl_parsely_read_builtin!(u6);
impl_parsely_read_builtin!(u7);
impl_parsely_read_builtin!(u8);
impl_parsely_read_builtin_bo!(u9);
impl_parsely_read_builtin_bo!(u10);
impl_parsely_read_builtin_bo!(u11);
impl_parsely_read_builtin_bo!(u12);
impl_parsely_read_builtin_bo!(u13);
impl_parsely_read_builtin_bo!(u14);
impl_parsely_read_builtin_bo!(u15);
impl_parsely_read_builtin_bo!(u16);
impl_parsely_read_builtin_bo!(u17);
impl_parsely_read_builtin_bo!(u18);
impl_parsely_read_builtin_bo!(u19);
impl_parsely_read_builtin_bo!(u20);
impl_parsely_read_builtin_bo!(u21);
impl_parsely_read_builtin_bo!(u22);
impl_parsely_read_builtin_bo!(u23);
impl_parsely_read_builtin_bo!(u24);
impl_parsely_read_builtin_bo!(u25);
impl_parsely_read_builtin_bo!(u26);
impl_parsely_read_builtin_bo!(u27);
impl_parsely_read_builtin_bo!(u28);
impl_parsely_read_builtin_bo!(u29);
impl_parsely_read_builtin_bo!(u30);
impl_parsely_read_builtin_bo!(u31);
impl_parsely_read_builtin_bo!(u32);
