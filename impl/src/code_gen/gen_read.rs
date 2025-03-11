use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    model_types::{Assertion, RequiredContext},
    syn_helpers::TypeExts,
    ParselyReadData, ParselyReadFieldData,
};

pub fn generate_parsely_read_impl(data: ParselyReadData) -> TokenStream {
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

fn generate_plain_read(ty: &syn::Type, context_values: &[syn::Expr]) -> TokenStream {
    quote! {
        #ty::read::<T, B>(buf, (#(#context_values,)*))
    }
}

fn generate_collection_read(
    count_expr: &syn::Expr,
    ty: &syn::Type,
    context_values: &[syn::Expr],
) -> TokenStream {
    let plain_read = generate_plain_read(ty, context_values);
    quote! {
        (0..(#count_expr)).map(|idx| {
            #plain_read.with_context( || format!("Index {idx}"))
        }).collect::<ParselyResult<Vec<_>>>()
    }
}

fn generate_map_read(field_name: &syn::Ident, map_fn: TokenStream) -> TokenStream {
    let field_name_string = field_name.to_string();
    quote! {
        {
            let original_value = ParselyRead::read::<T, B>(buf, ()).with_context(|| format!("Reading raw value for field '{}'", #field_name_string))?;
            let mapped_value = (#map_fn)(original_value).with_context(|| format!("Mapping raw value for field '{}'", #field_name_string))?;

            ParselyResult::<_>::Ok(mapped_value)
        }
    }
}

/// Generate an assertion 'block' that can be appended to a [`Result`] type by embedding it in an
/// `and_then` block.  Note that we take a [`syn::Expr`] for the assertion, but it needs to
/// effectively be a function (or a closure) which accepts the value type and returns a boolean.
fn generate_assertion(field_name: &syn::Ident, assertion: &Assertion) -> TokenStream {
    let assertion_string = quote! { #assertion }.to_string();
    let field_name_string = field_name.to_string();
    quote! {
        .and_then(|actual_value| {
            let assertion_func = #assertion;
            if !assertion_func(actual_value) {
                bail!("Assertion failed: value of field '{}' ('{}') didn't pass assertion: '{}'", #field_name_string, actual_value, #assertion_string)
            }
            Ok(actual_value)
        })
    }
}

fn wrap_in_optional(when_expr: &syn::Expr, inner: TokenStream) -> TokenStream {
    quote! {
        if #when_expr {
            Some(#inner)
        } else {
            None
        }
    }
}

fn generate_parsely_read_impl_struct(
    struct_name: syn::Ident,
    fields: darling::ast::Fields<ParselyReadFieldData>,
    required_context: Option<RequiredContext>,
) -> TokenStream {
    // Extract out the assignment expressions we'll do to assign the values of the context tuple
    // to the configured variable names, as well as the types of the context tuple.
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
            let read_type = f.buffer_type();

            // Context values that we need to pass to this field's ParselyRead::read method
            let context_values = f.context_values();

            let read_assignment = {
                let mut read_assignment_output = TokenStream::new();
                if let Some(ref assign_from) = f.assign_from {
                    // Because of this 'naked' Ok, compiler can complain about not being able to
                    // infer the proper error type when we apply '?' to this statement later, so
                    // fully-qualify it as a ParselyResult::<_>::Ok
                    read_assignment_output.extend(quote! {
                        ParselyResult::<_>::Ok(#assign_from)
                    })
                } else if let Some(ref reader) = f.reader {
                    read_assignment_output.extend(quote! {
                        #reader::<T, B>(buf, (#(#context_values,)*))
                    })
                } else if let Some(ref map) = f.common.map {
                    let map_fn = map.parse::<TokenStream>().unwrap();
                    read_assignment_output.extend(generate_map_read(field_name, map_fn));
                } else if f.ty.is_collection() {
                    let count_expr = f
                        .count
                        .as_ref()
                        .expect("Collection field '{field_name}' must have a 'count' attribute");
                    read_assignment_output.extend(generate_collection_read(
                        count_expr,
                        read_type,
                        context_values,
                    ));
                } else {
                    read_assignment_output.extend(generate_plain_read(read_type, context_values));
                }
                if let Some(ref assertion) = f.common.assertion {
                    read_assignment_output.extend(generate_assertion(field_name, assertion));
                }
                let error_context = format!("Reading field '{field_name}'");
                read_assignment_output.extend(quote! { .with_context(|| #error_context)?});
                read_assignment_output
            };

            // TODO: what cases should we allow to bypass a 'when' clause for an Option?
            let read_assignment =
                if f.ty.is_option() && f.common.map.is_none() && f.reader.is_none() {
                    let when_expr = f
                        .when
                        .as_ref()
                        .expect("Optional field '{field_name}' must have a 'when' attribute");
                    wrap_in_optional(when_expr, read_assignment)
                } else {
                    quote! { #read_assignment }
                };

            quote! {
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
