use parsely_rs::*;

#[derive(ParselyRead)]
struct Foo {
    #[parsely_read(while_pred = "buf.remaining_bytes() > 0")]
    data: Vec<u8>,
}

fn main() {
    let mut bits = Bits::from_static_bytes(&[2, 1, 2, 3]);

    let foo = Foo::read::<NetworkOrder>(&mut bits, ()).expect("successful parse");
    assert_eq!(foo.data.len(), 4);
}
