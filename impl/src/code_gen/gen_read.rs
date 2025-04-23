use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    get_crate_name, model_types::{wrap_read_with_padding_handling, CollectionLimit, FuncOrClosure, TypedFnArgList}, syn_helpers::TypeExts, ParselyReadData, ParselyReadFieldData
};

pub fn generate_parsely_read_impl(data: ParselyReadData) -> TokenStream {
    let struct_name = data.ident;
    if data.data.is_struct() {
        generate_parsely_read_impl_struct(
            struct_name,
            data.data.take_struct().unwrap(),
            data.buffer_type,
            data.alignment,
            data.required_context,
        )
    } else {
        todo!()
    }
}

fn generate_plain_read(ty: &syn::Type, context_values: &[syn::Expr]) -> TokenStream {
    quote! {
        #ty::read::<T>(buf, (#(#context_values,)*))
    }
}

fn generate_collection_read(
    limit: CollectionLimit,
    ty: &syn::Type,
    context_values: &[syn::Expr],
) -> TokenStream {
    let plain_read = generate_plain_read(ty, context_values);
    match limit {
        CollectionLimit::Count(count) => {
            quote! {
                (0..(#count)).map(|idx| {
                    #plain_read.with_context( || format!("Index {idx}"))
                }).collect::<ParselyResult<Vec<_>>>()
            }
        }
        CollectionLimit::While(pred) => {
            // Since this is multiple statements we wrap it in a closure
            quote! {
                (|| {
                    let mut values: Vec<ParselyResult<#ty>> = Vec::new();
                    let mut idx = 0;
                    while (#pred) {
                        values.push(#plain_read.with_context( || format!("Read {idx}")));
                        idx += 1
                    }
                    values.into_iter().collect::<ParselyResult<Vec<#ty>>>()
                })()
            }
        }
    }
}

fn generate_map_read(field_name: &syn::Ident, map_fn: TokenStream) -> TokenStream {
    let field_name_string = field_name.to_string();
    quote! {
        {
            let original_value = ParselyRead::read::<T>(buf, ()).with_context(|| format!("Reading raw value for field '{}'", #field_name_string))?;
            let mapped_value = (#map_fn)(original_value).with_context(|| format!("Mapping raw value for field '{}'", #field_name_string))?;

            ParselyResult::<_>::Ok(mapped_value)
        }
    }
}

/// Generate an assertion 'block' that can be appended to a [`Result`] type by embedding it in an
/// `and_then` block.  Note that we take a [`syn::Expr`] for the assertion, but it needs to
/// effectively be a function (or a closure) which accepts the value type and returns a boolean.
fn generate_assertion(field_name: &syn::Ident, assertion: &FuncOrClosure) -> TokenStream {
    let assertion_string = quote! { #assertion }.to_string();
    let field_name_string = field_name.to_string();
    quote! {
        .and_then(|actual_value| {
            let assertion_func = #assertion;
            if !assertion_func(&actual_value) {
                bail!("Assertion failed: value of field '{}' ('{:?}') didn't pass assertion: '{}'", #field_name_string, actual_value, #assertion_string)
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
    buffer_type: syn::Ident,
    struct_alignment: Option<usize>,
    required_context: Option<TypedFnArgList>,
) -> TokenStream {
    let crate_name = get_crate_name();
    // Extract out the assignment expressions we'll do to assign the values of the context tuple
    // to the configured variable names, as well as the types of the context tuple.
    let (context_assignments, context_types) = if let Some(ref required_context) = required_context
    {
        (required_context.assignments(), required_context.types())
    } else {
        (Vec::new(), Vec::new())
    };

    // TODO: clean these up to be more like the gen_write code
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
                } else if let Some(ref map) = f.common.map {
                    let map_fn = map.parse::<TokenStream>().unwrap();
                    read_assignment_output.extend(generate_map_read(field_name, map_fn));
                } else if f.ty.is_collection() {
                    let limit = if let Some(ref count) = f.count {
                        CollectionLimit::Count(count.clone())
                    } else if let Some(ref while_pred) = f.while_pred {
                        CollectionLimit::While(while_pred.clone())
                    } else {
                        panic!("Collection field '{field_name}' must have either 'count' or 'while' attribute");
                    };
                    read_assignment_output.extend(generate_collection_read(
                        limit,
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
            let read_assignment = if f.ty.is_option() && f.common.map.is_none() {
                let when_expr = f
                    .when
                    .as_ref()
                    .expect("Optional field '{field_name}' must have a 'when' attribute");
                wrap_in_optional(when_expr, read_assignment)
            } else {
                quote! { #read_assignment }
            };

            let mut output = TokenStream::new();
            output.extend(quote! {
                let #field_name = #read_assignment;
            });

            output = if let Some(alignment) = f.common.alignment {
                wrap_read_with_padding_handling(field_name, alignment, output)
            } else {
                output
            };
            
            if let Some(ref after) = f.common.after {
                output.extend(quote! {
                    #after;
                })
            }
            output
        })
        .collect::<Vec<TokenStream>>();

    let field_names = fields
        .iter()
        .map(|f| f.ident.as_ref().unwrap())
        .collect::<Vec<&syn::Ident>>();

    let body = if let Some(alignment) = struct_alignment {
        quote! {

            let __bytes_remaining_start = buf.remaining_bytes();

            #(#field_reads)*

            let __bytes_remaining_end = buf.remaining_bytes();
            let mut __amount_read = __bytes_remaining_start - __bytes_remaining_end;
            while __amount_read % #alignment != 0 {
                buf.get_u8().context("padding")?;
                __amount_read += 1;
            }
        }
    } else {
        quote! {
            #(#field_reads)*
        }
    };
    quote! {
        impl<B: #buffer_type> ::#crate_name::ParselyRead<B, (#(#context_types,)*)> for #struct_name {
            fn read<T: ::#crate_name::ByteOrder>(buf: &mut B, ctx: (#(#context_types,)*)) -> ::#crate_name::ParselyResult<Self> {
                #(#context_assignments)*

                #body

                Ok(Self { #(#field_names,)* })
            }
        }
    }
}
