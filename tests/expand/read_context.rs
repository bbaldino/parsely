use parsely_rs::*;

#[derive(ParselyRead)]
#[parsely_read(required_context("some_context_value: u8"))]
struct ReadContext {
    #[parsely_read(assign_from = "some_context_value")]
    one: u8,
}
