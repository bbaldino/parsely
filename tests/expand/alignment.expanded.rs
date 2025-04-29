use parsely_rs::*;
#[parsely(alignment = 4)]
struct Foo {
    one: u8,
}
impl ::parsely_rs::ParselyRead for Foo {
    type Ctx = ();
    fn read<B: BitBuf, T: ::parsely_rs::ByteOrder>(
        buf: &mut B,
        _ctx: (),
    ) -> ::parsely_rs::ParselyResult<Self> {
        let __bytes_remaining_start = buf.remaining_bytes();
        let one = u8::read::<_, T>(buf, ()).with_context(|| "Reading field 'one'")?;
        let __bytes_remaining_end = buf.remaining_bytes();
        let mut __amount_read = __bytes_remaining_start - __bytes_remaining_end;
        while __amount_read % 4usize != 0 {
            buf.get_u8().context("padding")?;
            __amount_read += 1;
        }
        Ok(Self { one })
    }
}
impl<B: BitBufMut> ::parsely_rs::ParselyWrite<B> for Foo {
    type Ctx = ();
    fn write<T: ByteOrder>(&self, buf: &mut B, ctx: Self::Ctx) -> ParselyResult<()> {
        let __bytes_remaining_start = buf.remaining_mut_bytes();
        u8::write::<T>(&self.one, buf, ())
            .with_context(|| ::alloc::__export::must_use({
                let res = ::alloc::fmt::format(
                    format_args!("Writing field \'{0}\'", "one"),
                );
                res
            }))?;
        let __bytes_remaining_end = buf.remaining_mut_bytes();
        let mut __amount_written = __bytes_remaining_start - __bytes_remaining_end;
        while __amount_written % 4usize != 0 {
            let _ = buf.put_u8(0).context("padding")?;
            __amount_written += 1;
        }
        Ok(())
    }
}
impl StateSync for Foo {
    type SyncCtx = ();
    fn sync(&mut self, (): ()) -> ParselyResult<()> {
        self.one
            .sync(())
            .with_context(|| ::alloc::__export::must_use({
                let res = ::alloc::fmt::format(
                    format_args!("Syncing field \'{0}\'", "one"),
                );
                res
            }))?;
        Ok(())
    }
}
