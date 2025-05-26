use anyhow::anyhow;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::{
    get_crate_name,
    model_types::{wrap_read_with_padding_handling2, MemberIdent},
    ParselyReadReceiver, TypedFnArgList,
};

use super::parsely_read_field_data::ParselyReadFieldData;

/// A struct which represents all information needed for generating a `ParselyRead` implementation
/// for a given struct.
pub(crate) struct ParselyReadStructData {
    pub(crate) ident: syn::Ident,
    pub(crate) style: darling::ast::Style,
    pub(crate) required_context: Option<TypedFnArgList>,
    pub(crate) alignment: Option<usize>,
    pub(crate) fields: Vec<ParselyReadFieldData>,
}

impl TryFrom<ParselyReadReceiver> for ParselyReadStructData {
    type Error = anyhow::Error;

    fn try_from(value: ParselyReadReceiver) -> Result<Self, Self::Error> {
        let (style, struct_receiver_fields) = value
            .data
            .take_struct()
            .ok_or(anyhow!("Not a struct"))?
            .split();
        let data_fields = struct_receiver_fields
            .into_iter()
            .enumerate()
            .map(|(field_index, field)| {
                let ident =
                    MemberIdent::from_ident_or_index(field.ident.as_ref(), field_index as u32);
                ParselyReadFieldData::from_receiver(ident, field)
            })
            .collect::<Vec<_>>();
        Ok(ParselyReadStructData {
            ident: value.ident,
            style,
            required_context: value.required_context,
            alignment: value.alignment,
            fields: data_fields,
        })
    }
}

impl ToTokens for ParselyReadStructData {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let crate_name = get_crate_name();
        let struct_name = &self.ident;
        // Extract out the assignment expressions we'll do to assign the values of the context tuple
        // to the configured variable names, as well as the types of the context tuple.
        let (context_assignments, context_types) =
            if let Some(ref required_context) = self.required_context {
                (required_context.assignments(), required_context.types())
            } else {
                (Vec::new(), Vec::new())
            };

        let fields = &self.fields;
        let field_reads = quote! {
            #(#fields)*
        };

        let body = if let Some(alignment) = self.alignment {
            wrap_read_with_padding_handling2(
                &MemberIdent::from_ident(&self.ident),
                alignment,
                field_reads,
            )
        } else {
            field_reads
        };
        let ctx_var = if context_types.is_empty() {
            format_ident!("_ctx")
        } else {
            format_ident!("ctx")
        };

        let field_names = fields
            .iter()
            .map(|f| f.common.ident.as_variable_name().to_owned())
            .collect::<Vec<_>>();

        // TODO: reduce the duplicated code here
        if self.style.is_struct() {
            tokens.extend(quote! {
            impl<B: BitBuf> ::#crate_name::ParselyRead<B> for #struct_name {
                type Ctx = (#(#context_types,)*);
                fn read<T: ::#crate_name::ByteOrder>(buf: &mut B, #ctx_var: (#(#context_types,)*)) -> ::#crate_name::ParselyResult<Self> {
                    #(#context_assignments)*

                    #body

                    Ok(Self { #(#field_names,)* })
                }
            }
        })
        } else {
            tokens.extend(quote! {
            impl<B: BitBuf> ::#crate_name::ParselyRead<B> for #struct_name {
                type Ctx = (#(#context_types,)*);
                fn read<T: ::#crate_name::ByteOrder>(buf: &mut B, #ctx_var: (#(#context_types,)*)) -> ::#crate_name::ParselyResult<Self> {
                    #(#context_assignments)*

                    #body

                    Ok(Self(#(#field_names,)* ))
                }
            }
        })
        }
    }
}
