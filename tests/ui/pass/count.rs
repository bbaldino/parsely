use parsely::*;

#[derive(ParselyRead)]
struct Foo {
    data_len: u8,
    #[parsely_read(count = "data_len")]
    data: Vec<u8>,
}

fn main() {
    let mut bits = Bits::from_static_bytes(&[2, 1, 2, 3]);

    let foo = Foo::read::<parsely::NetworkOrder>(&mut bits, ()).expect("successful parse");
    assert_eq!(foo.data.len(), 2);
}
