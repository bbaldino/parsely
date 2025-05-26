use super::parsely_read_field_data::ParselyReadFieldData;

pub struct ParselyReadVariantData {
    ident: syn::Ident,
    discriminant: Option<syn::Expr>,
    fields: Vec<ParselyReadFieldData>,
}
