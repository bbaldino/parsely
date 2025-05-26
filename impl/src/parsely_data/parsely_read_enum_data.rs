use crate::TypedFnArgList;

use super::parsely_read_variant_data::ParselyReadVariantData;

/// A struct which represents all information needed for generating a `ParselyRead` implementation
/// for a given struct.
pub struct ParselyReadEnumData {
    ident: syn::Ident,
    required_context: Option<TypedFnArgList>,
    alignment: Option<usize>,
    variants: Vec<ParselyReadVariantData>,
}
