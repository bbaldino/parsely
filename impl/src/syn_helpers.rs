pub(crate) trait TypeExts {
    fn is_option(&self) -> bool;
    fn is_collection(&self) -> bool;
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
