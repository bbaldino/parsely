use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::{
    code_gen::gen_read::{generate_collection_read, generate_plain_read, wrap_in_optional},
    model_types::{wrap_read_with_padding_handling, CollectionLimit, MemberIdent},
    ParselyReadFieldReceiver, TypeExts,
};

use super::parsely_common_field_data::ParselyCommonFieldData;

/// A struct which represents all information needed for generating logic to read a field from a
/// buffer.
#[derive(Debug)]
pub struct ParselyReadFieldData {
    /// Data common between read and write for fields
    pub(crate) common: ParselyCommonFieldData,
    /// Required when there's a collection field
    pub(crate) collection_limit: Option<CollectionLimit>,
    /// Instead of reading the value of this field from the buffer, assign it from the given
    /// [`syn::Ident`]
    pub(crate) assign_from: Option<syn::Expr>,
    /// 'when' is required when there's an optional field
    pub(crate) when: Option<syn::Expr>,
}

impl ParselyReadFieldData {
    pub(crate) fn from_receiver(
        field_ident: MemberIdent,
        receiver: ParselyReadFieldReceiver,
    ) -> Self {
        let collection_limit = if receiver.ty.is_collection() {
            if let Some(count) = receiver.count {
                Some(CollectionLimit::Count(count))
            } else if let Some(while_pred) = receiver.while_pred {
                Some(CollectionLimit::While(while_pred))
            } else {
                panic!("Collection type must have 'count' or 'while' attribute");
            }
        } else {
            None
        };
        let when = if receiver.ty.is_option() {
            Some(
                receiver
                    .when
                    .expect("Option field must have 'when' attribute"),
            )
        } else {
            None
        };
        let common = ParselyCommonFieldData {
            ident: field_ident,
            ty: receiver.ty,
            assertion: receiver.common.assertion,
            context: receiver.common.context,
            map: receiver.common.map,
            alignment: receiver.common.alignment,
        };
        Self {
            common,
            collection_limit,
            assign_from: receiver.assign_from,
            when,
        }
    }

    /// Get the 'buffer type' of this field (the type that will be used when reading from or
    /// writing to the buffer): for wrapper types (like [`Option`] or [`Vec`]), this will be the
    /// inner type.
    pub(crate) fn buffer_type(&self) -> &syn::Type {
        if self.common.ty.is_option() || self.common.ty.is_collection() {
            self.common
                .ty
                .inner_type()
                .expect("Option or collection has an inner type")
        } else {
            &self.common.ty
        }
    }

    /// Get the context values that need to be passed to the read or write call for this field
    pub(crate) fn context_values(&self) -> Vec<syn::Expr> {
        if let Some(ref field_context) = self.common.context {
            field_context.expressions(&format!(
                "Read context for field '{}'",
                self.common.ident.as_friendly_string()
            ))
        } else {
            vec![]
        }
    }
}

impl ToTokens for ParselyReadFieldData {
    /// Given the data associated with a field, generate the code for properly reading it from a
    /// buffer.
    ///
    /// The attributes set in the [`ParselyReadFieldData`] all shape the logic necessary in order to
    /// properly parse this field.  Roughly, the processing is as follows:
    ///
    /// 1. Check if an 'assign_from' attribute is set.  If so, we don't read from the buffer at all and
    ///    instead just assign the field to the result of the given expression.
    /// 2. Check if a 'map' attribute is set.  If so, we'll read a value as a different type and then
    ///    pass it t othe map function to arrive at the final type and assign it to the field.
    /// 3. Check if the field is a collection.  If so, some kind of accompanying 'limit' attribute is
    ///    required: either a 'count' attribute or a `while_pred` attribute that defines how many
    ///    elements should be read.
    /// 4. If none of the above are the case, do a 'plain' read where we just read the type directly
    ///    from the buffer.
    /// 5. If an 'assertion' attribute is present then generate code to assert on the read value using
    ///    the given assertion function or closure.
    /// 6. After the code to perform the read has been generated, we check if the field is an option
    ///    type.  If so, a 'when' attribute is required.  This is an expression that determines when the
    ///    read should actually be done.
    /// 7. Finally, if an 'alignment' attribute is present, code is added to detect and consume any
    ///    padding after the read.
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut output = TokenStream::new();
        if let Some(ref assign_expr) = self.assign_from {
            output.extend(quote! {
                ParselyResult::<_>::Ok(#assign_expr)
            });
        } else if let Some(ref map_expr) = self.common.map {
            map_expr.to_read_map_tokens(&self.common.ident, &mut output);
        } else if self.common.ty.is_collection() {
            // We've ensure collection_limit is set in this case elswhere.
            let limit = self.collection_limit.as_ref().unwrap();
            let read_type = self.buffer_type();
            output.extend(generate_collection_read(
                limit,
                read_type,
                &self.context_values(),
            ));
        } else {
            output.extend(generate_plain_read(
                self.buffer_type(),
                &self.context_values(),
            ));
        }

        if let Some(ref assertion) = self.common.assertion {
            assertion
                .to_read_assertion_tokens(&self.common.ident.as_friendly_string(), &mut output);
        }
        let error_context = format!("Reading field '{}'", self.common.ident.as_friendly_string());
        output.extend(quote! {
            .with_context(|| #error_context)?
        });

        output = if self.common.ty.is_option() && self.common.map.is_none() {
            // We've ensured 'when' is set in this case elsehwere
            let when_expr = self.when.as_ref().unwrap();
            wrap_in_optional(when_expr, output)
        } else {
            output
        };

        output = if let Some(alignment) = self.common.alignment {
            wrap_read_with_padding_handling(&self.common.ident, alignment, output)
        } else {
            output
        };

        let field_variable_name = self.common.ident.as_variable_name();
        tokens.extend(quote! {
            let #field_variable_name = #output;
        })
    }
}
