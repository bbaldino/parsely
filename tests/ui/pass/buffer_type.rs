use bitvec::prelude::*;
use parsely::*;

// A custom buffer type that will be used
pub trait CustomBuffer: BitBuf + BitBufMut {
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
        buf.put_u8(buf.write_value())?;

        Ok(())
    }
}

impl StateSync<()> for Inner {
    fn sync(&mut self, _sync_ctx: ()) -> ParselyResult<()> {
        Ok(())
    }
}

#[derive(Debug, ParselyRead, ParselyWrite)]
#[parsely(buffer_type = "CustomBuffer")]
struct MyStruct {
    inner: Inner,
}

impl<T> CustomBuffer for T where T: BitBuf + BitBufMut {}

pub fn main() {
    // The dummy 'CustomBuffer' type requires both Buf & BufMut for simplicity, so use BitsMut
    // here even though we're just reading.
    let mut bits = BitsMut::zeroed_bytes(1);

    let ms = MyStruct::read::<NetworkOrder>(&mut bits, ()).expect("successful read");
    assert_eq!(ms.inner.value, 42);

    let mut bits_mut = BitsMut::new();
    ms.write::<NetworkOrder>(&mut bits_mut, ())
        .expect("successful write");
    assert_eq!(&bits_mut[..], 24u8.view_bits::<Msb0>());
}
