use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    model_types::CollectionLimit,
    parsely_data::read::{
        parsely_read_enum_data::ParselyReadEnumData,
        parsely_read_struct_data::ParselyReadStructData,
    },
    ParselyReadReceiver,
};

pub fn generate_parsely_read_impl(data: ParselyReadReceiver) -> TokenStream {
    if data.data.is_struct() {
        let struct_data = ParselyReadStructData::try_from(data).unwrap();
        quote! {
            #struct_data
        }
    } else {
        let enum_data = ParselyReadEnumData::try_from(data).unwrap();
        quote! {
            #enum_data
        }
    }
}

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

pub(crate) fn wrap_in_optional(when_expr: &syn::Expr, inner: TokenStream) -> TokenStream {
    quote! {
        if #when_expr {
            Some(#inner)
        } else {
            None
        }
    }
}
