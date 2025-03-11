use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    model_types::RequiredContext, syn_helpers::TypeExts, ParselyWriteData, ParselyWriteFieldData,
};

pub fn generate_parsely_write_impl(data: ParselyWriteData) -> TokenStream {
    let struct_name = data.ident;
    if data.data.is_struct() {
        generate_parsely_write_impl_struct(
            struct_name,
            data.data.take_struct().unwrap(),
            data.required_context,
        )
    } else {
        todo!()
    }
}

fn generate_parsely_write_impl_struct(
    struct_name: syn::Ident,
    fields: darling::ast::Fields<ParselyWriteFieldData>,
    required_context: Option<RequiredContext>,
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
                    if !assertion_func(self.#field_name) {
                        bail!("Assertion failed: value of field '{}' ('{}') didn't pass assertion: '{}'", #field_name_string, self.#field_name, #assertion_string)
                    }
                })
            }

            if let Some(ref writer) = f.writer {
                field_write_output.extend(quote! {
                    #writer::<T, B>(&self.#field_name, buf, (#(#context_values,)*)).with_context(|| format!("Writing field '{}'", #field_name_string))?;
                });
            } else if f.ty.is_option() {
                field_write_output.extend(quote! {
                    if let Some(ref v) = self.#field_name {
                        #write_type::write::<T, B>(v, buf, (#(#context_values,)*)).with_context(|| format!("Writing field '{}'", #field_name_string))?;
                    }
                });
            } else if f.ty.is_collection() {
                field_write_output.extend(quote! {
                    self.#field_name.iter().enumerate().map(|(idx, v)| {
                        #write_type::write::<T, B>(v, buf, (#(#context_values,)*)).with_context(|| format!("Index {idx}"))
                    }).collect::<ParselyResult<Vec<_>>>().with_context(|| format!("Writing field '{}'", #field_name_string))?;
                });
            } else {
                field_write_output.extend(quote! {
                    #write_type::write::<T, B>(&self.#field_name, buf, (#(#context_values,)*)).with_context(|| format!("Writing field '{}'", #field_name_string))?;
                });
            }

            field_write_output
        })
        .collect::<Vec<TokenStream>>();

    quote! {
        impl parsely::ParselyWrite<(#(#context_types,)*)> for #struct_name {
            fn write<T: parsely::ByteOrder, B: parsely::BitWrite>(&self, buf: &mut B, ctx: (#(#context_types,)*)) -> parsely::ParselyResult<()> {
                #(#context_assignments)*

                #(#field_writes)*

                Ok(())
            }
        }
    }
}
