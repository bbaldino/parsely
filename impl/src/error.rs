use crate::parsely_write::ParselyWrite;

pub type ParselyResult<T> = anyhow::Result<T>;

/// Helper trait to coerce values of both `T: ParselyWrite` and `Result<T, E>: E:
/// Into<anyhow::Error>` into `ParselyResult<T>`.  We need a trait specifically for writing because
/// if we don't bound the impl for `T` in some way there's ambiguity: the compiler doesn't know if
/// we want `ParselyResult<T>` or `ParselyResult<Result<T, E>>`.
pub trait IntoWritableParselyResult<T> {
    fn into_parsely_result(self) -> ParselyResult<T>;
}

impl<T> IntoWritableParselyResult<T> for T
where
    T: ParselyWrite,
{
    fn into_parsely_result(self) -> ParselyResult<T> {
        Ok(self)
    }
}

impl<T, E> IntoWritableParselyResult<T> for Result<T, E>
where
    E: Into<anyhow::Error>,
{
    fn into_parsely_result(self) -> ParselyResult<T> {
        self.map_err(Into::into)
    }
}

/// When we need to convert an expression that may or may not be wrapped in a Result on the _read_
/// path, we can rely on the fact that we'll eventually be assigning the value to a field with a
/// concrete type and we can rely on type inference in order to figure out what that should be.
/// Because of that we don't want/need the `ParselyWrite` trait bounds on the impl like we have
/// above for the writable side, so we need a different trait here.
pub trait IntoParselyResult<T> {
    fn into_parsely_result_read(self) -> ParselyResult<T>;
}

impl<T> IntoParselyResult<T> for T {
    fn into_parsely_result_read(self) -> ParselyResult<T> {
        Ok(self)
    }
}

impl<T, E> IntoParselyResult<T> for Result<T, E>
where
    E: Into<anyhow::Error>,
{
    fn into_parsely_result_read(self) -> ParselyResult<T> {
        self.map_err(Into::into)
    }
}
