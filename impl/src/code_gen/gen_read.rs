use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    get_crate_name,
    model_types::{
        wrap_read_with_padding_handling, CollectionLimit, FuncOrClosure, TypedFnArgList,
    },
    syn_helpers::TypeExts,
    ParselyReadData, ParselyReadFieldData,
};

pub fn generate_parsely_read_impl(data: ParselyReadData) -> TokenStream {
    let struct_name = data.ident;
    if data.data.is_struct() {
        generate_parsely_read_impl_struct(
            struct_name,
            data.data.take_struct().unwrap(),
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

fn generate_map_read(field_name: &syn::Ident, map_fn: &FuncOrClosure) -> TokenStream {
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

/// Given the data associated with a field, generate the code for properly reading it from a
/// buffer.
///
/// The attributes set in the [`ParselyReadFieldData`] all shape the logic necessary in order to
/// properly parse this field.  Roughly, the processing is as follows:
///
/// 1. Check if an 'assign_from' attribute is set.  If so, we don't read from the buffer at all and
///    instead just assign the field to the result of the given expression.
/// 2. Check if a 'map' attribute is set.  If so, we'll read a value as a different type and then
///    pass it t othe map function to arrive at the final type and assign it to the field.
/// 3. Check if the field is a collection.  If so, some kind of accompanying 'limit' attribute is
///    required: either a 'count' attribute or a `while_pred` attribute that defines how many
///    elements should be read.
/// 4. If none of the above are the case, do a 'plain' read where we just read the type directly
///    from the buffer.
/// 5. If an 'assertion' attribute is present, then generate code to assert on the read value using
///    the given assertion function or closure.
/// 6. After the code to perform the read has been generated, we check if the field is an option
///    type.  If so, a 'when' attribute is required.  This is an expression that determines when the
///    read should actually be done.
/// 7. Finally, if an 'alignment' attribute is present, code is added to detect and consume any
///    padding after the read.
fn generate_field_read(field_data: &ParselyReadFieldData) -> TokenStream {
    let field_name = field_data
        .ident
        .as_ref()
        .expect("Only named fields supported");
    let read_type = field_data.buffer_type();
    // Context values that we need to pass to this field's ParselyRead::read method
    let context_values = field_data.context_values();
    let mut output = TokenStream::new();

    if let Some(ref assign_expr) = field_data.assign_from {
        output.extend(quote! {
            ParselyResult::<_>::Ok(#assign_expr)
        })
    } else if let Some(ref map_fn) = field_data.common.map {
        output.extend(generate_map_read(field_name, map_fn));
    } else if field_data.ty.is_collection() {
        let limit = if let Some(ref count) = field_data.count {
            CollectionLimit::Count(count.clone())
        } else if let Some(ref while_pred) = field_data.while_pred {
            CollectionLimit::While(while_pred.clone())
        } else {
            panic!("Collection field '{field_name}' must have either 'count' or 'while' attribute");
        };
        output.extend(generate_collection_read(limit, read_type, &context_values));
    } else {
        output.extend(generate_plain_read(read_type, &context_values));
    }

    // println!("tokenstream: {}", output);

    if let Some(ref assertion) = field_data.common.assertion {
        output.extend(generate_assertion(field_name, assertion));
    }
    let error_context = format!("Reading field '{field_name}'");
    output.extend(quote! { .with_context(|| #error_context)?});

    // TODO: what cases should we allow to bypass a 'when' clause for an Option?
    output = if field_data.ty.is_option() && field_data.common.map.is_none() {
        let when_expr = field_data
            .when
            .as_ref()
            .expect("Optional field '{field_name}' must have a 'when' attribute");
        wrap_in_optional(when_expr, output)
    } else {
        output
    };

    output = if let Some(alignment) = field_data.common.alignment {
        wrap_read_with_padding_handling(field_name, alignment, output)
    } else {
        output
    };

    // println!("token stream before assignment: {output}");

    quote! {
        let #field_name = #output;
    }
}

fn generate_parsely_read_impl_struct(
    struct_name: syn::Ident,
    fields: darling::ast::Fields<ParselyReadFieldData>,
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

    let field_reads = fields
        .iter()
        .map(generate_field_read)
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

    let ctx_var = if context_types.is_empty() {
        format_ident!("_ctx")
    } else {
        format_ident!("ctx")
    };

    quote! {
        impl<B: BitBuf> ::#crate_name::ParselyRead<B, (#(#context_types,)*)> for #struct_name {
            fn read<T: ::#crate_name::ByteOrder>(buf: &mut B, #ctx_var: (#(#context_types,)*)) -> ::#crate_name::ParselyResult<Self> {
                #(#context_assignments)*

                #body

                Ok(Self { #(#field_names,)* })
            }
        }
    }
}
