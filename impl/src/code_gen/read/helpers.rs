use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{model_types::CollectionLimit, syn_helpers::MemberExts};

pub(crate) fn generate_plain_read(ty: &syn::Type, context_values: &[syn::Expr]) -> TokenStream {
    quote! {
        #ty::read::<T>(buf, (#(#context_values,)*))
    }
}

pub(crate) fn generate_collection_read(
    limit: &CollectionLimit,
    ty: &syn::Type,
    context_values: &[syn::Expr],
) -> TokenStream {
    let plain_read = generate_plain_read(ty, context_values);
    match limit {
        CollectionLimit::Count(count) => {
            quote! {
                (|| {
                    let item_count = #count;
                    let mut items: Vec<#ty> = Vec::with_capacity(item_count as usize);
                    for idx in 0..item_count {
                        let item = #plain_read.with_context(|| format!("Index {idx}"))?;
                        items.push(item);
                    }
                    ParselyResult::Ok(items)

                })()
            }
        }
        CollectionLimit::While(pred) => {
            // Since this is multiple statements we wrap it in a closure
            quote! {
                (|| {
                    let mut values: Vec<ParselyResult<#ty>> = Vec::new();
                    let mut idx = 0;
                    while (#pred) {
                        values.push(#plain_read.with_context( || format!("Read {idx}")));
                        idx += 1
                    }
                    values.into_iter().collect::<ParselyResult<Vec<#ty>>>()
                })()
            }
        }
    }
}

pub(crate) fn wrap_read_with_padding_handling(
    element_ident: &syn::Member,
    alignment: usize,
    inner: TokenStream,
) -> TokenStream {
    let bytes_read_before_ident = format_ident!(
        "__bytes_read_before_{}_read",
        element_ident.as_friendly_string()
    );
    quote! {
        let #bytes_read_before_ident = buf.remaining_bytes();

        #inner

        while (#bytes_read_before_ident - buf.remaining_bytes()) % #alignment != 0 {
            buf.get_u8().context("consuming padding")?;
        }
    }
}
