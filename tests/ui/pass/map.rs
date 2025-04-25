use parsely_rs::*;

#[derive(ParselyRead, ParselyWrite)]
struct Foo {
    // #[parsely_read(map = "|v: u8| -> ParselyResult<String> { Ok(v.to_string()) }")]
    // #[parsely_write(map = "|v: &str| -> ParselyResult<u8> { Ok(v.parse()?) }")]
    #[parsely_read(map = "|v: u8| { v.to_string() }")]
    #[parsely_write(map = "|v: &str| { v.parse::<u8>() }")]
    value: String,
}

fn main() {
    let mut bits = Bits::from_static_bytes(&[42]);

    let foo = Foo::read::<NetworkOrder>(&mut bits, ()).expect("successful parse");
    assert_eq!(foo.value, "42");

    let mut bits_mut = BitsMut::new();

    let foo = Foo {
        value: String::from("42"),
    };

    foo.write::<NetworkOrder>(&mut bits_mut, ())
        .expect("successful write");
    assert_eq!(bits.get_u8().unwrap(), 42);
}
