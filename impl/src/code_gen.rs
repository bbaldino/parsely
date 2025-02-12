use proc_macro2::TokenStream;
use quote::quote;

use crate::ParselyData;

pub fn generate_parsely_read_impl(data: ParselyData) -> TokenStream {
    let struct_name = data.ident;
    let byte_order = data.byte_order.unwrap();
    let data_struct = data.data.take_struct().unwrap();
    eprintln!("Fields type: {:?}", data_struct.style);
    let field_reads = data_struct
        .fields
        .iter()
        .map(|f| {
            let field_name = f.ident.as_ref().unwrap();
            let ty = &f.ty;
            let error_context = format!("{field_name}");

            quote! {
                let #field_name = #ty::read::<T, B>(buf, ()).context(#error_context)?;
            }
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
