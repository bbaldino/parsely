use parsely_rs::*;

#[derive(ParselyRead, ParselyWrite)]
#[parsely(alignment = 4)]
struct Foo {
    one: u8,
}
