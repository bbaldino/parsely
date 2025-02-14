# Development Notes

Random notes/thoughts while developing.

### byte order modeling

At first I had the ParselyRead trait look like this:

```rust
pub trait ParselyRead<Ctx>: Sized {
    fn read<T: ByteOrder, B: BitRead>(buf: &mut B, ctx: Ctx) -> ParselyResult<Self>;
}
```

but then I realized that when reading a struct, I had to specify the ByteOrder,
even though I planned to annotate that on the struct itself.  I guess it could
be argued that doing it on the struct _technically_ doesn't make sense, since
you could read a given struct from multiple places?  So maybe it _should_ be on
the read call, and then all other values inherit it?

If we go this direction, then I don't think having a byteorder attribute at the
parsely level makes sense?  And the trait definition can stay as it is.

--> Decided to go with this: it makes the most sense for the byteorder to be
determined at call time, not at definition time.  The byteorder will have to be
passed to the initial read call and then will propagate to all nested calls.
