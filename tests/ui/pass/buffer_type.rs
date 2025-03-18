use parsely::*;

// A custom buffer type that will be used
pub trait CustomBuffer: BitRead + BitWrite {
    fn read_value(&self) -> u8 {
        42
    }

    fn write_value(&self) -> u8 {
        24
    }
}

#[derive(Debug)]
struct Inner {
    value: u8,
}

// Verify that the custom buffer type is propagated down to the read and write calls correctly
impl<B: CustomBuffer> ParselyRead<B, ()> for Inner {
    fn read<T: ByteOrder>(buf: &mut B, _ctx: ()) -> ParselyResult<Self> {
        Ok(Inner {
            value: buf.read_value(),
        })
    }
}

impl<B: CustomBuffer> ParselyWrite<B, ()> for Inner {
    fn write<T: ByteOrder>(&self, buf: &mut B, _ctx: ()) -> ParselyResult<()> {
        buf.write_u8(buf.write_value())?;

        Ok(())
    }
}

#[derive(Debug, ParselyRead, ParselyWrite)]
#[parsely(buffer_type = "CustomBuffer")]
struct MyStruct {
    inner: Inner,
}

impl<T> CustomBuffer for T where T: BitRead + BitWrite {}

pub fn main() {
    let data: Vec<u8> = vec![0; 1];
    let mut cursor = BitCursor::from_vec(data);

    let ms = MyStruct::read::<NetworkOrder>(&mut cursor, ()).expect("successful read");
    assert_eq!(ms.inner.value, 42);

    let data: Vec<u8> = vec![0; 1];
    let mut write_cursor = BitCursor::from_vec(data);
    ms.write::<NetworkOrder>(&mut write_cursor, ())
        .expect("successful write");
    // println!("data: {:x}", write_cursor.into_inner());
    let data = write_cursor.into_inner().into_vec();
    assert_eq!(data[0], 24);
}
