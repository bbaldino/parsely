use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::syn_helpers::MemberExts;

pub(crate) fn wrap_write_with_padding_handling(
    element_ident: &syn::Member,
    alignment: usize,
    inner: TokenStream,
) -> TokenStream {
    let bytes_written_before_ident = format_ident!(
        "__bytes_written_before_{}_write",
        element_ident.as_friendly_string()
    );

    quote! {
        let #bytes_written_before_ident = buf.remaining_mut_bytes();

        #inner

        while (#bytes_written_before_ident - buf.remaining_mut_bytes()) % #alignment != 0 {
            buf.put_u8(0).context("adding padding")?;
        }
    }
}

#[derive(Debug)]
pub(crate) enum ParentType {
    Struct,
    Enum,
}
