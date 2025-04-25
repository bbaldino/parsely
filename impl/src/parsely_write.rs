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

pub trait ParselyWrite: StateSync + Sized {
    fn write<B: BitBufMut, Ctx, T: ByteOrder>(&self, buf: &mut B, ctx: Ctx) -> ParselyResult<()>;
}

macro_rules! impl_parsely_write_builtin {
    ($type:ty) => {
        impl ParselyWrite for $type {
            fn write<B: BitBufMut, Ctx, T: ByteOrder>(
                &self,
                buf: &mut B,
                _: Ctx,
            ) -> ParselyResult<()> {
                ::paste::paste! {
                    Ok(buf.[<put_ $type>](*self)?)
                }
            }
        }
    };
}

macro_rules! impl_parsely_write_builtin_bo {
    ($type:ty) => {
        impl ParselyWrite for $type {
            fn write<B: BitBufMut, Ctx, T: ByteOrder>(
                &self,
                buf: &mut B,
                _: Ctx,
            ) -> ParselyResult<()> {
                ::paste::paste! {
                    Ok(buf.[<put_ $type>]::<T>(*self)?)
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
