use parsely_rs::*;
struct Foo {
    #[parsely(assertion = "|v: &u8| *v % 2 == 0")]
    value: u8,
}
impl<B: BitBuf> ::parsely_rs::ParselyRead<B, ()> for Foo {
    fn read<T: ::parsely_rs::ByteOrder>(
        buf: &mut B,
        _ctx: (),
    ) -> ::parsely_rs::ParselyResult<Self> {
        let value = u8::read::<T>(buf, ())
            .and_then(|read_value| {
                let assertion_func = |v: &u8| *v % 2 == 0;
                if !assertion_func(&read_value) {
                    return ::anyhow::__private::Err(
                        ::anyhow::Error::msg(
                            ::alloc::__export::must_use({
                                let res = ::alloc::fmt::format(
                                    format_args!(
                                        "Assertion failed: value of field \'{0}\' (\'{1:?}\') didn\'t pass assertion: \'{2}\'",
                                        "value", read_value, | v : & u8 | * v % 2 == 0,
                                    ),
                                );
                                res
                            }),
                        ),
                    );
                }
                Ok(read_value)
            })
            .with_context(|| "Reading field 'value'")?;
        Ok(Self { value })
    }
}
impl<B: BitBufMut> ::parsely_rs::ParselyWrite<B, ()> for Foo {
    fn write<T: ::parsely_rs::ByteOrder>(
        &self,
        buf: &mut B,
        ctx: (),
    ) -> ::parsely_rs::ParselyResult<()> {
        let __value_assertion_func = |v: &u8| *v % 2 == 0;
        if !__value_assertion_func(&self.value) {
            return ::anyhow::__private::Err(
                ::anyhow::Error::msg(
                    ::alloc::__export::must_use({
                        let res = ::alloc::fmt::format(
                            format_args!(
                                "Assertion failed: value of field \'{0}\' (\'{1:?}\') didn\'t pass assertion: \'{2}\'",
                                "value", self.value, | v : & u8 | * v % 2 == 0,
                            ),
                        );
                        res
                    }),
                ),
            );
        }
        u8::write::<T>(&self.value, buf, ())
            .with_context(|| ::alloc::__export::must_use({
                let res = ::alloc::fmt::format(
                    format_args!("Writing field \'{0}\'", "value"),
                );
                res
            }))?;
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
