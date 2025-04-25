use darling::{ast, FromMeta};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::parse::Parse;

use crate::get_crate_name;

pub(crate) enum CollectionLimit {
    Count(syn::Expr),
    While(syn::Expr),
}

#[derive(Debug)]
pub(crate) struct TypedFnArgList(pub(crate) Vec<TypedFnArg>);

impl TypedFnArgList {
    /// Get the types from this [`TypedFnArgList`] as a vec
    pub(crate) fn types(&self) -> Vec<&syn::Type> {
        self.0.iter().map(|t| t.ty()).collect()
    }

    pub(crate) fn names(&self) -> Vec<&syn::Ident> {
        self.0.iter().map(|t| t.name()).collect()
    }

    // TODO: this is context-specific, but now this type is more generic.  move it?
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

impl FromMeta for TypedFnArgList {
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

#[derive(Debug, Default)]
pub(crate) struct Context(Vec<syn::Expr>);

impl Context {
    /// Get the context arguments as a vector of expressions, with an erorr context including the
    /// given `context` value.
    pub(crate) fn expressions(&self, context: &str) -> Vec<syn::Expr> {
        // We support Context expressions that return a ParselyResult or a raw value.  So now wrap
        // all the expressions with code that will normalize all of the results into
        // ParselyResults.
        self.0
            .iter()
            .cloned()
            .enumerate()
            .map(|(idx, e)| {
                syn::parse2(quote! {
                    (#e).into_parsely_result().with_context(|| format!("{}: expression {}", #context, #idx))?
                })
                .unwrap()
            })
            .collect()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
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
            _ => unreachable!("TypedFnArg should always be typed"),
        }
    }

    pub(crate) fn name(&self) -> &syn::Ident {
        match self.0 {
            syn::FnArg::Typed(ref pat_type) => match *pat_type.pat {
                syn::Pat::Ident(ref pat_ident) => &pat_ident.ident,
                _ => unreachable!("TypedFnArg should always have an ident"),
            },
            _ => unreachable!("TypedFnArg should always be typed"),
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
            _ => Err(input.error("Failed to parse Local, expected a local declaration")),
        }
    }
}

impl ToTokens for Local {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.0.to_tokens(tokens)
    }
}

#[derive(Debug)]
pub(crate) enum ExprOrFunc {
    Expr(syn::Expr),
    Func(syn::Ident),
}

impl Parse for ExprOrFunc {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if let Ok(expr) = syn::Expr::parse(input) {
            Ok(ExprOrFunc::Expr(expr))
        } else if let Ok(id) = syn::Ident::parse(input) {
            Ok(ExprOrFunc::Func(id))
        } else {
            Err(input.error("Failed to parse ExporOrFunc: expected ExprClosure or Ident"))
        }
    }
}

impl ToTokens for ExprOrFunc {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            ExprOrFunc::Expr(e) => e.to_tokens(tokens),
            ExprOrFunc::Func(f) => f.to_tokens(tokens),
        }
    }
}

impl FromMeta for ExprOrFunc {
    fn from_string(value: &str) -> darling::Result<Self> {
        syn::parse_str::<ExprOrFunc>(value).map_err(darling::Error::custom)
    }
}

#[derive(Debug)]
pub(crate) enum FuncOrClosure {
    Func(syn::Ident),
    Closure(syn::ExprClosure),
}

impl Parse for FuncOrClosure {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if let Ok(expr_closure) = syn::ExprClosure::parse(input) {
            Ok(FuncOrClosure::Closure(expr_closure))
        } else if let Ok(id) = syn::Ident::parse(input) {
            Ok(FuncOrClosure::Func(id))
        } else {
            Err(input.error("Failed to parse FuncOrClosure: expected ExprClosure or Ident"))
        }
    }
}

impl ToTokens for FuncOrClosure {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            FuncOrClosure::Func(m) => m.to_tokens(tokens),
            FuncOrClosure::Closure(c) => c.to_tokens(tokens),
        }
    }
}

impl FromMeta for FuncOrClosure {
    fn from_string(value: &str) -> darling::Result<Self> {
        syn::parse_str::<FuncOrClosure>(value).map_err(darling::Error::custom)
    }
}

/// A map expression that can be applied to a value after reading or before writing
#[derive(Debug)]
pub(crate) struct MapExpr(FuncOrClosure);

impl FromMeta for MapExpr {
    fn from_string(value: &str) -> darling::Result<Self> {
        Ok(Self(FuncOrClosure::from_string(value)?))
    }
}

impl MapExpr {
    pub(crate) fn to_read_map_tokens(&self, field_name: &syn::Ident, tokens: &mut TokenStream) {
        let crate_name = get_crate_name();
        let field_name_string = field_name.to_string();
        let map_expr = &self.0;
        // TODO: is there a case where context might be required for reading the 'buffer_type'
        // value?
        tokens.extend(quote! {
            {
                let original_value = ::#crate_name::ParselyRead::read::<T>(buf, ())
                    .with_context(|| format!("Reading raw value for field '{}'", #field_name_string))?;
                (#map_expr)(original_value).into_parsely_result_read()
                    .with_context(|| format!("Mapping raw value for field '{}'", #field_name_string))
            }
        })
    }

    pub(crate) fn to_write_map_tokens(&self, field_name: &syn::Ident, tokens: &mut TokenStream) {
        let crate_name = get_crate_name();
        let field_name_string = field_name.to_string();
        let map_expr = &self.0;
        tokens.extend(quote! {
            {
                let mapped_value = (#map_expr)(&self.#field_name).into_parsely_result()
                    .with_context(|| format!("Mapping raw value for field '{}'", #field_name_string))?;
                ::#crate_name::ParselyWrite::write::<B, T>(&mapped_value, buf, ())
                    .with_context(|| format!("Writing mapped value for field '{}'", #field_name_string))?;
            }
        })
    }
}

/// An assertion that can be used after reading a value or before writing one
#[derive(Debug)]
pub(crate) struct Assertion(FuncOrClosure);

impl Parse for Assertion {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self(FuncOrClosure::parse(input)?))
    }
}

impl FromMeta for Assertion {
    fn from_string(value: &str) -> darling::Result<Self> {
        Ok(Self(FuncOrClosure::from_string(value)?))
    }
}

impl Assertion {
    pub(crate) fn to_read_assertion_tokens(&self, field_name: &str, tokens: &mut TokenStream) {
        let assertion = &self.0;
        let assertion_string = quote! { #assertion }.to_string();
        tokens.extend(quote! {
            .and_then(|read_value| {
                let assertion_func = #assertion;
                if !assertion_func(&read_value) {
                    bail!("Assertion failed: value of field '{}' ('{:?}') didn't pass assertion: '{}'", #field_name, read_value, #assertion_string)
                }
                Ok(read_value)
            })
        });
    }

    pub(crate) fn to_write_assertion_tokens(&self, field_name: &str, tokens: &mut TokenStream) {
        let assertion = &self.0;
        let assertion_string = quote! { #assertion }.to_string();
        let assertion_func_ident = format_ident!("__{}_assertion_func", field_name);
        let field_name_ident = format_ident!("{field_name}");
        tokens.extend(quote! {
            let #assertion_func_ident = #assertion;
            if !#assertion_func_ident(&self.#field_name_ident) {
                bail!("Assertion failed: value of field '{}' ('{:?}') didn't pass assertion: '{}'", #field_name, self.#field_name_ident, #assertion_string)
            }
        })
    }
}

pub(crate) fn wrap_read_with_padding_handling(
    element_ident: &syn::Ident,
    alignment: usize,
    inner: TokenStream,
) -> TokenStream {
    let bytes_read_before_ident = format_ident!("__bytes_read_before_{element_ident}_read");
    let bytes_read_after_ident = format_ident!("__bytes_read_after_{element_ident}_read");
    let amount_read_ident = format_ident!("__bytes_read_for_{element_ident}");

    quote! {
        let #bytes_read_before_ident = buf.remaining_bytes();

        #inner

        let #bytes_read_after_ident = buf.remaining_bytes();
        let mut #amount_read_ident = #bytes_read_before_ident - #bytes_read_after_ident;
        while #amount_read_ident % #alignment != 0 {
            let _ = buf.get_u8().context("padding")?;
            #amount_read_ident += 1;
        }
    }
}

pub(crate) fn wrap_write_with_padding_handling(
    element_ident: &syn::Ident,
    alignment: usize,
    inner: TokenStream,
) -> TokenStream {
    let bytes_written_before_ident = format_ident!("__bytes_written_before_{element_ident}_write");
    let bytes_written_after_ident = format_ident!("__bytes_written_after_{element_ident}_write");
    let amount_written_ident = format_ident!("__bytes_written_for_{element_ident}");

    quote! {
        let #bytes_written_before_ident = buf.remaining_bytes();

        #inner

        let #bytes_written_after_ident = buf.remaining_bytes();
        let mut #amount_written_ident = #bytes_written_after_ident - #bytes_written_before_ident;
        while #amount_written_ident % #alignment != 0 {
            buf.put_u8(0).context("padding")?;
            #amount_written_ident += 1;
        }
    }
}
