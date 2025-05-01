use parsely_rs::*;
struct Foo {
    #[parsely_read(map = "|v: u8| { v.to_string() }")]
    #[parsely_write(map = "|v: &str| { v.parse::<u8>() }")]
    value: String,
}
impl<B: BitBuf> ::parsely_rs::ParselyRead<B> for Foo {
    type Ctx = ();
    fn read<T: ::parsely_rs::ByteOrder>(
        buf: &mut B,
        _ctx: (),
    ) -> ::parsely_rs::ParselyResult<Self> {
        let value = {
            let original_value = ::parsely_rs::ParselyRead::read::<T>(buf, ())
                .with_context(|| ::alloc::__export::must_use({
                    let res = ::alloc::fmt::format(
                        format_args!("Reading raw value for field \'{0}\'", "value"),
                    );
                    res
                }))?;
            (|v: u8| { v.to_string() })(original_value)
                .into_parsely_result()
                .with_context(|| ::alloc::__export::must_use({
                    let res = ::alloc::fmt::format(
                        format_args!("Mapping raw value for field \'{0}\'", "value"),
                    );
                    res
                }))
        }
            .with_context(|| "Reading field 'value'")?;
        Ok(Self { value })
    }
}
impl<B: BitBufMut> ::parsely_rs::ParselyWrite<B> for Foo {
    type Ctx = ();
    fn write<T: ByteOrder>(&self, buf: &mut B, ctx: Self::Ctx) -> ParselyResult<()> {
        {
            let mapped_value = (|v: &str| { v.parse::<u8>() })(&self.value);
            let result = <_ as IntoWritableParselyResult<
                _,
                B,
            >>::into_writable_parsely_result(mapped_value)
                .with_context(|| ::alloc::__export::must_use({
                    let res = ::alloc::fmt::format(
                        format_args!("Mapping raw value for field \'{0}\'", "value"),
                    );
                    res
                }))?;
            ::parsely_rs::ParselyWrite::write::<T>(&result, buf, ())
                .with_context(|| ::alloc::__export::must_use({
                    let res = ::alloc::fmt::format(
                        format_args!("Writing mapped value for field \'{0}\'", "value"),
                    );
                    res
                }))?;
        }
        Ok(())
    }
}
impl StateSync for Foo {
    type SyncCtx = ();
    fn sync(&mut self, (): ()) -> ParselyResult<()> {
        self.value
            .sync(())
            .with_context(|| ::alloc::__export::must_use({
                let res = ::alloc::fmt::format(
                    format_args!("Syncing field \'{0}\'", "value"),
                );
                res
            }))?;
        Ok(())
    }
}
