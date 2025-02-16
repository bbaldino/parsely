pub(crate) trait TypeExts {
    fn is_option(&self) -> bool;
    fn option_inner_type(&self) -> Option<&syn::Type>;
}

impl TypeExts for syn::Type {
    fn is_option(&self) -> bool {
        matches!(self, syn::Type::Path(type_path) if type_path.path.segments.len() == 1
                && type_path.path.segments[0].ident == "Option" && {
            true
        })
    }

    fn option_inner_type(&self) -> Option<&syn::Type> {
        let syn::Type::Path(ty) = self else {
            return None;
        };
        if ty.qself.is_some() {
            return None;
        }

        let ty = &ty.path;

        if ty.segments.is_empty() || ty.segments.last().unwrap().ident != "Option" {
            return None;
        }

        if !(ty.segments.len() == 1
            || (ty.segments.len() == 3
                && ["core", "std"].contains(&ty.segments[0].ident.to_string().as_str())
                && ty.segments[1].ident == "option"))
        {
            return None;
        }

        let last_segment = ty.segments.last().unwrap();
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
