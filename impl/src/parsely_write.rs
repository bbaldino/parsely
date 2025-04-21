use bits_io::prelude::*;

use crate::error::ParselyResult;

pub trait StateSync<SyncCtx>: Sized {
    // TODO: I think we should probably do a default impl of sync here that just returns Ok(())
    fn sync(&mut self, sync_ctx: SyncCtx) -> ParselyResult<()>;
}

// TODO: should this be changed to require StateSync? I think so? Pretty sure we assume it exists,
// so it'll move errors sooner
pub trait ParselyWrite<B, Ctx>: Sized {
    fn write<T: ByteOrder>(&self, buf: &mut B, ctx: Ctx) -> ParselyResult<()>;
}

macro_rules! impl_parsely_write_builtin {
    ($type:ty) => {
        impl<B: BitBufMut> ParselyWrite<B, ()> for $type {
            fn write<T: ByteOrder>(&self, buf: &mut B, _: ()) -> ParselyResult<()> {
                ::paste::paste! {
                    Ok(buf.[<put_ $type>](*self)?)
                }
            }
        }
    };
}

macro_rules! impl_parsely_write_builtin_bo {
    ($type:ty) => {
        impl<B: BitBufMut> ParselyWrite<B, ()> for $type {
            fn write<T: ByteOrder>(&self, buf: &mut B, _: ()) -> ParselyResult<()> {
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
        impl StateSync<()> for $type {
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
