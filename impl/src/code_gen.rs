use proc_macro2::TokenStream;
use quote::quote;

use crate::ParselyData;

pub fn generate_parsely_read_impl(data: ParselyData) -> TokenStream {
    let struct_name = data.ident;
    let data_struct = data.data.take_struct().unwrap();
    eprintln!("Fields type: {:?}", data_struct.style);
    let field_reads = data_struct
        .fields
        .iter()
        .map(|f| {
            let field_name = f.ident.as_ref().unwrap();
            let ty = &f.ty;
            let error_context = format!("Reading field '{field_name}'");

            let mut output = quote! {
                let #field_name = #ty::read::<T, B>(buf, ())
            };
            if let Some(ref fixed_value) = f.fixed {
                // Note: evaluate '#fixed_value' since it's an expression and we don't want to
                // evaluate it twice (one for the check and again in the error case)
                output.extend(quote! {
                    .and_then(|v| {
                        let expected_value = #fixed_value;
                        if v != expected_value {
                            bail!("Required to have fixed value '{}', but instead had '{}'", v, expected_value)
                        } else {
                            Ok(v)
                        }
                    })
                })
            }
            output.extend(quote! { .context(#error_context)?;});
            output
        })
        .collect::<Vec<TokenStream>>();
    let field_names = data_struct
        .fields
        .iter()
        .map(|f| f.ident.as_ref().unwrap())
        .collect::<Vec<&syn::Ident>>();
    quote! {
        impl parsely::ParselyRead<()> for #struct_name {
            fn read<T: parsely::ByteOrder, B: parsely::BitRead>(buf: &mut B, _ctx: ()) -> parsely::ParselyResult<Self> {
                #(#field_reads)*

                Ok(Self { #(#field_names,)* })
            }
        }
    }
}
