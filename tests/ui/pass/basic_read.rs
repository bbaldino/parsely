use parsely_rs::*;

#[derive(ParselyRead)]
struct Foo {
    one: bool,
}

fn main() {
    let mut cursor = Bits::from_static_bytes(&[0b10101010]);

    let foo = Foo::read::<NetworkOrder>(&mut cursor, ()).expect("successful parse");
    assert!(foo.one);
}
