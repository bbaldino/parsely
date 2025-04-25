use parsely_rs::*;

#[derive(ParselyRead, ParselyWrite)]
struct Foo {
    #[parsely(assertion = "|v: &u8| *v % 2 == 0")]
    value: u8,
}
