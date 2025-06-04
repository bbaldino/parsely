use parsely_rs::*;
#[parsely(key_type = "u8")]
enum Foo {
    #[parsely(id = 1)]
    One,
    #[parsely(id = 2)]
    Two(u8),
    #[parsely(id = 3)]
    Three { bar: u8, baz: u16 },
}
#[automatically_derived]
impl ::core::fmt::Debug for Foo {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            Foo::One => ::core::fmt::Formatter::write_str(f, "One"),
            Foo::Two(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Two", &__self_0)
            }
            Foo::Three { bar: __self_0, baz: __self_1 } => {
                ::core::fmt::Formatter::debug_struct_field2_finish(
                    f,
                    "Three",
                    "bar",
                    __self_0,
                    "baz",
                    &__self_1,
                )
            }
        }
    }
}
impl<B: BitBuf> ::parsely_rs::ParselyRead<B> for Foo {
    type Ctx = ();
    fn read<T: ::parsely_rs::ByteOrder>(
        buf: &mut B,
        (): (),
    ) -> ::parsely_rs::ParselyResult<Self> {
        let match_value = <u8 as ::parsely_rs::ParselyRead<_>>::read::<T>(buf, ())
            .with_context(|| ::alloc::__export::must_use({
                let res = ::alloc::fmt::format(
                    format_args!("Tag for enum \'{0}\'", "Foo"),
                );
                res
            }))?;
        match match_value {
            1 => Ok(Foo::One),
            2 => {
                let field_0 = u8::read::<T>(buf, ())
                    .with_context(|| "Reading field 'Field 0'")?;
                Ok(Foo::Two(field_0))
            }
            3 => {
                let bar = u8::read::<T>(buf, ()).with_context(|| "Reading field 'bar'")?;
                let baz = u16::read::<T>(buf, ())
                    .with_context(|| "Reading field 'baz'")?;
                Ok(Foo::Three { bar, baz })
            }
            _ => {
                ParselyResult::<
                    _,
                >::Err(
                    ::anyhow::__private::must_use({
                        let error = ::anyhow::__private::format_err(
                            format_args!("No arms matched value"),
                        );
                        error
                    }),
                )
            }
        }
    }
}
impl<B: BitBufMut> ::parsely_rs::ParselyWrite<B> for Foo {
    type Ctx = ();
    fn write<T: ByteOrder>(&self, buf: &mut B, (): Self::Ctx) -> ParselyResult<()> {
        match self {
            Foo::One => {
                let tag_value: u8 = 1;
                ::parsely_rs::ParselyWrite::write::<T>(&tag_value, buf, ())?;
            }
            Foo::Two(ref field_0) => {
                let tag_value: u8 = 2;
                ::parsely_rs::ParselyWrite::write::<T>(&tag_value, buf, ())?;
                u8::write::<T>(&field_0, buf, ())
                    .with_context(|| ::alloc::__export::must_use({
                        let res = ::alloc::fmt::format(
                            format_args!("Writing field \'{0}\'", "Field 0"),
                        );
                        res
                    }))?;
            }
            Foo::Three { ref bar, ref baz } => {
                let tag_value: u8 = 3;
                ::parsely_rs::ParselyWrite::write::<T>(&tag_value, buf, ())?;
                u8::write::<T>(&bar, buf, ())
                    .with_context(|| ::alloc::__export::must_use({
                        let res = ::alloc::fmt::format(
                            format_args!("Writing field \'{0}\'", "bar"),
                        );
                        res
                    }))?;
                u16::write::<T>(&baz, buf, ())
                    .with_context(|| ::alloc::__export::must_use({
                        let res = ::alloc::fmt::format(
                            format_args!("Writing field \'{0}\'", "baz"),
                        );
                        res
                    }))?;
            }
            _ => {
                ParselyResult::<
                    (),
                >::Err(
                    ::anyhow::__private::must_use({
                        let error = ::anyhow::__private::format_err(
                            format_args!("No arms matched self"),
                        );
                        error
                    }),
                )?
            }
        }
        Ok(())
    }
}
impl ::parsely_rs::StateSync for Foo {
    type SyncCtx = ();
    fn sync(&mut self, (): ()) -> ParselyResult<()> {
        Ok(())
    }
}
