use parsely_rs::*;

#[derive(ParselyRead, ParselyWrite)]
struct Foo {
    #[parsely_read(map = "|v: u8| { v.to_string() }")]
    #[parsely_write(map = "|v: &str| { v.parse::<u8>() }")]
    value: String,
}
