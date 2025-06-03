use ::anyhow::anyhow;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::{anyhow, get_crate_name, syn_helpers::MemberExts, ParselyReadReceiver, TypedFnArgList};

use super::{
    helpers::wrap_read_with_padding_handling, parsely_read_field_data::ParselyReadFieldData,
    parsely_read_variant_data::ParselyReadVariantData,
};

/// A struct which represents all information needed for generating a `ParselyRead` implementation
/// for a given struct.
#[derive(Debug)]
pub(crate) struct ParselyReadEnumData {
    pub(crate) ident: syn::Ident,
    pub(crate) required_context: TypedFnArgList,
    pub(crate) alignment: Option<usize>,
    pub(crate) key_type: syn::Type,
    pub(crate) variants: Vec<ParselyReadVariantData>,
}

impl TryFrom<ParselyReadReceiver> for ParselyReadEnumData {
    type Error = anyhow::Error;

    fn try_from(value: ParselyReadReceiver) -> Result<Self, Self::Error> {
        let key_type = value
            .key_type
            .ok_or(anyhow!("'key_type' attribute is required on enums"))?;
        let variants = value
            .data
            .take_enum()
            .ok_or(anyhow!("Not an enum"))?
            .into_iter()
            .map(|v| {
                let data_fields = v
                    .fields
                    .into_iter()
                    .enumerate()
                    .map(|(field_index, field)| {
                        let ident = syn::Member::from_ident_or_index(
                            field.ident.as_ref(),
                            field_index as u32,
                        );
                        ParselyReadFieldData::from_receiver(ident, field)
                    })
                    .collect::<Vec<_>>();
                ParselyReadVariantData {
                    enum_name: value.ident.clone(),
                    ident: v.ident,
                    id: v.id,
                    discriminant: v.discriminant,
                    fields: data_fields,
                }
            })
            .collect::<Vec<_>>();

        Ok(ParselyReadEnumData {
            ident: value.ident,
            key_type,
            required_context: value.required_context,
            alignment: value.alignment,
            variants,
        })
    }
}

impl ToTokens for ParselyReadEnumData {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let crate_name = get_crate_name();
        let enum_name = &self.ident;
        let enum_name_string = enum_name.to_string();
        let (context_variables, context_types) =
            (self.required_context.names(), self.required_context.types());

        let match_type = &self.key_type;

        let match_arms = &self.variants;
        let body = quote! {
            let match_value = <#match_type as ::#crate_name::ParselyRead<_>>::read::<T>(buf, ()).with_context(|| format!("Tag for enum '{}'", #enum_name_string))?;
            match match_value {
                #(#match_arms)*
                _ => ParselyResult::<_>::Err(anyhow!("No arms matched value")),
            }
        };

        let body = if let Some(alignment) = self.alignment {
            wrap_read_with_padding_handling(
                &syn::Member::Named(self.ident.clone()),
                alignment,
                body,
            )
        } else {
            body
        };

        // TODO: should the enum id be able to be read from the buffer?  we could have it support
        // being an expr that returns a result or not, like other things.  so it could be
        // "buf.get_u8()"
        tokens.extend(quote! {
            impl<B: BitBuf> ::#crate_name::ParselyRead<B> for #enum_name {
                type Ctx = (#(#context_types,)*);
                fn read<T: ::#crate_name::ByteOrder>(buf: &mut B, (#(#context_variables,)*): (#(#context_types,)*)) -> ::#crate_name::ParselyResult<Self> {
                    #body
                }
            }
        });
    }
}
