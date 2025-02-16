use proc_macro2::TokenStream;
use quote::quote;

use crate::{syn_helpers::TypeExts, ParselyData};

pub fn generate_parsely_read_impl(data: ParselyData) -> TokenStream {
    let struct_name = data.ident;
    let data_struct = data.data.take_struct().unwrap();
    eprintln!("Fields type: {:?}", data_struct.style);

    let (context_assignments, context_types) =
        if let Some(ref required_context) = data.required_context {
            (required_context.assignments(), required_context.types())
        } else {
            (Vec::new(), Vec::new())
        };

    let field_reads = data_struct
        .fields
        .iter()
        .map(|f| {
            let field_name = f.ident.as_ref().unwrap();
            let read_type = if f.ty.is_option() {
                f.ty.option_inner_type().unwrap()
            } else {
                &f.ty
            };

            // Context values that we need to pass to this field's ParselyRead::read method
            let context_values = if let Some(ref field_context) = f.context {
                field_context.expressions()
            } else {
                Vec::new()
            };

            let read_assignment = {
                let mut read_assignment_output = TokenStream::new();
                read_assignment_output.extend(quote!{#read_type::read::<T, B>(buf, (#(#context_values,)*))});
                if let Some(ref fixed_value) = f.fixed {
                    // Note: evaluate '#fixed_value' since it's an expression and we don't want to
                    // evaluate it twice (one for the check and again in the error case)
                    read_assignment_output.extend(quote! {
                        .and_then(|actual_value| {
                            let expected_value = #fixed_value;
                            if actual_value != expected_value {
                                bail!("Required to have fixed value '{}', but instead had '{}'",  expected_value, actual_value)
                            } else {
                                Ok(actual_value)
                            }
                        })
                    })
                }
                let error_context = format!("Reading field '{field_name}'");
                read_assignment_output.extend(quote! { .with_context(|| #error_context)?});
                read_assignment_output
            };

            let read_assignment = if f.ty.is_option() {
                let when_clause = f.when.as_ref().expect("Optional field '{field_name}' must have a 'when' attribute");
                quote!{
                    if #when_clause {
                        Some(#read_assignment)
                    } else {
                        None
                    }
                }
            } else {
                quote!{ #read_assignment}
            };

            quote!{
                let #field_name = { #read_assignment };
            }

        })
        .collect::<Vec<TokenStream>>();
    let field_names = data_struct
        .fields
        .iter()
        .map(|f| f.ident.as_ref().unwrap())
        .collect::<Vec<&syn::Ident>>();
    quote! {
        impl parsely::ParselyRead<(#(#context_types,)*)> for #struct_name {
            fn read<T: parsely::ByteOrder, B: parsely::BitRead>(buf: &mut B, ctx: (#(#context_types,)*)) -> parsely::ParselyResult<Self> {
                #(#context_assignments)*

                #(#field_reads)*

                Ok(Self { #(#field_names,)* })
            }
        }
    }
}
