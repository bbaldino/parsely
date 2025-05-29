use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::{
    code_gen::parsely_common_field_data::ParselyCommonFieldData,
    model_types::{Context, ExprOrFunc, MemberIdent},
    syn_helpers::TypeExts,
    ParselyWriteFieldReceiver,
};

use super::helpers::wrap_write_with_padding_handling;

#[derive(Debug)]
pub(crate) struct ParselyWriteFieldData {
    pub(crate) common: ParselyCommonFieldData,
    /// An expression or function call that will be used to update this field in the generated
    /// `StateSync` implementation for its parent type.
    pub(crate) sync_expr: Option<ExprOrFunc>,
    /// An list of expressions that should be passed as context to this field's sync method.  The
    /// sync method provides an opportunity to synchronize "linked" fields, where one field's value
    /// depends on the value of another.
    pub(crate) sync_with: Context,
}

impl ParselyWriteFieldData {
    pub(crate) fn from_receiver(
        field_ident: MemberIdent,
        receiver: ParselyWriteFieldReceiver,
    ) -> Self {
        let common = ParselyCommonFieldData {
            ident: field_ident,
            ty: receiver.ty,
            assertion: receiver.common.assertion,
            context: receiver.common.context,
            map: receiver.common.map,
            alignment: receiver.common.alignment,
        };
        Self {
            common,
            sync_expr: receiver.sync_expr,
            sync_with: receiver.sync_with,
        }
    }

    /// Get the context values that need to be passed to the read or write call for this field
    pub(crate) fn sync_with_expressions(&self) -> Vec<syn::Expr> {
        let field_name = self.common.ident.as_friendly_string();
        self.sync_with
            .expressions(&format!("Sync context for field '{field_name}'"))
    }

    /// Get this field's `sync` call expression
    pub(crate) fn to_sync_call_tokens(&self) -> TokenStream {
        let field_name = self.common.ident.field_name();
        let field_name_string = self.common.ident.as_friendly_string();
        if let Some(ref sync_expr) = self.sync_expr {
            quote! {
                self.#field_name = (#sync_expr).into_parsely_result().with_context(|| format!("Syncing field '{}'", #field_name_string))?;
            }
        } else if self.sync_with.is_empty() && self.common.ty.is_wrapped() {
            // We'll allow this combination to skip a call to sync: for types like Option<T> or
            // Vec<T>, synchronization is only going to make sense if a custom function was
            // provided.
            quote! {}
        } else {
            let sync_with = self.sync_with_expressions();
            quote! {
                self.#field_name.sync((#(#sync_with,)*)).with_context(|| format!("Syncing field '{}'", #field_name_string))?;
            }
        }
    }
}

impl ToTokens for ParselyWriteFieldData {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let field_name = self.common.ident.field_name();
        let field_name_string = field_name.to_string();
        let write_type = self.common.buffer_type();
        // Context values that we need to pass to this field's ParselyWrite::write method
        let context_values = self.common.context_values();

        let mut output = TokenStream::new();

        if let Some(ref assertion) = self.common.assertion {
            assertion.to_write_assertion_tokens(&field_name_string, &mut output);
        }

        if let Some(ref map_expr) = self.common.map {
            map_expr.to_write_map_tokens(&field_name, &mut output);
        } else if self.common.ty.is_option() {
            output.extend(quote! {
                    if let Some(ref v) = self.#field_name {
                        #write_type::write::<T>(v, buf, (#(#context_values,)*)).with_context(|| format!("Writing field '{}'", #field_name_string))?;
                    }
                });
        } else if self.common.ty.is_collection() {
            output.extend(quote! {
                    self.#field_name.iter().enumerate().map(|(idx, v)| {
                        #write_type::write::<T>(v, buf, (#(#context_values,)*)).with_context(|| format!("Index {idx}"))
                    }).collect::<ParselyResult<Vec<_>>>().with_context(|| format!("Writing field '{}'", #field_name_string))?;
                });
        } else {
            output.extend(quote! {
                    #write_type::write::<T>(&self.#field_name, buf, (#(#context_values,)*)).with_context(|| format!("Writing field '{}'", #field_name_string))?;
                });
        }

        output = if let Some(alignment) = self.common.alignment {
            wrap_write_with_padding_handling(&self.common.ident, alignment, output)
        } else {
            output
        };

        tokens.extend(output);
    }
}
