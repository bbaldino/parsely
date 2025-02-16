use parsely::*;

#[derive(ParselyRead)]
struct Foo {
    #[parsely(fixed = "true")]
    one: bool,
}

fn main() {
    let data = vec![0b10101010];
    let mut cursor = BitCursor::from_vec(data);

    Foo::read::<parsely::NetworkOrder, _>(&mut cursor, ()).expect("successful parse");
}
