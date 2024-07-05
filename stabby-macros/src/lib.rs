//
// Copyright (c) 2023 ZettaScale Technology
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   Pierre Avital, <pierre.avital@me.com>
//

use std::collections::HashSet;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use syn::{parse::Parser, DeriveInput, TypeParamBound};

#[allow(dead_code)]
pub(crate) fn logfile(logfile: std::path::PathBuf) -> impl std::io::Write {
    use std::{fs::OpenOptions, io::BufWriter};
    let logfile = BufWriter::new(
        OpenOptions::new()
            .append(true)
            .create(true)
            .open(logfile)
            .unwrap(),
    );
    logfile
}

#[allow(unused_macros)]
macro_rules! log {
    ($path: literal, $pat: literal, $e: expr) => {{
        let logfile = std::path::PathBuf::from($path);
        use std::io::Write;
        let e = $e;
        writeln!(crate::logfile(logfile), $pat, e);
        e
    }};
    ($pat: literal, $e: expr) => {
        log!("logfile.txt", $pat, $e)
    };
    ($e: expr) => {
        log!("{}", $e)
    };
}

pub(crate) fn tl_mod() -> proc_macro2::TokenStream {
    match proc_macro_crate::crate_name("stabby-abi") {
        Ok(proc_macro_crate::FoundCrate::Itself) => return quote!(crate),
        Ok(proc_macro_crate::FoundCrate::Name(crate_name)) => {
            let crate_name = Ident::new(&crate_name, Span::call_site());
            return quote!(#crate_name);
        }
        _ => {}
    }
    match proc_macro_crate::crate_name("stabby")
        .expect("Couldn't find `stabby` in your dependencies")
    {
        proc_macro_crate::FoundCrate::Itself => quote!(crate::abi),
        proc_macro_crate::FoundCrate::Name(crate_name) => {
            let crate_name = Ident::new(&crate_name, Span::call_site());
            quote!(#crate_name::abi)
        }
    }
}

/// The lifeblood of stabby.
///
/// The README should provide all the necessary explainations.
#[proc_macro_attribute]
pub fn stabby(stabby_attrs: TokenStream, tokens: TokenStream) -> TokenStream {
    if let Ok(DeriveInput {
        attrs,
        vis,
        ident,
        generics,
        data,
    }) = syn::parse(tokens.clone())
    {
        match data {
            syn::Data::Struct(data) => {
                structs::stabby(attrs, vis, ident, generics, data, &stabby_attrs)
            }
            syn::Data::Enum(data) => {
                enums::stabby(attrs, vis, ident, generics, data, &stabby_attrs)
            }
            syn::Data::Union(data) => {
                unions::stabby(attrs, vis, ident, generics, data, &stabby_attrs)
            }
        }
    } else if let Ok(fn_spec) = syn::parse(tokens.clone()) {
        functions::stabby(syn::parse(stabby_attrs).unwrap(), fn_spec)
    } else if let Ok(trait_spec) = syn::parse(tokens.clone()) {
        traits::stabby(trait_spec, &stabby_attrs)
    } else if let Ok(async_block) = syn::parse::<syn::ExprAsync>(tokens) {
        quote!(Box::new(#async_block).into())
    } else {
        panic!("Expected a type declaration, a trait declaration or a function declaration")
    }
    .into()
}

/// Returns the appropriate type of vtable for a trait object.
///
/// Usage: `vtable!(TraitA + TraitB<Output=u16> + Send + Sync)`
/// Note that the ordering of traits is significant.
#[proc_macro]
pub fn vtable(tokens: TokenStream) -> TokenStream {
    let st = tl_mod();
    let bounds =
        syn::punctuated::Punctuated::<TypeParamBound, syn::token::Add>::parse_separated_nonempty
            .parse(tokens)
            .unwrap();
    let mut vt = quote!(#st::vtable::VtDrop);
    for bound in bounds {
        match &bound {
            TypeParamBound::Trait(t) => vt = quote!(< dyn #t as #st::vtable::CompoundVt >::Vt<#vt>),
            TypeParamBound::Lifetime(lt) => panic!("Cannot give lifetimes to vtables, use `Dyn<{lt}, P, Vt>` or `DynRef<{lt}, Vt> instead`"),
        }
    }
    vt.into()
}

enum PtrType {
    Path(proc_macro2::TokenStream),
    Ref,
    RefMut,
}
struct DynPtr {
    ptr: PtrType,
    bounds: Vec<syn::TraitBound>,
    lifetime: Option<syn::Lifetime>,
}
impl syn::parse::Parse for DynPtr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let (mut this, elem) = match input.parse::<syn::Type>()? {
            syn::Type::Path(syn::TypePath {
                path:
                    syn::Path {
                        leading_colon,
                        mut segments,
                    },
                ..
            }) => {
                let syn::PathSegment {
                    ident,
                    arguments:
                        syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                            colon2_token: None,
                            mut args,
                            ..
                        }),
                } = segments.pop().unwrap().into_value()
                else {
                    panic!()
                };
                if args.len() != 1 {
                    panic!("Pointer-type must have exactly one generic argument containing `dyn Bounds`")
                }
                let arg = args.pop().unwrap().into_value();
                let syn::GenericArgument::Type(ty) = arg else {
                    panic!()
                };
                (
                    DynPtr {
                        ptr: PtrType::Path(quote!(#leading_colon #segments #ident)),
                        lifetime: None,
                        bounds: Vec::new(),
                    },
                    ty,
                )
            }
            syn::Type::Reference(syn::TypeReference {
                lifetime,
                mutability,
                elem,
                ..
            }) => (
                DynPtr {
                    ptr: if mutability.is_some() {
                        PtrType::RefMut
                    } else {
                        PtrType::Ref
                    },
                    lifetime,
                    bounds: Vec::new(),
                },
                *elem,
            ),
            _ => panic!("Only references and paths are supported by this macro"),
        };
        let syn::Type::TraitObject(syn::TypeTraitObject { bounds, .. }) = elem else {
            panic!("expected `dyn` not found")
        };
        for bound in bounds {
            match bound {
                TypeParamBound::Trait(t) => this.bounds.push(t),
                TypeParamBound::Lifetime(lt) => {
                    if this.lifetime.is_some() {
                        panic!("Only a single lifetime is supported in this macro")
                    } else {
                        this.lifetime = Some(lt)
                    }
                }
            }
        }
        Ok(this)
    }
}

/// Returns the appropriate type for a stabby equivalent of a trait object.
///
/// Usage: `dynptr!(Box<dyn TraitA + TraitB<Output=u16> + Send + Sync + 'a>)`
/// Note that the ordering of traits is significant.
#[proc_macro]
pub fn dynptr(tokens: TokenStream) -> TokenStream {
    let st = tl_mod();
    let DynPtr {
        ptr,
        bounds,
        lifetime,
    } = syn::parse(tokens).unwrap();
    let mut vt = quote!(#st::vtable::VtDrop);
    for bound in bounds {
        vt = quote!(< dyn #bound as #st::vtable::CompoundVt >::Vt<#vt>);
    }
    let lifetime = lifetime.unwrap_or(syn::Lifetime::new("'static", Span::call_site()));
    match ptr {
        PtrType::Path(path) => quote!(#st::Dyn<#lifetime, #path<()>, #vt>),
        PtrType::RefMut => quote!(#st::Dyn<#lifetime, &#lifetime mut (), #vt>),
        PtrType::Ref => quote!(#st::DynRef<#lifetime, #vt>),
    }
    .into()
}

mod enums;
mod functions;
mod structs;
mod traits;
mod unions;
pub(crate) mod utils;

mod tyops;
#[proc_macro]
pub fn tyeval(tokens: TokenStream) -> TokenStream {
    tyops::tyeval(&tokens.into()).into()
}

mod gen_closures;
#[proc_macro]
pub fn gen_closures_impl(_: TokenStream) -> TokenStream {
    gen_closures::gen_closures().into()
}

#[derive(Clone)]
enum Type {
    Syn(syn::Type),
    Report(Report),
}
impl From<syn::Type> for Type {
    fn from(value: syn::Type) -> Self {
        Self::Syn(value)
    }
}
impl From<Report> for Type {
    fn from(value: Report) -> Self {
        Self::Report(value)
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Tyty {
    Struct,
    Union,
    Enum(enums::Repr),
}
impl ToTokens for Tyty {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let st = crate::tl_mod();
        let tyty = match self {
            Tyty::Struct => quote!(#st::report::TyTy::Struct),
            Tyty::Union => quote!(#st::report::TyTy::Union),
            Tyty::Enum(r) => {
                let s = format!("{r:?}");
                quote!(#st::report::TyTy::Enum(#st::str::Str::new(#s)))
            }
        };
        tokens.extend(tyty);
    }
}
#[derive(Clone)]
pub(crate) struct Report {
    name: String,
    fields: Vec<(String, Type)>,
    version: u32,
    module: proc_macro2::TokenStream,
    pub tyty: Tyty,
}
impl Report {
    pub fn r#struct(
        name: impl Into<String>,
        version: u32,
        module: proc_macro2::TokenStream,
    ) -> Self {
        Self {
            name: name.into(),
            fields: Vec::new(),
            version,
            module: if module.is_empty() {
                quote!(::core::module_path!())
            } else {
                module
            },
            tyty: Tyty::Struct,
        }
    }
    pub fn r#enum(name: impl Into<String>, version: u32, module: proc_macro2::TokenStream) -> Self {
        Self {
            name: name.into(),
            fields: Vec::new(),
            version,
            module: if module.is_empty() {
                quote!(::core::module_path!())
            } else {
                module
            },
            tyty: Tyty::Enum(enums::Repr::Stabby),
        }
    }
    pub fn r#union(
        name: impl Into<String>,
        version: u32,
        module: proc_macro2::TokenStream,
    ) -> Self {
        Self {
            name: name.into(),
            fields: Vec::new(),
            version,
            module: if module.is_empty() {
                quote!(::core::module_path!())
            } else {
                module
            },
            tyty: Tyty::Union,
        }
    }
    pub fn add_field(&mut self, name: String, ty: impl Into<Type>) {
        self.fields.push((name, ty.into()));
    }
    fn __bounds(
        &self,
        bounded_types: &mut HashSet<syn::Type>,
        mut report_bounds: proc_macro2::TokenStream,
        st: &proc_macro2::TokenStream,
    ) -> proc_macro2::TokenStream {
        for (_, ty) in self.fields.iter() {
            match ty {
                Type::Syn(ty) => {
                    if bounded_types.insert(ty.clone()) {
                        report_bounds = quote!(#ty: #st::IStable, #report_bounds);
                    }
                }
                Type::Report(report) => {
                    report_bounds = report.__bounds(bounded_types, report_bounds, st)
                }
            }
        }
        report_bounds
    }
    pub fn bounds(&self) -> proc_macro2::TokenStream {
        let st = crate::tl_mod();
        let mut bounded_types = HashSet::new();
        self.__bounds(&mut bounded_types, quote!(), &st)
    }

    pub fn crepr(&self) -> proc_macro2::TokenStream {
        let st = crate::tl_mod();
        match self.tyty {
            Tyty::Struct => {
                // TODO: For user comfort, having this would be better, but reading from env vars doesn't work in proc-macros.
                // let max_tuple = std::env::var("CARGO_CFG_STABBY_MAX_TUPLE")
                //     .map_or(32, |s| s.parse().unwrap_or(32))
                //     .max(10);
                // panic!("{max_tuple}");
                // if self.fields.len() > max_tuple {
                //     panic!("stabby doesn't support structures with more than {max_tuple} direct fields, you should probably split it at that point; or you can also raise this limit using `--cfg stabby_max_tuple=N` to your RUSTFLAGS at the cost of higher compile times")
                // }
                let tuple = quote::format_ident!("Tuple{}", self.fields.len());
                let fields = self.fields.iter().map(|f| match &f.1 {
                    Type::Syn(ty) => quote! (<#ty as #st::IStable>::CType),
                    Type::Report(r) => r.crepr(),
                });
                quote! {
                    #st::tuple::#tuple <#(#fields,)*>
                }
            }
            Tyty::Union => {
                let mut crepr = quote!(());
                for f in &self.fields {
                    let ty = match &f.1 {
                        Type::Syn(ty) => quote! (#ty),
                        Type::Report(r) => r.crepr(),
                    };
                    crepr = quote!(#st::Union<#ty, #crepr>);
                }
                quote! {<#crepr as #st::IStable>::CType}
            }
            Tyty::Enum(r) => {
                let mut clone = self.clone();
                clone.tyty = Tyty::Union;
                let crepr = clone.crepr();
                let determinant = quote::format_ident!("{r:?}");
                quote! {
                    #st::tuple::Tuple2<#determinant, #crepr>
                }
            }
        }
    }
}
impl ToTokens for Report {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let st = crate::tl_mod();
        let mut fields = quote!(None);

        for (name, ty) in &self.fields {
            fields = match ty {
                Type::Syn(ty) => quote! {
                    Some(& #st::report::FieldReport {
                        name: #st::str::Str::new(#name),
                        ty: <#ty as #st::IStable>::REPORT,
                        next_field: #st::StableLike::new(#fields)
                    })
                },
                Type::Report(re) => quote! {
                    Some(& #st::report::FieldReport {
                        name: #st::str::Str::new(#name),
                        ty: &#re,
                        next_field: #st::StableLike::new(#fields)
                    })
                },
            }
        }
        let Self {
            name,
            version,
            tyty,
            module,
            ..
        } = self;
        tokens.extend(quote!(#st::report::TypeReport {
            name: #st::str::Str::new(#name),
            module: #st::str::Str::new(#module),
            fields: unsafe{#st::StableLike::new(#fields)},
            version: #version,
            tyty: #tyty,
        }));
    }
}

#[proc_macro_attribute]
pub fn export(attrs: TokenStream, fn_spec: TokenStream) -> TokenStream {
    crate::functions::export(attrs, syn::parse(fn_spec).unwrap()).into()
}

#[proc_macro_attribute]
pub fn import(attrs: TokenStream, fn_spec: TokenStream) -> TokenStream {
    crate::functions::import(attrs, syn::parse(fn_spec).unwrap()).into()
}

#[proc_macro]
pub fn canary_suffixes(_: TokenStream) -> TokenStream {
    let mut stream = quote::quote!();
    for (name, spec) in functions::CanarySpec::ARRAY.iter().skip(2) {
        let id = quote::format_ident!("CANARY_{}", name.to_ascii_uppercase());
        let suffix = spec.to_string();
        stream.extend(quote::quote!(pub const #id: &'static str = #suffix;));
    }
    stream.into()
}

trait Unself {
    fn unself(&self, this: &syn::Ident) -> Self;
}
impl Unself for syn::Path {
    fn unself(&self, this: &syn::Ident) -> Self {
        let syn::Path {
            leading_colon,
            segments,
        } = self;
        if self.is_ident("Self") || self.is_ident(this) {
            let st = crate::tl_mod();
            return syn::parse2(quote! {#st::istable::_Self}).unwrap();
        }
        syn::Path {
            leading_colon: *leading_colon,
            segments: segments
                .iter()
                .map(|syn::PathSegment { ident, arguments }| syn::PathSegment {
                    ident: ident.clone(),
                    arguments: match arguments {
                        syn::PathArguments::None => syn::PathArguments::None,
                        syn::PathArguments::AngleBracketed(
                            syn::AngleBracketedGenericArguments {
                                colon2_token,
                                lt_token,
                                args,
                                gt_token,
                            },
                        ) => syn::PathArguments::AngleBracketed(
                            syn::AngleBracketedGenericArguments {
                                colon2_token: *colon2_token,
                                lt_token: *lt_token,
                                args: args
                                    .iter()
                                    .map(|arg| match arg {
                                        syn::GenericArgument::Type(ty) => {
                                            syn::GenericArgument::Type(ty.unself(this))
                                        }
                                        syn::GenericArgument::Binding(syn::Binding {
                                            ident,
                                            eq_token,
                                            ty,
                                        }) => syn::GenericArgument::Binding(syn::Binding {
                                            ident: ident.clone(),
                                            eq_token: *eq_token,
                                            ty: ty.unself(this),
                                        }),
                                        syn::GenericArgument::Constraint(syn::Constraint {
                                            ident,
                                            colon_token,
                                            bounds,
                                        }) => syn::GenericArgument::Constraint(syn::Constraint {
                                            ident: ident.clone(),
                                            colon_token: *colon_token,
                                            bounds: bounds.iter().map(|b| b.unself(this)).collect(),
                                        }),
                                        other => other.clone(),
                                    })
                                    .collect(),
                                gt_token: *gt_token,
                            },
                        ),
                        syn::PathArguments::Parenthesized(syn::ParenthesizedGenericArguments {
                            paren_token,
                            inputs,
                            output,
                        }) => {
                            syn::PathArguments::Parenthesized(syn::ParenthesizedGenericArguments {
                                paren_token: *paren_token,
                                inputs: inputs.iter().map(|t| t.unself(this)).collect(),
                                output: match output {
                                    syn::ReturnType::Default => syn::ReturnType::Default,
                                    syn::ReturnType::Type(arrow, ty) => {
                                        syn::ReturnType::Type(*arrow, Box::new(ty.unself(this)))
                                    }
                                },
                            })
                        }
                    },
                })
                .collect(),
        }
    }
}
impl Unself for syn::TypeParamBound {
    fn unself(&self, this: &syn::Ident) -> Self {
        match self {
            TypeParamBound::Trait(syn::TraitBound {
                paren_token,
                modifier,
                lifetimes,
                path,
            }) => TypeParamBound::Trait(syn::TraitBound {
                paren_token: *paren_token,
                modifier: *modifier,
                lifetimes: lifetimes.clone(),
                path: path.unself(this),
            }),
            TypeParamBound::Lifetime(l) => TypeParamBound::Lifetime(l.clone()),
        }
    }
}
impl Unself for syn::Type {
    fn unself(&self, this: &syn::Ident) -> syn::Type {
        match self {
            syn::Type::Array(syn::TypeArray {
                elem,
                len,
                bracket_token,
                semi_token,
            }) => syn::Type::Array(syn::TypeArray {
                elem: Box::new(elem.unself(this)),
                len: len.clone(),
                bracket_token: *bracket_token,
                semi_token: *semi_token,
            }),
            syn::Type::BareFn(syn::TypeBareFn {
                lifetimes,
                unsafety,
                abi,
                inputs,
                output,
                fn_token,
                paren_token,
                variadic,
            }) => syn::Type::BareFn(syn::TypeBareFn {
                lifetimes: lifetimes.clone(),
                unsafety: *unsafety,
                abi: abi.clone(),
                inputs: inputs
                    .iter()
                    .map(|syn::BareFnArg { attrs, name, ty }| syn::BareFnArg {
                        attrs: attrs.clone(),
                        name: name.clone(),
                        ty: ty.unself(this),
                    })
                    .collect(),
                output: match output {
                    syn::ReturnType::Default => syn::ReturnType::Default,
                    syn::ReturnType::Type(arrow, ret) => {
                        syn::ReturnType::Type(*arrow, Box::new(ret.unself(this)))
                    }
                },
                fn_token: *fn_token,
                paren_token: *paren_token,
                variadic: variadic.clone(),
            }),
            syn::Type::Group(syn::TypeGroup { group_token, elem }) => {
                syn::Type::Group(syn::TypeGroup {
                    group_token: *group_token,
                    elem: Box::new(elem.unself(this)),
                })
            }
            syn::Type::ImplTrait(syn::TypeImplTrait { impl_token, bounds }) => {
                syn::Type::ImplTrait(syn::TypeImplTrait {
                    impl_token: *impl_token,
                    bounds: bounds.into_iter().map(|p| p.unself(this)).collect(),
                })
            }
            syn::Type::Paren(syn::TypeParen { paren_token, elem }) => {
                syn::Type::Paren(syn::TypeParen {
                    paren_token: *paren_token,
                    elem: Box::new(elem.unself(this)),
                })
            }
            syn::Type::Path(syn::TypePath { qself, path }) => syn::Type::Path(syn::TypePath {
                qself: qself.as_ref().map(
                    |syn::QSelf {
                         lt_token,
                         ty,
                         position,
                         as_token,
                         gt_token,
                     }| syn::QSelf {
                        lt_token: *lt_token,
                        ty: Box::new(ty.unself(this)),
                        position: *position,
                        as_token: *as_token,
                        gt_token: *gt_token,
                    },
                ),
                path: path.unself(this),
            }),
            syn::Type::Ptr(syn::TypePtr {
                star_token,
                const_token,
                mutability,
                elem,
            }) => syn::Type::Ptr(syn::TypePtr {
                star_token: *star_token,
                const_token: *const_token,
                mutability: *mutability,
                elem: Box::new(elem.unself(this)),
            }),
            syn::Type::Reference(syn::TypeReference {
                and_token,
                lifetime,
                mutability,
                elem,
            }) => syn::Type::Reference(syn::TypeReference {
                and_token: *and_token,
                lifetime: lifetime.clone(),
                mutability: *mutability,
                elem: Box::new(elem.unself(this)),
            }),
            syn::Type::Slice(syn::TypeSlice {
                bracket_token,
                elem,
            }) => syn::Type::Slice(syn::TypeSlice {
                bracket_token: *bracket_token,
                elem: Box::new(elem.unself(this)),
            }),
            syn::Type::TraitObject(syn::TypeTraitObject { dyn_token, bounds }) => {
                syn::Type::TraitObject(syn::TypeTraitObject {
                    dyn_token: *dyn_token,
                    bounds: bounds.iter().map(|b| b.unself(this)).collect(),
                })
            }
            syn::Type::Tuple(syn::TypeTuple { paren_token, elems }) => {
                syn::Type::Tuple(syn::TypeTuple {
                    paren_token: *paren_token,
                    elems: elems.iter().map(|ty| ty.unself(this)).collect(),
                })
            }
            o => o.clone(),
        }
    }
}
