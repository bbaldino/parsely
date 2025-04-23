use parsely::*;

#[derive(ParselyRead)]
struct Foo {
    has_value: bool,
    #[parsely_read(when = "has_value")]
    value: Option<u7>,
}

fn main() {
    let mut bits = Bits::from_static_bytes(&[0b1_0000001, 2]);
    let foo = Foo::read::<parsely::NetworkOrder>(&mut bits, ()).expect("successful parse");

    assert_eq!(foo.value, Some(u7::new(1)));
}
