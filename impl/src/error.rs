pub type ParselyResult<T> = anyhow::Result<T>;

pub trait IntoParselyResult<T> {
    fn into_parsely_result(self) -> ParselyResult<T>;
}

impl<T> IntoParselyResult<T> for T {
    fn into_parsely_result(self) -> ParselyResult<T> {
        Ok(self)
    }
}

impl<T, E> IntoParselyResult<T> for Result<T, E>
where
    E: Into<anyhow::Error>,
{
    fn into_parsely_result(self) -> ParselyResult<T> {
        self.map_err(Into::into)
    }
}

pub fn wrap_in_parsely_result<T>(value: impl IntoParselyResult<T>) -> ParselyResult<T> {
    value.into_parsely_result()
}
