use proc_macro2::TokenStream;
use quote::quote;

use crate::{model_types::RequiredContext, syn_helpers::TypeExts, ParselyData, ParselyFieldData};

pub fn generate_parsely_read_impl(data: ParselyData) -> TokenStream {
    let struct_name = data.ident;
    if data.data.is_struct() {
        generate_parsely_read_impl_struct(
            struct_name,
            data.data.take_struct().unwrap(),
            data.required_context,
        )
    } else {
        todo!()
    }
}

fn generate_parsely_read_impl_struct(
    struct_name: syn::Ident,
    fields: darling::ast::Fields<ParselyFieldData>,
    required_context: Option<RequiredContext>,
) -> TokenStream {
    let (context_assignments, context_types) = if let Some(ref required_context) = required_context
    {
        (required_context.assignments(), required_context.types())
    } else {
        (Vec::new(), Vec::new())
    };

    let field_reads = fields
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
                // TODO: separated these branches because wrapping 'assign_from' in 'Ok' didn't
                // seem to play nicely with with_context when I first tried it.  I suspect that
                // should be able to work though, so it might be nice to make these paths
                // consistent again, even if the with_context is unnecessary for the 'assign_from'
                // case.
                if let Some(ref assign_from) = f.assign_from {
                    read_assignment_output.extend(quote!{
                        #assign_from
                    })
                } else {
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
                }
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
                quote!{ #read_assignment }
            };

            quote!{
                let #field_name = #read_assignment;
            }

        })
        .collect::<Vec<TokenStream>>();
    let field_names = fields
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
