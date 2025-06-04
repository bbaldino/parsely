use proc_macro2::TokenStream;
use quote::quote;

pub(crate) fn wrap_in_optional(condition: &syn::Expr, inner: TokenStream) -> TokenStream {
    quote! {
        if #condition {
            Some(#inner)
        } else {
            None
        }
    }
}
