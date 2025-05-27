use proc_macro2::TokenStream;
use quote::quote;

use crate::model_types::CollectionLimit;

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
