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
                if let Some(ref assign_from) = f.assign_from {
                    // Because of this 'naked' Ok, compiler can complain about not being able to
                    // infer the proper error type when we apply '?' to this statement later, so
                    // fully-qualify it as a ParselyResult::<_>::Ok
                    read_assignment_output.extend(quote!{
                        ParselyResult::<_>::Ok(#assign_from)
                    })
                } else if  let Some(ref map) = f.map {
                    // When doing a mapping, we need to do multiple operations:
                    // 1. The initial read
                    // 2. The mapping of the read value
                    // So we encapsulate those inside of a block here to make it look like one
                    // operations to be consistent with the other operations.
                    let ts = map.parse::<TokenStream>().unwrap();
                    read_assignment_output.extend(quote! {
                        {
                            let #field_name = ParselyRead::read::<T, B>(buf, ()).with_context(|| format!("Reading raw value for field '{}'", #field_name_str))?;
                            let #field_name = (#ts)(#field_name).with_context(|| format!("Mapping raw value for field '{}'", #field_name_str))?;

                            ParselyResult::<_>::Ok(#field_name)
                        }
                    })
                } else if f.ty.is_collection() {
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
