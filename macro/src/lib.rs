use proc_macro::TokenStream;
use syn::parse_macro_input;

#[proc_macro_derive(ParselyRead, attributes(parsely, parsely_read))]
pub fn derive_parsely_read(item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as proc_macro2::TokenStream);

    match parsely_impl::derive_parsely_read(item) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(ParselyWrite, attributes(parsely, parsely_write))]
pub fn derive_parsely_write(item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as proc_macro2::TokenStream);

    match parsely_impl::derive_parsely_write(item) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
