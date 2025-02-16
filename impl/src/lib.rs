mod code_gen;
pub mod error;
mod model_types;
pub mod parsely_read;

pub use bit_cursor::{
    bit_read::BitRead, byte_order::BigEndian, byte_order::ByteOrder, byte_order::LittleEndian,
    byte_order::NetworkOrder,
};
pub mod nsw_types {
    pub use bit_cursor::nsw_types::*;
}

pub mod anyhow {
    pub use anyhow::*;
}

use code_gen::generate_parsely_read_impl;
use darling::{ast, FromDeriveInput, FromField, FromMeta};
use model_types::TypedFnArg;
use proc_macro2::TokenStream;
use syn::DeriveInput;

#[doc(hidden)]
pub fn derive_parsely_read(item: TokenStream) -> std::result::Result<TokenStream, syn::Error> {
    let ast: DeriveInput = syn::parse2(item)?;
    let data = ParselyData::from_derive_input(&ast)?;
    eprintln!("HELLO, WORLD, data = {data:#?}");
    // eprintln!("HELLO, WORLD, item = {ast:#?}");

    Ok(generate_parsely_read_impl(data))
}

#[derive(Debug, FromField)]
#[darling(attributes(parsely))]
pub struct ParselyFieldData {
    /// Get the ident of the field. For fields in tuple or newtype structs or
    /// enum bodies, this can be `None`.
    ident: Option<syn::Ident>,

    /// This magic field name pulls the type from the input.
    ty: syn::Type,

    fixed: Option<syn::Expr>,
}

#[derive(Debug)]
struct RequiredContext(pub(crate) Vec<TypedFnArg>);

impl RequiredContext {
    pub(crate) fn types(&self) -> Vec<&syn::Type> {
        self.0.iter().map(|t| t.ty()).collect()
    }
}

impl FromMeta for RequiredContext {
    fn from_none() -> Option<Self> {
        None
    }

    fn from_list(items: &[ast::NestedMeta]) -> darling::Result<Self> {
        let required_context: Vec<TypedFnArg> = items
            .iter()
            .map(|item| {
                match item {
                    // TODO: better error message here
                    ast::NestedMeta::Meta(_) => Err(darling::Error::unsupported_format(
                        "FnArg literals required",
                    )),
                    ast::NestedMeta::Lit(lit) => match lit {
                        syn::Lit::Str(s) => s.parse().map_err(|e| e.into()),
                        l => Err(darling::Error::unexpected_lit_type(l)),
                    },
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self(required_context))
    }
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(parsely), supports(struct_any, enum_any))]
pub struct ParselyData {
    ident: syn::Ident,
    required_context: Option<RequiredContext>,
    data: ast::Data<(), ParselyFieldData>,
}
