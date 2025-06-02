use anyhow::anyhow;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::{
    get_crate_name, model_types::TypedFnArgList, syn_helpers::MemberExts, ParselyWriteReceiver,
};

use super::{
    helpers::{wrap_write_with_padding_handling, ParentType},
    parsely_write_field_data::ParselyWriteFieldData,
};

pub(crate) struct ParselyWriteStructData {
    pub(crate) ident: syn::Ident,
    pub(crate) required_context: TypedFnArgList,
    pub(crate) alignment: Option<usize>,
    pub(crate) sync_args: TypedFnArgList,
    pub(crate) fields: Vec<ParselyWriteFieldData>,
}

impl TryFrom<ParselyWriteReceiver> for ParselyWriteStructData {
    type Error = anyhow::Error;

    fn try_from(value: ParselyWriteReceiver) -> Result<Self, Self::Error> {
        let struct_receiver_fields = value.data.take_struct().ok_or(anyhow!("Not a struct"))?;
        let data_fields = struct_receiver_fields
            .into_iter()
            .enumerate()
            .map(|(field_index, field)| {
                let ident =
                    syn::Member::from_ident_or_index(field.ident.as_ref(), field_index as u32);
                ParselyWriteFieldData::from_receiver(ident, ParentType::Struct, field)
            })
            .collect::<Vec<_>>();

        Ok(ParselyWriteStructData {
            ident: value.ident,
            required_context: value.required_context,
            alignment: value.alignment,
            sync_args: value.sync_args,
            fields: data_fields,
        })
    }
}

impl ToTokens for ParselyWriteStructData {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let crate_name = get_crate_name();
        let struct_name = &self.ident;
        let (context_variables, context_types) =
            (self.required_context.names(), self.required_context.types());

        let fields = &self.fields;
        let field_writes = quote! {
            #(#fields)*
        };

        let sync_field_calls = fields
            .iter()
            .map(|f| f.to_sync_call_tokens())
            .collect::<Vec<_>>();

        let (sync_args_variables, sync_args_types) =
            (self.sync_args.names(), self.sync_args.types());

        let body = if let Some(alignment) = self.alignment {
            wrap_write_with_padding_handling(
                &syn::Member::Named(self.ident.clone()),
                alignment,
                field_writes,
            )
        } else {
            field_writes
        };

        tokens.extend(quote! {
            impl<B: BitBufMut> ::#crate_name::ParselyWrite<B> for #struct_name {
                type Ctx = (#(#context_types,)*);
                fn write<T: ByteOrder>(
                    &self,
                    buf: &mut B,
                    (#(#context_variables,)*): Self::Ctx,
                ) -> ParselyResult<()> {

                    #body

                    Ok(())
                }
            }

            impl ::#crate_name::StateSync for #struct_name {
                type SyncCtx = (#(#sync_args_types,)*);
                fn sync(&mut self, (#(#sync_args_variables,)*): (#(#sync_args_types,)*)) -> ParselyResult<()> {
                    #(#sync_field_calls)*

                    Ok(())
                }

            }
        });
    }
}
