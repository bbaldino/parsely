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
    pub use bits_io::nsw_types::*;
}

pub mod anyhow {
    pub use anyhow::*;
}

use code_gen::{gen_read::generate_parsely_read_impl, gen_write::generate_parsely_write_impl};
use darling::{ast, FromDeriveInput, FromField, FromMeta};
use model_types::{Context, ExprOrFunc, FuncOrClosure, TypedFnArgList};
use proc_macro2::TokenStream;
use syn::DeriveInput;
use syn_helpers::TypeExts;

#[doc(hidden)]
pub fn derive_parsely_read(item: TokenStream) -> std::result::Result<TokenStream, syn::Error> {
    let ast: DeriveInput = syn::parse2(item)?;
    let data = ParselyReadData::from_derive_input(&ast)?;
    // eprintln!("parsely_read data = {data:#?}");
    // eprintln!("HELLO, WORLD, item = {ast:#?}");

    Ok(generate_parsely_read_impl(data))
}

#[doc(hidden)]
pub fn derive_parsely_write(item: TokenStream) -> std::result::Result<TokenStream, syn::Error> {
    let ast: DeriveInput = syn::parse2(item)?;
    let data = ParselyWriteData::from_derive_input(&ast)?;
    // eprintln!("parsely_write data = {data:#?}");
    // eprintln!("HELLO, WORLD, item = {ast:#?}");

    Ok(generate_parsely_write_impl(data))
}

#[derive(Debug, FromField, FromMeta)]
pub struct ParselyCommonFieldData {
    // Note: 'magic' fields (ident, ty, etc.) don't work with 'flatten' so can't be held here.
    // See https://github.com/TedDriggs/darling/issues/330

    // generics: Option<syn::Ident>,
    assertion: Option<FuncOrClosure>,

    /// Values that need to be passed as context to this fields read or write method
    context: Option<Context>,

    /// An optional mapping that will be applied to the read value
    map: Option<syn::LitStr>,

    /// An expression which will be evaluated after reading or writing the field
    after: Option<syn::Expr>,

    /// An optional indicator that this field is or needs to be aligned to the given byte alignment
    /// via padding.
    alignment: Option<usize>,
}

#[derive(Debug, FromField)]
#[darling(attributes(parsely, parsely_read))]
pub struct ParselyReadFieldData {
    ident: Option<syn::Ident>,

    ty: syn::Type,

    #[darling(flatten)]
    common: ParselyCommonFieldData,
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

impl ParselyReadFieldData {
    /// Get the 'buffer type' of this field (the type that will be used when reading from or
    /// writing to the buffer): for wrapper types (like [`Option`] or [`Vec`]), this will be the
    /// inner type.
    /// TODO: rename this so it's not confusing with the new buffer_type attribute
    pub(crate) fn buffer_type(&self) -> &syn::Type {
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
        if let Some(ref field_context) = self.common.context {
            field_context.expressions()
        } else {
            &[]
        }
    }
}

#[derive(Debug, FromField)]
#[darling(attributes(parsely, parsely_write))]
pub struct ParselyWriteFieldData {
    ident: Option<syn::Ident>,

    ty: syn::Type,

    #[darling(flatten)]
    common: ParselyCommonFieldData,

    /// An optional function or closure that will be called to synchronize this field based on some
    /// external data
    sync_func: Option<ExprOrFunc>,

    /// An list of expressions that should be passed as context to this field's sync method.  The
    /// sync method provides an opportunity to synchronize "linked" fields, where one field's value
    /// depends on the value of another.
    #[darling(default)]
    sync_with: Context,
}

impl ParselyWriteFieldData {
    /// Get the 'buffer type' of this field (the type that will be used when reading from or
    /// writing to the buffer): for wrapper types (like [`Option`] or [`Vec`]), this will be the
    /// inner type.
    pub(crate) fn buffer_type(&self) -> &syn::Type {
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
        if let Some(ref field_context) = self.common.context {
            field_context.expressions()
        } else {
            &[]
        }
    }
}

fn default_read_buffer_type_ident() -> syn::Ident {
    syn::Ident::new("BitBuf", proc_macro2::Span::call_site())
}

fn default_write_buffer_type_ident() -> syn::Ident {
    syn::Ident::new("BitBufMut", proc_macro2::Span::call_site())
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(parsely, parsely_read), supports(struct_any, enum_any))]
pub struct ParselyReadData {
    ident: syn::Ident,
    required_context: Option<TypedFnArgList>,
    #[darling(default = default_read_buffer_type_ident)]
    buffer_type: syn::Ident,
    alignment: Option<usize>,
    data: ast::Data<(), ParselyReadFieldData>,
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(parsely, parsely_write), supports(struct_any, enum_any))]
pub struct ParselyWriteData {
    ident: syn::Ident,
    required_context: Option<TypedFnArgList>,
    sync_args: Option<TypedFnArgList>,
    #[darling(default = default_write_buffer_type_ident)]
    buffer_type: syn::Ident,
    alignment: Option<usize>,
    data: ast::Data<(), ParselyWriteFieldData>,
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
