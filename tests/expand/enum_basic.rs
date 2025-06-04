use parsely_rs::*;

#[derive(Debug, ParselyRead, ParselyWrite)]
#[parsely(key_type = "u8")]
enum Foo {
    #[parsely(id = 1)]
    One,
    #[parsely(id = 2)]
    Two(u8),
    #[parsely(id = 3)]
    Three { bar: u8, baz: u16 },
}
