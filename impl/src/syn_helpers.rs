use quote::format_ident;

pub(crate) trait TypeExts {
    fn is_option(&self) -> bool;
    fn is_collection(&self) -> bool;
    fn is_wrapped(&self) -> bool;
    fn inner_type(&self) -> Option<&syn::Type>;
}

impl TypeExts for syn::Type {
    fn is_option(&self) -> bool {
        matches!(self, syn::Type::Path(type_path) if type_path.path.segments.len() == 1
                && type_path.path.segments[0].ident == "Option" && {
            true
        })
    }

    fn is_collection(&self) -> bool {
        matches!(self, syn::Type::Path(type_path) if type_path.path.segments.len() == 1
                && type_path.path.segments[0].ident == "Vec" && {
            true
        })
    }

    fn is_wrapped(&self) -> bool {
        self.is_option() || self.is_collection()
    }

    fn inner_type(&self) -> Option<&syn::Type> {
        // eprintln!("Getting inner type of {self:?}");
        let syn::Type::Path(ty) = self else {
            return None;
        };
        if ty.qself.is_some() {
            // qself is set when there's something like ""<Vec<T> as SomeTrait>::Associated".  For
            // now we'll ignore those cases.
            return None;
        }
        let path = &ty.path;

        // TODO: can we not validate the last path here?  Should this method take in some value
        // that would be used to compare to the last path segment here? ("Vec", "Option", etc)
        if path.segments.is_empty() {
            //|| path.segments.last().unwrap().ident != "Option" {
            return None;
        }
        if path.segments.len() != 1 {
            return None;
        }
        let last_segment = path.segments.last().unwrap();
        let syn::PathArguments::AngleBracketed(generics) = &last_segment.arguments else {
            return None;
        };
        if generics.args.len() != 1 {
            return None;
        }
        let syn::GenericArgument::Type(inner_type) = &generics.args[0] else {
            return None;
        };

        Some(inner_type)
    }
}

pub(crate) trait MemberExts {
    fn from_ident_or_index(ident: Option<&syn::Ident>, index: u32) -> Self;
    /// Return the value of this `syn::Member` as a user-friendly String.  This version is intended
    /// to be used for things like error messages.
    fn as_friendly_string(&self) -> String;
    /// Return the value of this `syn::Member` in the form of a `syn::Ident` that can be used as a
    /// local variable.
    fn as_variable_name(&self) -> syn::Ident;
}

impl MemberExts for syn::Member {
    fn from_ident_or_index(ident: Option<&syn::Ident>, index: u32) -> Self {
        if let Some(ident) = ident {
            syn::Member::Named(ident.clone())
        } else {
            syn::Member::Unnamed(syn::Index {
                index,
                span: proc_macro2::Span::call_site(),
            })
        }
    }

    fn as_friendly_string(&self) -> String {
        match self {
            syn::Member::Named(ref ident) => ident.to_string(),
            syn::Member::Unnamed(ref index) => format!("Field {}", index.index),
        }
    }

    fn as_variable_name(&self) -> syn::Ident {
        match self {
            syn::Member::Named(ref ident) => ident.clone(),
            syn::Member::Unnamed(ref index) => format_ident!("field_{}", index.index),
        }
    }
}
