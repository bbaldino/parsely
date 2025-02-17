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
            let field_name = f.ident.as_ref().expect("Field has a name");
            let field_name_str = field_name.to_string();
            let read_type = if f.ty.is_option() || f.ty.is_collection() {
                f.ty.inner_type().expect("Option or collection has an inner type")
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
                if f.ty.is_collection() {
                    let count_expr = f.count.as_ref().expect("Collection field '{field_name}' must have a 'count' attribute"); 
                    read_assignment_output.extend(quote! {
                        (0..(#count_expr)).map(|idx| {
                            #read_type::read::<T, B>(buf, (#(#context_values,)*)).with_context(|| format!("Index {idx}")) 
                        }).collect::<ParselyResult<Vec<_>>>()
                    });
                } else {
                    read_assignment_output.extend(quote! {
                        #read_type::read::<T, B>(buf, (#(#context_values,)*))
                    });
                }
                if let Some(ref assertion) = f.assertion {
                    let assertion_string = quote! { #assertion }.to_string();
                    // Note: assign the value of the assertion expression to a variable to make it
                    // calleable.
                    read_assignment_output.extend(quote! {
                        .and_then(|actual_value| {
                            let assertion_func = #assertion;
                            if !assertion_func(actual_value) {
                                bail!("Assertion failed: value of field '{}' ('{}') didn't pass assertion: '{}'", #field_name_str, actual_value, #assertion_string)
                            }
                            Ok(actual_value)
                        })
                    })
                }
                let error_context = format!("Reading field '{field_name}'");
                read_assignment_output.extend(quote! { .with_context(|| #error_context)?});
                read_assignment_output
            };

            let read_assignment = if f.ty.is_option() {
                let when_expr = f.when.as_ref().expect("Optional field '{field_name}' must have a 'when' attribute");
                quote! {
                    if #when_expr {
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
