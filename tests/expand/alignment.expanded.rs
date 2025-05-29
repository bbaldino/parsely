use parsely_rs::*;
#[parsely(alignment = 4)]
struct Foo {
    one: u8,
}
impl<B: BitBuf> ::parsely_rs::ParselyRead<B> for Foo {
    type Ctx = ();
    fn read<T: ::parsely_rs::ByteOrder>(
        buf: &mut B,
        (): (),
    ) -> ::parsely_rs::ParselyResult<Self> {
        let __bytes_read_before_Foo_read = buf.remaining_bytes();
        let one = u8::read::<T>(buf, ()).with_context(|| "Reading field 'one'")?;
        while (__bytes_read_before_Foo_read - buf.remaining_bytes()) % 4usize != 0 {
            buf.get_u8().context("consuming padding")?;
        }
        Ok(Self { one })
    }
}
impl<B: BitBufMut> ::parsely_rs::ParselyWrite<B> for Foo {
    type Ctx = ();
    fn write<T: ByteOrder>(&self, buf: &mut B, (): Self::Ctx) -> ParselyResult<()> {
        let __bytes_written_before_Foo_write = buf.remaining_mut_bytes();
        u8::write::<T>(&self.one, buf, ())
            .with_context(|| ::alloc::__export::must_use({
                let res = ::alloc::fmt::format(
                    format_args!("Writing field \'{0}\'", "one"),
                );
                res
            }))?;
        while (__bytes_written_before_Foo_write - buf.remaining_mut_bytes()) % 4usize
            != 0
        {
            buf.put_u8(0).context("adding padding")?;
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
