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

            if let Some(ref writer) = f.writer {
                field_write_output.extend(quote! {
                    #writer::<T>(&self.#field_name, buf, (#(#context_values,)*)).with_context(|| format!("Writing field '{}'", #field_name_string))?;
                });
            } else if let Some(ref map) = f.common.map {
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
        // Filter for any field that has a 'sync_func' or a 'sync_with' attribute:
        // 'sync_func' attributes mean this field needs to be updated based on some data that was
        // passed in to the sync method
        // 'sync_with' attirbutes mean this field's 'sync' method needs to be called with some data
        // TODO: I'm pretty sure we need to _always_ call the type's sync method, but built-in
        // types don't currently have a sync method defined, so that'll fail in some cases.  I
        // think sync will need to become a trait instead and we'll need to generate impls for all
        // ParselyWrite types.  Or maybe the sync method should be part of the ParselyWrite trait?
        .filter(|f| f.sync_func.is_some() || f.sync_with.is_some())
        .map(|f| {
            let field_name = f.ident.as_ref().expect("Field has a name");
            let field_name_string = field_name.to_string();
            // Note: I think these two fields should be mutually exclusive, but will see how it
            // goes
            if let Some(ref sync_func) = f.sync_func {
                quote! {
                    self.#field_name = #sync_func.with_context(|| format!("Syncing field '{}'", #field_name_string))?;
                }
            } else if let Some(ref sync_with) = f.sync_with {
                let sync_with = sync_with.expressions();
                quote! {
                    self.#field_name.sync((#(#sync_with,)*)).with_context(|| format!("Syncing field '{}'", #field_name_string))?;
                }
            } else {
                unreachable!()
            }
        })
        .collect::<Vec<TokenStream>>();

    quote! {
        impl<B: #buffer_type> parsely::ParselyWrite<B, (#(#context_types,)*)> for #struct_name {
            fn write<T: parsely::ByteOrder>(&self, buf: &mut B, ctx: (#(#context_types,)*)) -> parsely::ParselyResult<()> {
                #(#context_assignments)*

                #(#field_writes)*

                Ok(())
            }
        }

        impl #struct_name {
            pub fn sync(&mut self, (#(#sync_args_variables,)*): (#(#sync_args_types,)*)) -> ParselyResult<()> {
                #(#sync_field_calls)*

                Ok(())
            }
        }
    }
}
