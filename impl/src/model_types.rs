use darling::{ast, FromMeta};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::parse::Parse;

#[derive(Debug)]
pub(crate) struct RequiredContext(pub(crate) Vec<TypedFnArg>);

impl RequiredContext {
    pub(crate) fn types(&self) -> Vec<&syn::Type> {
        self.0.iter().map(|t| t.ty()).collect()
    }

    pub(crate) fn assignments(&self) -> Vec<Local> {
        self.0
            .iter()
            .enumerate()
            .map(|(idx, fn_arg)| {
                let idx: syn::Index = idx.into();
                syn::parse2::<Local>(quote! {
                    let #fn_arg = ctx.#idx;
                })
                .unwrap()
            })
            .collect::<Vec<_>>()
    }
}

impl FromMeta for RequiredContext {
    fn from_none() -> Option<Self> {
        None
    }

    fn from_list(items: &[ast::NestedMeta]) -> darling::Result<Self> {
        let required_context: Vec<TypedFnArg> = items
            .iter()
            .map(|item| {
                match item {
                    // TODO: better error message here
                    ast::NestedMeta::Meta(_) => Err(darling::Error::unsupported_format(
                        "FnArg literals required",
                    )),
                    ast::NestedMeta::Lit(lit) => match lit {
                        syn::Lit::Str(s) => s.parse().map_err(|e| e.into()),
                        l => Err(darling::Error::unexpected_lit_type(l)),
                    },
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self(required_context))
    }
}

#[derive(Debug)]
pub(crate) struct Context(Vec<syn::Expr>);

impl Context {
    pub(crate) fn expressions(&self) -> Vec<syn::Expr> {
        self.0.clone()
    }
}

impl FromMeta for Context {
    fn from_none() -> Option<Self> {
        None
    }
    fn from_list(items: &[ast::NestedMeta]) -> darling::Result<Self> {
        let expressions: Vec<syn::Expr> = items
            .iter()
            .map(|item| {
                match item {
                    // TODO: better error message here
                    ast::NestedMeta::Meta(_) => Err(darling::Error::unsupported_format(
                        "FnArg literals required",
                    )),
                    ast::NestedMeta::Lit(lit) => match lit {
                        syn::Lit::Str(s) => s.parse().map_err(|e| e.into()),
                        l => Err(darling::Error::unexpected_lit_type(l)),
                    },
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self(expressions))
    }
}

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
