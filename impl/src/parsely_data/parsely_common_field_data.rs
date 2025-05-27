use crate::{model_types::MemberIdent, Assertion, Context, MapExpr};

/// Items that are needed for both reading and writing a field to/from a buffer.
#[derive(Debug)]
pub(crate) struct ParselyCommonFieldData {
    pub(crate) ident: MemberIdent,
    /// The field's type
    pub(crate) ty: syn::Type,

    pub(crate) assertion: Option<Assertion>,
    /// Values that need to be passed as context to this fields read or write method
    pub(crate) context: Option<Context>,

    /// An optional mapping that will be applied to the read value
    pub(crate) map: Option<MapExpr>,
    /// An optional indicator that this field is or needs to be aligned to the given byte alignment
    /// via padding.
    pub(crate) alignment: Option<usize>,
}
