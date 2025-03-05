mod code_gen;
pub mod error;
mod model_types;
pub mod parsely_read;
mod syn_helpers;

pub use bit_cursor::{
    bit_cursor::BitCursor, bit_read::BitRead, byte_order::BigEndian, byte_order::ByteOrder,
    byte_order::LittleEndian, byte_order::NetworkOrder,
};
pub mod nsw_types {
    pub use bit_cursor::nsw_types::*;
}

pub mod anyhow {
    pub use anyhow::*;
}

use code_gen::generate_parsely_read_impl;
use darling::{ast, FromDeriveInput, FromField};
use model_types::{Assertion, Context, RequiredContext};
use proc_macro2::TokenStream;
use syn::DeriveInput;
use syn_helpers::TypeExts;

#[doc(hidden)]
pub fn derive_parsely_read(item: TokenStream) -> std::result::Result<TokenStream, syn::Error> {
    let ast: DeriveInput = syn::parse2(item)?;
    let data = ParselyData::from_derive_input(&ast)?;
    // eprintln!("HELLO, WORLD, data = {data:#?}");
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

    // generics: Option<syn::Ident>,
    assertion: Option<Assertion>,
    context: Option<Context>,

    /// 'when' is required when there's an optional field
    when: Option<syn::Expr>,

    /// 'count' is required when the field is a collection
    count: Option<syn::Expr>,

    /// Instead of reading the value of this field from the buffer, assign it from the given
    /// [`syn::Ident`]
    assign_from: Option<syn::Ident>,

    /// An optional mapping that will be applied to the read value
    map: Option<syn::LitStr>,

    /// An optional custom reader function.  This function must have the same basic read signature
    /// as [`ParselyRead::read`].
    reader: Option<syn::Ident>,
}

impl ParselyFieldData {
    /// Get the 'read type' of this field: for wrapper types (like [`Option`] or [`Vec`]), this will
    /// be the inner type.
    pub(crate) fn read_type(&self) -> &syn::Type {
        if self.ty.is_option() || self.ty.is_collection() {
            self.ty
                .inner_type()
                .expect("Option or collection has an inner type")
        } else {
            &self.ty
        }
    }

    /// Get the context values that need to be passed to the read or write call for this field
    pub(crate) fn context_values(&self) -> &[syn::Expr] {
        if let Some(ref field_context) = self.context {
            field_context.expressions()
        } else {
            &[]
        }
    }
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(parsely), supports(struct_any, enum_any))]
pub struct ParselyData {
    ident: syn::Ident,
    required_context: Option<RequiredContext>,
    data: ast::Data<(), ParselyFieldData>,
}
