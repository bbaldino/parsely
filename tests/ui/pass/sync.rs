use parsely::*;

#[derive(Debug, ParselyWrite)]
// sync_args denotes that this type's sync method takes additional arguments.  By default a type's
// sync field takes no arguments
#[parsely_write(sync_args("payload_length_bytes: &u16"))]
struct Header {
    version: u8,
    packet_type: u8,
    // sync_func attributes add lines to this type's sync method to update its own fields.  In this
    // case the length field should equal the size of the header (4 bytes) + the sie of the
    // payload.
    #[parsely_write(sync_func = "|(plb,): (&u16,)| -> ParselyResult<u16> { Ok(plb + 4) }")]
    length_bytes: u16,
}

#[derive(Debug, ParselyWrite)]
struct Packet {
    // sync_with attributes add lines to this type's sync method to call sync on its fields (and
    // what arguments to pass)
    #[parsely_write(sync_with = "&(self.data.len() as u16)")]
    header: Header,
    data: Vec<u8>,
}

fn main() {
    let mut packet = Packet {
        header: Header {
            version: 1,
            packet_type: 2,
            length_bytes: 0,
        },
        data: vec![1, 2, 3, 4],
    };

    packet.sync(()).unwrap();

    assert_eq!(packet.header.length_bytes, 8);
}
