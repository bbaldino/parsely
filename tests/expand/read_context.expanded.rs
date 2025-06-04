use parsely_rs::*;
#[parsely_read(required_context("some_context_value: u8"))]
struct ReadContext {
    #[parsely_read(assign_from = "some_context_value")]
    one: u8,
}
impl<B: BitBuf> ::parsely_rs::ParselyRead<B> for ReadContext {
    type Ctx = (u8,);
    fn read<T: ::parsely_rs::ByteOrder>(
        buf: &mut B,
        (some_context_value,): (u8,),
    ) -> ::parsely_rs::ParselyResult<Self> {
        let one = ParselyResult::<_>::Ok(some_context_value)
            .with_context(|| "Reading field 'one'")?;
        Ok(Self { one })
    }
}
