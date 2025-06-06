use crate::{get_crate_name, syn_helpers::MemberExts};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use super::parsely_write_field_data::ParselyWriteFieldData;

pub(crate) struct ParselyWriteVariantData {
    pub(crate) enum_name: syn::Ident,
    pub(crate) ident: syn::Ident,
    pub(crate) id: syn::Expr,
    pub(crate) discriminant: Option<syn::Expr>,
    pub(crate) key_type: syn::Type,
    pub(crate) fields: Vec<ParselyWriteFieldData>,
}

impl ParselyWriteVariantData {
    /// Returns true if this variant contains named fields, false otherwise
    fn named_fields(&self) -> bool {
        self.fields
            .iter()
            .any(|f| matches!(f.common.ident, syn::Member::Named(_)))
    }
}

impl ToTokens for ParselyWriteVariantData {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let crate_name = get_crate_name();
        let enum_name = &self.enum_name;
        let variant_name = &self.ident;

        let tag_expr = &self.id;
        let tag_type = &self.key_type;
        let tag_write = quote! {
            let tag_value: #tag_type = #tag_expr;
            ::#crate_name::ParselyWrite::write::<T>(&tag_value, buf, ())?;
        };

        let body = if let Some(ref discriminant) = self.discriminant {
            quote! {
                #enum_name::#variant_name => {
                    #tag_write
                    let discriminant_value = #discriminant;

                    discriminant_value.write::<T>(buf, ()).context("Writing discriminant value of variant #variant_name")
                }
            }
        } else if !self.fields.is_empty() {
            let fields = &self.fields;
            let field_variable_names = fields
                .iter()
                .map(|f| f.common.ident.as_variable_name())
                .collect::<Vec<_>>();
            if self.named_fields() {
                quote! {
                    #enum_name::#variant_name { #(ref #field_variable_names,)* } => {
                        #tag_write
                        #(#fields)*
                    }
                }
            } else {
                quote! {
                    #enum_name::#variant_name(#(ref #field_variable_names,)*) => {
                        #tag_write
                        #(#fields)*
                    }
                }
            }
        } else {
            quote! {
                #enum_name::#variant_name => {
                    #tag_write
                }
            }
        };

        tokens.extend(body);
    }
}
