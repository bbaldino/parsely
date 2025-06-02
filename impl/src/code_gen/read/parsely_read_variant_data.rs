use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use super::parsely_read_field_data::ParselyReadFieldData;
use crate::syn_helpers::MemberExts;

#[derive(Debug)]
pub(crate) struct ParselyReadVariantData {
    pub(crate) enum_name: syn::Ident,
    pub(crate) ident: syn::Ident,
    pub(crate) id: syn::Expr,
    pub(crate) discriminant: Option<syn::Expr>,
    pub(crate) fields: Vec<ParselyReadFieldData>,
}

impl ParselyReadVariantData {
    /// Returns true if this variant contains named fields, false otherwise
    fn named_fields(&self) -> bool {
        self.fields
            .iter()
            .any(|f| matches!(f.common.ident, syn::Member::Named(_)))
    }
}

impl ToTokens for ParselyReadVariantData {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let arm_expr = &self.id;
        let arm_body = if let Some(ref discriminant) = self.discriminant {
            quote! {
                #discriminant
            }
        } else {
            let fields = &self.fields;
            quote! {
                #(#fields)*


            }
        };
        let field_names = self
            .fields
            .iter()
            .map(|f| f.common.ident.as_variable_name().to_owned())
            .collect::<Vec<_>>();
        let enum_name = &self.enum_name;
        let variant_name = &self.ident;
        // TODO: don't think we're handling discriminant correctly here
        if self.fields.is_empty() {
            tokens.extend(quote! {
                #arm_expr => {
                    #arm_body

                    Ok(#enum_name::#variant_name)
                }
            })
        } else if self.named_fields() {
            tokens.extend(quote! {
                #arm_expr => {
                    #arm_body

                    Ok(#enum_name::#variant_name { #(#field_names,)* })
                }
            })
        } else {
            tokens.extend(quote! {
                #arm_expr => {
                    #arm_body

                    Ok(#enum_name::#variant_name(#(#field_names,)* ))
                }
            })
        }
    }
}
