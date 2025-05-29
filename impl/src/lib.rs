mod code_gen;
pub mod error;
mod model_types;
pub mod parsely_read;
pub mod parsely_write;
mod syn_helpers;

pub use bits_io::{
    buf::bit_buf::BitBuf,
    buf::bit_buf_exts::BitBufExts,
    buf::bit_buf_mut::BitBufMut,
    buf::bit_buf_mut_exts::BitBufMutExts,
    buf::bits::Bits,
    buf::bits_mut::BitsMut,
    buf::byte_order::{BigEndian, ByteOrder, LittleEndian, NetworkOrder},
    io::{bit_cursor::BitCursor, bit_read::BitRead, bit_write::BitWrite},
};

pub mod nsw_types {
    pub use bits_io::nsw_types::from_bitslice::BitSliceUxExts;
    pub use bits_io::nsw_types::*;
}

pub mod anyhow {
    pub use anyhow::*;
}

use code_gen::{
    read::{
        parsely_read_enum_data::ParselyReadEnumData,
        parsely_read_struct_data::ParselyReadStructData,
    },
    write::parsely_write_struct_data::ParselyWriteStructData,
};
use darling::{ast, FromDeriveInput, FromField, FromMeta, FromVariant};
use model_types::{Assertion, Context, ExprOrFunc, MapExpr, TypedFnArgList};
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;
use syn_helpers::TypeExts;

#[doc(hidden)]
pub fn derive_parsely_read(item: TokenStream) -> std::result::Result<TokenStream, syn::Error> {
    let ast: DeriveInput = syn::parse2(item)?;
    let data = ParselyReadReceiver::from_derive_input(&ast)?;

    // println!("{data:#?}");

    if data.data.is_struct() {
        let struct_data = ParselyReadStructData::try_from(data).unwrap();
        Ok(quote! {
            #struct_data
        })
    } else {
        let enum_data = ParselyReadEnumData::try_from(data).unwrap();
        Ok(quote! {
            #enum_data
        })
    }
}

#[doc(hidden)]
pub fn derive_parsely_write(item: TokenStream) -> std::result::Result<TokenStream, syn::Error> {
    let ast: DeriveInput = syn::parse2(item)?;
    let data = ParselyWriteReceiver::from_derive_input(&ast)?;

    if data.data.is_struct() {
        let struct_data = ParselyWriteStructData::try_from(data).unwrap();
        Ok(quote! {
            #struct_data
        })
    } else {
        todo!()
    }

    // Ok(generate_parsely_write_impl(data))
}

#[derive(Debug, FromField, FromMeta)]
pub struct ParselyCommonFieldReceiver {
    // Note: 'magic' fields (ident, ty, etc.) don't work with 'flatten' so can't be held here.
    // See https://github.com/TedDriggs/darling/issues/330

    // generics: Option<syn::Ident>,
    assertion: Option<Assertion>,

    /// Values that need to be passed as context to this fields read or write method
    context: Option<Context>,

    /// An optional mapping that will be applied to the read value
    map: Option<MapExpr>,

    /// An optional indicator that this field is or needs to be aligned to the given byte alignment
    /// via padding.
    alignment: Option<usize>,
}

#[derive(Debug, FromField)]
#[darling(attributes(parsely, parsely_read))]
pub struct ParselyReadFieldReceiver {
    ident: Option<syn::Ident>,

    ty: syn::Type,

    #[darling(flatten)]
    common: ParselyCommonFieldReceiver,
    /// 'count' is required when the field is a collection
    count: Option<syn::Expr>,
    /// 'while' is an alternate option to 'count' to use with a collection field
    // #[darling(rename = "while")]
    // TODO: hopefully can get this to work as 'while'
    while_pred: Option<syn::Expr>,

    /// Instead of reading the value of this field from the buffer, assign it from the given
    /// [`syn::Ident`]
    assign_from: Option<syn::Expr>,

    /// 'when' is required when there's an optional field
    when: Option<syn::Expr>,
}

#[derive(Debug, FromVariant)]
#[darling(attributes(parsely, parsely_read))]
pub struct ParselyReadVariantReceiver {
    ident: syn::Ident,
    discriminant: Option<syn::Expr>,
    id: syn::Expr,
    fields: ast::Fields<ParselyReadFieldReceiver>,
}

#[derive(Debug, FromField)]
#[darling(attributes(parsely, parsely_write))]
pub struct ParselyWriteFieldReceiver {
    ident: Option<syn::Ident>,

    ty: syn::Type,

    #[darling(flatten)]
    common: ParselyCommonFieldReceiver,

    /// An expression or function call that will be used to update this field in the generated
    /// `StateSync` implementation for its parent type.
    sync_expr: Option<ExprOrFunc>,

    /// An list of expressions that should be passed as context to this field's sync method.  The
    /// sync method provides an opportunity to synchronize "linked" fields, where one field's value
    /// depends on the value of another.
    #[darling(default)]
    sync_with: Context,
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(parsely, parsely_read), supports(struct_any, enum_any))]
pub struct ParselyReadReceiver {
    ident: syn::Ident,
    required_context: Option<TypedFnArgList>,
    alignment: Option<usize>,
    // Enums require a value to match on to determine which variant should be parsed
    key: Option<syn::Expr>,
    data: ast::Data<ParselyReadVariantReceiver, ParselyReadFieldReceiver>,
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(parsely, parsely_write), supports(struct_any, enum_any))]
pub struct ParselyWriteReceiver {
    ident: syn::Ident,
    required_context: Option<TypedFnArgList>,
    sync_args: Option<TypedFnArgList>,
    alignment: Option<usize>,
    data: ast::Data<(), ParselyWriteFieldReceiver>,
}

pub(crate) fn get_crate_name() -> syn::Ident {
    let found_crate =
        proc_macro_crate::crate_name("parsely-rs").expect("parsely-rs is present in Cargo.toml");

    let crate_name = match found_crate {
        proc_macro_crate::FoundCrate::Itself => "parsely-rs".to_string(),
        proc_macro_crate::FoundCrate::Name(name) => name,
    };

    syn::Ident::new(&crate_name, proc_macro2::Span::call_site())
}
