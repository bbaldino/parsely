use std::marker::PhantomData;

use bits_io::prelude::*;

use crate::error::ParselyResult;

/// A trait for syncing a field with any required context.  In order to prevent accidental misses
/// of this trait, it's required for all `ParselyWrite` implementors.  When generating the
/// `ParselyWrite` implementation, `sync` will be called on every field.
pub trait StateSync: Sized {
    type SyncCtx;

    fn sync(&mut self, _sync_ctx: Self::SyncCtx) -> ParselyResult<()> {
        Ok(())
    }
}

#[macro_export]
macro_rules! impl_stateless_sync {
    ($ty:ty) => {
        impl StateSync for $ty {
            type SyncCtx = ();
        }
    };
}

pub trait ParselyWrite2<B, Ctx>: StateSync + Sized {
    fn write<T: ByteOrder>(&self, buf: &mut B, ctx: Ctx) -> ParselyResult<()>;
}

pub trait ParselyWrite: StateSync + Sized {
    type Buf: BitBufMut;
    type Ctx;
    fn write<T: ByteOrder>(&self, buf: &mut Self::Buf, ctx: Self::Ctx) -> ParselyResult<()>;
}

pub struct WriteAdaptor<'a, T, B> {
    pub value: &'a T,
    _buf: PhantomData<B>,
}

impl<'a, T, B> WriteAdaptor<'a, T, B> {
    pub fn new(value: &'a T) -> Self {
        Self {
            value,
            _buf: PhantomData,
        }
    }
}

// The `WriteAdaptor` type enables two things:
// 1. We can now use `ParselyWrite` as a trait bounds, which is useful to allow type inference for
//    scenarios like the `map` functionality on the write path, where we don't have a concrete type
//    to infer the result of the map function to, other than something that can be written.
// 2. Allows us to avoid having to define a concrete associated type in the impls of ParselyWrite:
//    since we can use the generics on WriteAdaptor to set as the associated typed in the
//    ParselyWrite impl.
//
// One downside of it, though, is that now a `WriteAdaptor` instance needs to be created to do
// writes.  This isn't a performance concern, but is a bit awkward from an API perspective

pub trait ParselyWritableExt {
    fn write_with<'a, B, BO>(
        &'a self,
        buf: &mut B,
        ctx: <WriteAdaptor<'a, Self, B> as ParselyWrite>::Ctx,
    ) -> ParselyResult<()>
    where
        Self: Sized,
        B: BitBufMut,
        BO: ByteOrder,
        WriteAdaptor<'a, Self, B>: ParselyWrite<Buf = B>;
}

impl<T> ParselyWritableExt for T {
    fn write_with<'a, B, BO>(
        &'a self,
        buf: &mut B,
        ctx: <WriteAdaptor<'a, Self, B> as ParselyWrite>::Ctx,
    ) -> ParselyResult<()>
    where
        B: BitBufMut,
        BO: ByteOrder,
        WriteAdaptor<'a, Self, B>: ParselyWrite<Buf = B>,
    {
        WriteAdaptor::<_, B>::new(self).write::<BO>(buf, ctx)
    }
}

macro_rules! impl_parsely_write_builtin {
    ($type:ty) => {
        impl<B: BitBufMut> ParselyWrite2<B, ()> for $type {
            fn write<T: ByteOrder>(&self, buf: &mut B, _: ()) -> ParselyResult<()> {
                ::paste::paste! {
                    Ok(buf.[<put_ $type>](*self)?)
                }
            }
        }
        impl<'a, B: BitBufMut> ParselyWrite for WriteAdaptor<'a, $type, B> {
            type Buf = B;
            type Ctx = ();
            fn write<T: ByteOrder>(&self, buf: &mut Self::Buf, _: Self::Ctx) -> ParselyResult<()> {
                ::paste::paste! {
                    Ok(buf.[<put_ $type>](*self.value)?)
                }
            }
        }
    };
}

macro_rules! impl_parsely_write_builtin_bo {
    ($type:ty) => {
        impl<B: BitBufMut> ParselyWrite2<B, ()> for $type {
            fn write<T: ByteOrder>(&self, buf: &mut B, _: ()) -> ParselyResult<()> {
                ::paste::paste! {
                    Ok(buf.[<put_ $type>]::<T>(*self)?)
                }
            }
        }
        impl<'a, B: BitBufMut> ParselyWrite for WriteAdaptor<'a, $type, B> {
            type Buf = B;
            type Ctx = ();
            fn write<T: ByteOrder>(&self, buf: &mut Self::Buf, _: Self::Ctx) -> ParselyResult<()> {
                ::paste::paste! {
                    Ok(buf.[<put_ $type>]::<T>(*self.value)?)
                }
            }
        }
    };
}

/// Accepts an array of types and calls the given macro on each of them
/// Examples:
/// for_all!({u1, u2, u3}, @some_macro);
macro_rules! for_all {
    ({$type:ty}, @$macro:ident) => {
        $macro!($type);
    };
    ({$type:ty, $($tail:ty),*}, @$macro:ident) => {
        $macro!($type);
        for_all!({$($tail),*}, @$macro);
    };
}

macro_rules! impl_state_sync_builtin {
    ($type:ty) => {
        impl StateSync for $type {
            type SyncCtx = ();
            fn sync(&mut self, _sync_ctx: ()) -> ParselyResult<()> {
                Ok(())
            }
        }
        impl<'a, B: BitBufMut> StateSync for WriteAdaptor<'a, $type, B> {
            type SyncCtx = ();
            fn sync(&mut self, _sync_ctx: ()) -> ParselyResult<()> {
                Ok(())
            }
        }
    };
}

for_all!({bool, u1, u2, u3, u4, u5, u6, u7, u8}, @impl_parsely_write_builtin);
for_all!({u9, u10, u11, u12, u13, u14, u15, u16}, @impl_parsely_write_builtin_bo);
for_all!({u17, u18, u19, u20, u21, u22, u23, u24}, @impl_parsely_write_builtin_bo);
for_all!({u25, u26, u27, u28, u29, u30, u31, u32}, @impl_parsely_write_builtin_bo);

for_all!({bool, u1, u2, u3, u4, u5, u6, u7, u8}, @impl_state_sync_builtin);
for_all!({u9, u10, u11, u12, u13, u14, u15, u16}, @impl_state_sync_builtin);
for_all!({u17, u18, u19, u20, u21, u22, u23, u24}, @impl_state_sync_builtin);
for_all!({u25, u26, u27, u28, u29, u30, u31, u32}, @impl_state_sync_builtin);
for_all!({String}, @impl_state_sync_builtin);
