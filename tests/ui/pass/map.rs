use parsely::*;

#[derive(ParselyRead, ParselyWrite)]
struct Foo {
    #[parsely_read(map = "|v: u1| -> ParselyResult<bool> { Ok(v > 0) }")]
    #[parsely_write(map = "|v: &bool| -> ParselyResult<u1> { Ok(u1::from(*v)) }")]
    one: bool,
}

fn main() {
    let data = vec![0b10101010];
    let mut cursor = BitCursor::from_vec(data);

    let foo = Foo::read::<parsely::NetworkOrder, _>(&mut cursor, ()).expect("successful parse");
    assert!(foo.one);

    let data: Vec<u8> = vec![0; 1];
    let mut cursor = BitCursor::from_vec(data);

    let foo = Foo { one: true };

    foo.write::<parsely::NetworkOrder, _>(&mut cursor, ())
        .expect("successful write");
    let data = cursor.into_inner();
    assert_eq!(data[0], true);
}
