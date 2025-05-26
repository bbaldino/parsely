use crate::{model_types::MemberIdent, Assertion, Context, MapExpr};

/// Items that are common across `ParselyRead` and `ParselyWrite` fields
pub struct ParselyCommonFieldData {
    pub ident: MemberIdent,
    /// The field's type
    pub ty: syn::Type,

    pub assertion: Option<Assertion>,
    /// Values that need to be passed as context to this fields read or write method
    pub context: Option<Context>,

    /// An optional mapping that will be applied to the read value
    pub map: Option<MapExpr>,
    /// An optional indicator that this field is or needs to be aligned to the given byte alignment
    /// via padding.
    pub alignment: Option<usize>,
}
