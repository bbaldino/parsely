use crate::{model_types::MemberIdent, syn_helpers::TypeExts, Assertion, Context, MapExpr};

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

impl ParselyCommonFieldData {
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
    pub(crate) fn context_values(&self) -> Vec<syn::Expr> {
        if let Some(ref field_context) = self.context {
            field_context.expressions(&format!(
                "Read context for field '{}'",
                self.ident.as_friendly_string()
            ))
        } else {
            vec![]
        }
    }
}
