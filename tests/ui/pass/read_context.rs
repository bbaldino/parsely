use parsely_rs::*;

#[derive(ParselyRead)]
#[parsely_read(required_context("some_context_value: u8"))]
struct ReadContext {
    #[parsely_read(assign_from = "some_context_value")]
    one: u8,
}

fn main() {
    let mut buf = Bits::from_static_bytes(&[]);

    let value = ReadContext::read::<NetworkOrder>(&mut buf, (42,)).expect("successful parse");
    assert_eq!(value.one, 42);
}
