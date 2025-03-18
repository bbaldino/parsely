use parsely::*;

#[derive(ParselyRead)]
struct Foo {
    data_len: u8,
    #[parsely_read(count = "data_len")]
    data: Vec<u8>,
}

fn main() {
    let data = vec![2, 1, 2, 3];
    let mut cursor = BitCursor::from_vec(data);

    let foo = Foo::read::<parsely::NetworkOrder>(&mut cursor, ()).expect("successful parse");
    assert_eq!(foo.data.len(), 2);
}
