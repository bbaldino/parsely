use anyhow::anyhow;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::{
    get_crate_name,
    model_types::{MemberIdent, TypedFnArgList},
    ParselyWriteReceiver,
};

use super::{
    helpers::wrap_write_with_padding_handling, parsely_write_field_data::ParselyWriteFieldData,
};

pub(crate) struct ParselyWriteStructData {
    pub(crate) ident: syn::Ident,
    pub(crate) required_context: Option<TypedFnArgList>,
    pub(crate) alignment: Option<usize>,
    pub(crate) sync_args: Option<TypedFnArgList>,
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
                    MemberIdent::from_ident_or_index(field.ident.as_ref(), field_index as u32);
                ParselyWriteFieldData::from_receiver(ident, field)
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
        let (context_assignments, context_types) =
            if let Some(ref required_context) = self.required_context {
                (required_context.assignments(), required_context.types())
            } else {
                (vec![], vec![])
            };

        let fields = &self.fields;
        let field_writes = quote! {
            #(#fields)*
        };

        let sync_field_calls = fields
            .iter()
            .map(|f| f.to_sync_call_tokens())
            .collect::<Vec<_>>();

        let (sync_args_variables, sync_args_types) = if let Some(ref sync_args) = self.sync_args {
            (sync_args.names(), sync_args.types())
        } else {
            (vec![], vec![])
        };

        let body = if let Some(alignment) = self.alignment {
            wrap_write_with_padding_handling(
                &MemberIdent::from_ident(&self.ident),
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
                    ctx: Self::Ctx,
                ) -> ParselyResult<()> {
                    #(#context_assignments)*

                    #body

                    Ok(())
                }
            }

            impl StateSync for #struct_name {
                type SyncCtx = (#(#sync_args_types,)*);
                fn sync(&mut self, (#(#sync_args_variables,)*): (#(#sync_args_types,)*)) -> ParselyResult<()> {
                    #(#sync_field_calls)*

                    Ok(())
                }

            }
        });
    }
}
