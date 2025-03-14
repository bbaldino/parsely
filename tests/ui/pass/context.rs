use parsely::*;

#[derive(ParselyRead, ParselyWrite)]
#[parsely_read(required_context("size_one: u32", "size_two: u32"))]
struct Inner {
    #[parsely_read(count = "size_one")]
    data: Vec<u8>,
    #[parsely_read(count = "size_two")]
    data2: Vec<u8>,
}

#[derive(ParselyRead, ParselyWrite)]
#[parsely_read(required_context("size: u32"))]
struct Foo {
    #[parsely_read(count = "size")]
    data: Vec<u8>,
    #[parsely_read(context("size / 2", "size / 2"))]
    inner: Inner,
}

fn main() {
    let data = vec![1, 2, 3, 4];
    let mut cursor = BitCursor::from_vec(data);

    let foo = Foo::read::<parsely::NetworkOrder, _>(&mut cursor, (2,)).expect("successful parse");

    // Should have only read 2 values
    assert_eq!(foo.data.len(), 2);
    assert_eq!(foo.inner.data.len(), 1);
    assert_eq!(foo.inner.data2.len(), 1);
}
