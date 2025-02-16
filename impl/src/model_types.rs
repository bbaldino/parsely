use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::Parse;

/// Same as [`syn::FnArg`] but only allows the [`syn::FnArg::Typed`] variant.
#[derive(Debug)]
pub(crate) struct TypedFnArg(syn::FnArg);

impl Parse for TypedFnArg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        match syn::FnArg::parse(input) {
            Ok(syn::FnArg::Typed(t)) => Ok(Self(syn::FnArg::Typed(t))),
            Ok(syn::FnArg::Receiver(_)) => todo!("figure out error to return here"),
            Err(e) => Err(e),
        }
    }
}

impl TypedFnArg {
    pub(crate) fn ty(&self) -> &syn::Type {
        match self.0 {
            syn::FnArg::Typed(ref t) => &t.ty,
            _ => unreachable!(),
        }
    }
}

impl ToTokens for TypedFnArg {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens)
    }
}

/// [`syn::Local`] exists but doesn't have its own parse method, it get parsed as part of
/// [`syn::Stmt`]
pub(crate) struct Local(pub(crate) syn::Local);

impl Parse for Local {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        match syn::Stmt::parse(input) {
            Ok(syn::Stmt::Local(l)) => Ok(Self(l)),
            _ => todo!("error tbd"),
        }
    }
}

impl ToTokens for Local {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.0.to_tokens(tokens)
    }
}
