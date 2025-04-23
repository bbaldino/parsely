use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    model_types::TypedFnArgList, syn_helpers::TypeExts, ParselyWriteData, ParselyWriteFieldData,
};

pub fn generate_parsely_write_impl(data: ParselyWriteData) -> TokenStream {
    let struct_name = data.ident;
    if data.data.is_struct() {
        generate_parsely_write_impl_struct(
            struct_name,
            data.data.take_struct().unwrap(),
            data.required_context,
            data.buffer_type,
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
    buffer_type: syn::Ident,
    struct_alignment: Option<usize>,
    sync_args: Option<TypedFnArgList>,
) -> TokenStream {
    let (context_assignments, context_types) = if let Some(ref required_context) = required_context
    {
        (required_context.assignments(), required_context.types())
    } else {
        (Vec::new(), Vec::new())
    };
    let field_writes = fields
        .iter()
        .map(|f| {
            let field_name = f.ident.as_ref().expect("Field has a name");
            let field_name_string = field_name.to_string();
            let write_type = f.buffer_type();
            let context_values = f.context_values();
            let mut field_write_output = TokenStream::new();

            if let Some(ref assertion) = f.common.assertion {
                let assertion_string = quote! { #assertion }.to_string();
                field_write_output.extend(quote! {
                    let assertion_func = #assertion;
                    if !assertion_func(&self.#field_name) {
                        bail!("Assertion failed: value of field '{}' ('{:?}') didn't pass assertion: '{}'", #field_name_string, self.#field_name, #assertion_string)
                    }
                })
            }

            if let Some(ref map) = f.common.map {
                let map_fn = map.parse::<TokenStream>().unwrap();
                field_write_output.extend(quote! {
                    let mapped_value = (#map_fn)(&self.#field_name).with_context(|| format!("Mapping raw value for field '{}'", #field_name_string))?;
                    ParselyWrite::write::<T>(&mapped_value, buf, ()).with_context(|| format!("Writing mapped value for field '{}'", #field_name_string))?;
                });
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

            if let Some(ref after) = f.common.after {
                field_write_output.extend(quote! {
                    #after;
                })
            }

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
        // a) call the sync function provided in the sync_func attribute
        // b) call the sync function on that type with any provided sync_with arguments
        .map(|f| {
            let field_name = f.ident.as_ref().expect("Field has a name");
            let field_name_string = field_name.to_string();
            if let Some(ref sync_func) = f.sync_func {
                quote! {
                    self.#field_name = #sync_func.with_context(|| format!("Syncing field '{}'", #field_name_string))?;
                }
            } else if f.sync_with.is_empty() && f.ty.is_wrapped() {
                // We'll allow this combination to skip a call to sync: for types like Option<T> or
                // Vec<T>, synchronization is only going to make sense if a custom function was
                // provided.
                quote! {}
            } else {
                let sync_with = f.sync_with.expressions();
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
        impl<B: #buffer_type> parsely::ParselyWrite<B, (#(#context_types,)*)> for #struct_name {
            fn write<T: parsely::ByteOrder>(&self, buf: &mut B, ctx: (#(#context_types,)*)) -> parsely::ParselyResult<()> {
                #(#context_assignments)*

                #body

                Ok(())
            }
        }

        impl StateSync<(#(#sync_args_types,)*)> for #struct_name {
            fn sync(&mut self, (#(#sync_args_variables,)*): (#(#sync_args_types,)*)) -> ParselyResult<()> {
                #(#sync_field_calls)*

                Ok(())
            }

        }
    }
}
