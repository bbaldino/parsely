use anyhow::anyhow;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::{
    get_crate_name, model_types::TypedFnArgList, syn_helpers::MemberExts, ParselyWriteReceiver,
};

use super::{
    helpers::{wrap_write_with_padding_handling, ParentType},
    parsely_write_field_data::ParselyWriteFieldData,
    parsely_write_variant_data::ParselyWriteVariantData,
};

pub(crate) struct ParselyWriteEnumData {
    pub(crate) ident: syn::Ident,
    pub(crate) required_context: TypedFnArgList,
    pub(crate) alignment: Option<usize>,
    pub(crate) sync_args: TypedFnArgList,
    pub(crate) variants: Vec<ParselyWriteVariantData>,
}

impl TryFrom<ParselyWriteReceiver> for ParselyWriteEnumData {
    type Error = anyhow::Error;

    fn try_from(value: ParselyWriteReceiver) -> Result<Self, Self::Error> {
        let key_type = value
            .key_type
            .ok_or(anyhow!("'key' attribute is required on enums"))?;
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
                        ParselyWriteFieldData::from_receiver(ident, ParentType::Enum, field)
                    })
                    .collect::<Vec<_>>();
                ParselyWriteVariantData {
                    enum_name: value.ident.clone(),
                    ident: v.ident,
                    discriminant: v.discriminant,
                    id: v.id,
                    key_type: key_type.clone(),
                    fields: data_fields,
                }
            })
            .collect::<Vec<_>>();
        Ok(ParselyWriteEnumData {
            ident: value.ident,
            required_context: value.required_context,
            alignment: value.alignment,
            sync_args: value.sync_args,
            variants,
        })
    }
}

impl ToTokens for ParselyWriteEnumData {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let crate_name = get_crate_name();
        let enum_name = &self.ident;
        let (context_variables, context_types) =
            (self.required_context.names(), self.required_context.types());
        let match_arms = &self.variants;

        let body = quote! {
            match self {
                #(#match_arms)*
                _ => ParselyResult::<()>::Err(anyhow!("No arms matched self"))?,
            }
        };

        let body = if let Some(alignment) = self.alignment {
            wrap_write_with_padding_handling(
                &syn::Member::Named(self.ident.clone()),
                alignment,
                body,
            )
        } else {
            body
        };

        let (sync_args_variables, sync_args_types) =
            (self.sync_args.names(), self.sync_args.types());

        // TODO: need to think about what the sync impl for an enum should look like and finish
        // that
        tokens.extend(quote! {
            impl<B: BitBufMut> ::#crate_name::ParselyWrite<B> for #enum_name {
                type Ctx = (#(#context_types,)*);
                fn write<T: ByteOrder>(&self, buf: &mut B, (#(#context_variables,)*): Self::Ctx,) -> ParselyResult<()> {
                    #body

                    Ok(())
                }
            }

            impl ::#crate_name::StateSync for #enum_name {
                type SyncCtx = (#(#sync_args_types,)*);
                fn sync(&mut self, (#(#sync_args_variables,)*): (#(#sync_args_types,)*)) -> ParselyResult<()> {
                    Ok(())
                }
            }
        });
    }
}
