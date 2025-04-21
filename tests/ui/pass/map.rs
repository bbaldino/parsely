use bitvec::prelude::*;
use parsely::*;

#[derive(ParselyRead, ParselyWrite)]
struct Foo {
    #[parsely_read(map = "|v: u1| -> ParselyResult<bool> { Ok(v > 0) }")]
    #[parsely_write(map = "|v: &bool| -> ParselyResult<u1> { Ok(u1::from(*v)) }")]
    one: bool,
}

fn main() {
    let mut bits = Bits::from_static_bytes(&[0b10101010]);

    let foo = Foo::read::<NetworkOrder>(&mut bits, ()).expect("successful parse");
    assert!(foo.one);

    let mut bits_mut = BitsMut::new();

    let foo = Foo { one: true };

    foo.write::<NetworkOrder>(&mut bits_mut, ())
        .expect("successful write");
    assert_eq!(&bits_mut[..], bits![u8, Msb0; 1]);
}
