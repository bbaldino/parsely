use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    get_crate_name,
    model_types::{wrap_write_with_padding_handling, TypedFnArgList},
    syn_helpers::TypeExts,
    ParselyWriteData, ParselyWriteFieldData,
};

pub fn generate_parsely_write_impl(data: ParselyWriteData) -> TokenStream {
    let struct_name = data.ident;
    if data.data.is_struct() {
        generate_parsely_write_impl_struct(
            struct_name,
            data.data.take_struct().unwrap(),
            data.required_context,
            data.alignment,
            data.sync_args,
        )
    } else {
        todo!()
    }
}

fn generate_parsely_write_impl_struct(
    struct_name: syn::Ident,
    fields: darling::ast::Fields<ParselyWriteFieldData>,
    required_context: Option<TypedFnArgList>,
    struct_alignment: Option<usize>,
    sync_args: Option<TypedFnArgList>,
) -> TokenStream {
    let crate_name = get_crate_name();
    let (context_assignments, context_types) = if let Some(ref required_context) = required_context
    {
        (required_context.assignments(), required_context.types())
    } else {
        (Vec::new(), Vec::new())
    };
    // TODO: clean this up like the read code
    let field_writes = fields
        .iter()
        .map(|f| {
            let field_name = f.ident.as_ref().expect("Field has a name");
            let field_name_string = field_name.to_string();
            let write_type = f.buffer_type();
            // Context values that we need to pass to this field's ParselyWrite::write method
            let context_values = f.context_values();

            let mut field_write_output = TokenStream::new();

            if let Some(ref assertion) = f.common.assertion {
                assertion.to_write_assertion_tokens(&field_name_string, &mut field_write_output);
            }

            // TODO: these write calls should be qualified.  Something like <#write_type as
            // ParselyWrite>::write
            if let Some(ref map_expr) = f.common.map {
                map_expr.to_write_map_tokens(field_name, &mut field_write_output);
            } else if f.ty.is_option() {
                field_write_output.extend(quote! {
                    if let Some(ref v) = self.#field_name {
                        #write_type::write::<T>(v, buf, (#(#context_values,)*)).with_context(|| format!("Writing field '{}'", #field_name_string))?;
                    }
                });
            } else if f.ty.is_collection() {
                field_write_output.extend(quote! {
                    self.#field_name.iter().enumerate().map(|(idx, v)| {
                        #write_type::write::<T>(v, buf, (#(#context_values,)*)).with_context(|| format!("Index {idx}"))
                    }).collect::<ParselyResult<Vec<_>>>().with_context(|| format!("Writing field '{}'", #field_name_string))?;
                });
            } else {
                field_write_output.extend(quote! {
                    #write_type::write::<T>(&self.#field_name, buf, (#(#context_values,)*)).with_context(|| format!("Writing field '{}'", #field_name_string))?;
                });
            }

            field_write_output = if let Some(alignment) = f.common.alignment {
                wrap_write_with_padding_handling(field_name, alignment, field_write_output)
            } else {
                field_write_output
            };

            field_write_output
        })
        .collect::<Vec<TokenStream>>();

    // sync_args_types lays out the types inside this struct's sync method args tuple. Unless a
    // 'sync_args' attribute was used, by default it doesn't need any.
    let (sync_args_variables, sync_args_types) = if let Some(ref sync_args) = sync_args {
        (sync_args.names(), sync_args.types())
    } else {
        (Vec::new(), Vec::new())
    };

    let sync_field_calls = fields
        .iter()
        // 'sync_with' attirbutes mean this field's 'sync' method needs to be called with some data
        // Iterate over all fields and either:
        // a) execute the expression provided in the sync_expr attribute
        // b) call the sync function on that type with any provided sync_with arguments
        .map(|f| {
            let field_name = f.ident.as_ref().expect("Field has a name");
            let field_name_string = field_name.to_string();
            if let Some(ref sync_expr) = f.sync_expr {
                quote! {
                    self.#field_name = (#sync_expr).into_parsely_result_read().with_context(|| format!("Syncing field '{}'", #field_name_string))?;
                }
            } else if f.sync_with.is_empty() && f.ty.is_wrapped() {
                // We'll allow this combination to skip a call to sync: for types like Option<T> or
                // Vec<T>, synchronization is only going to make sense if a custom function was
                // provided.
                quote! {}
            } else {
                let sync_with = f.sync_with_expressions();
                quote! {
                    self.#field_name.sync((#(#sync_with,)*)).with_context(|| format!("Syncing field '{}'", #field_name_string))?;
                }
            }
        })
        .collect::<Vec<TokenStream>>();

    let body = if let Some(alignment) = struct_alignment {
        quote! {

            let __bytes_remaining_start = buf.remaining_mut_bytes();

            #(#field_writes)*

            let __bytes_remaining_end = buf.remaining_mut_bytes();
            let mut __amount_written = __bytes_remaining_start - __bytes_remaining_end;
            while __amount_written % #alignment != 0 {
                let _ = buf.put_u8(0).context("padding")?;
                __amount_written += 1;
            }

        }
    } else {
        quote! {
            #(#field_writes)*
        }
    };

    quote! {
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
    }
}
