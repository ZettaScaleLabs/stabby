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

use std::ops::Deref;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    token::{Const, Mut, Unsafe},
    Abi, AngleBracketedGenericArguments, BoundLifetimes, Expr, Lifetime, PatType, Path,
    PathArguments, PathSegment, Receiver, Signature, TraitItemMethod, TraitItemType, Type,
    TypeArray, TypeBareFn, TypeGroup, TypeParen, TypePath, TypePtr, TypeReference, TypeTuple,
};

use crate::utils::{IGenerics, Unbound};
#[derive(Clone, Default)]
struct SelfDependentTypes {
    inner: Vec<Ty>,
}
impl SelfDependentTypes {
    fn push(&mut self, ty: Ty) -> bool {
        if self.find(&ty).is_none() {
            self.inner.push(ty);
            true
        } else {
            false
        }
    }
    fn extend(&mut self, it: impl Iterator<Item = Ty>) {
        for ty in it {
            self.push(ty);
        }
    }
    fn find(&self, ty: &Ty) -> Option<usize> {
        let tystr = quote!(#ty).to_string();
        self.iter().position(|t| quote!(#t).to_string() == tystr)
    }
    fn unselfed(&self, elem: &Ty) -> Ty {
        match elem {
            Ty::Never => Ty::Never,
            Ty::Unit => Ty::Unit,
            Ty::SelfReferencial(ty) => {
                let Some(n) = self.find(ty) else {
                    panic!("Couldn't find type {ty}")
                };
                Ty::Path {
                    segment: Self::unselfed_n(n),
                    arguments: Arguments::None,
                    next: None,
                }
            }
            Ty::Arbitrary { prefix, next } => Ty::Arbitrary {
                prefix: prefix.clone(),
                next: Box::new(self.unselfed(next)),
            },
            Ty::Reference {
                lifetime,
                mutability,
                elem,
            } => Ty::Reference {
                lifetime: lifetime.clone(),
                mutability: *mutability,
                elem: Box::new(self.unselfed(elem)),
            },
            Ty::Ptr {
                const_token,
                mutability,
                elem,
            } => Ty::Ptr {
                const_token: *const_token,
                mutability: *mutability,
                elem: Box::new(self.unselfed(elem)),
            },
            Ty::Array { elem, len } => Ty::Array {
                elem: Box::new(self.unselfed(elem)),
                len: len.clone(),
            },
            Ty::BareFn {
                lifetimes,
                unsafety,
                abi,
                inputs,
                output,
            } => Ty::BareFn {
                lifetimes: lifetimes.clone(),
                unsafety: *unsafety,
                abi: abi.clone(),
                inputs: inputs.iter().map(|elem| self.unselfed(elem)).collect(),
                output: Box::new(self.unselfed(output)),
            },
            Ty::Path {
                segment,
                arguments,
                next,
            } => Ty::Path {
                segment: segment.clone(),
                arguments: match arguments {
                    Arguments::None => Arguments::None,
                    Arguments::AngleBracketed { generics } => Arguments::AngleBracketed {
                        generics: generics
                            .iter()
                            .map(|g| match g {
                                GenericArgument::Type(ty) => {
                                    GenericArgument::Type(self.unselfed(ty))
                                }
                                g => g.clone(),
                            })
                            .collect(),
                    },
                },
                next: next.as_ref().map(|elem| Box::new(self.unselfed(elem))),
            },
            Ty::LeadingColon { next } => Ty::LeadingColon {
                next: Box::new(self.unselfed(next)),
            },
            Ty::Qualified {
                target,
                as_trait,
                next,
            } => Ty::Qualified {
                target: Box::new(self.unselfed(target)),
                as_trait: Box::new(self.unselfed(as_trait)),
                next: Box::new(self.unselfed(next)),
            },
        }
    }
    fn unselfed_iter(&self) -> impl Iterator<Item = Ident> {
        (0..self.inner.len()).map(Self::unselfed_n)
    }
    fn unselfed_n(i: usize) -> Ident {
        quote::format_ident!("_stabby_unselfed_{i}")
    }
}
impl Deref for SelfDependentTypes {
    type Target = [Ty];
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
struct DynTraitDescription<'a> {
    ident: &'a Ident,
    vis: &'a syn::Visibility,
    generics: &'a syn::Generics,
    functions: Vec<DynTraitFn<'a>>,
    mut_functions: Vec<DynTraitFn<'a>>,
    self_dependent_types: SelfDependentTypes,
    vt_attrs: Vec<proc_macro2::TokenStream>,
    bounds: Vec<DynTraitBound>,
    check_bounds: bool,
}
struct DynTraitBound {
    target: Ty,
    lifetimes: Option<BoundLifetimes>,
    bound: Result<Ty, Lifetime>,
}
struct DynTraitFn<'a> {
    ident: &'a Ident,
    generics: &'a syn::Generics,
    abi: TokenStream,
    unsafety: Option<syn::token::Unsafe>,
    receiver: Receiver,
    inputs: Vec<Ty>,
    output: Option<Ty>,
}
impl quote::ToTokens for DynTraitFn<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let DynTraitFn {
            ident,
            generics,
            abi,
            unsafety,
            receiver,
            inputs,
            output,
        } = self;
        let inputs = inputs.iter().enumerate().map(|(i, ty)| {
            let id = quote::format_ident!("_{i}");
            quote!(#id: #ty,)
        });
        let output = output.as_ref().map(|ty| quote!(->#ty));
        tokens.extend(quote!(#unsafety #abi fn #ident #generics (#receiver, #(#inputs)*) #output))
    }
}
struct SubAttr {
    inner: proc_macro2::TokenStream,
}
impl syn::parse::Parse for SubAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        syn::parenthesized!(content in input);
        Ok(SubAttr {
            inner: content.parse()?,
        })
    }
}
impl<'a> From<(&'a mut syn::ItemTrait, bool)> for DynTraitDescription<'a> {
    fn from(
        (
            syn::ItemTrait {
                vis,
                ident,
                generics,
                brace_token: _,
                items,
                attrs,
                ..
            },
            checked,
        ): (&'a mut syn::ItemTrait, bool),
    ) -> Self {
        let mut this = DynTraitDescription {
            ident,
            vis,
            generics,
            functions: Vec::new(),
            mut_functions: Vec::new(),
            bounds: Default::default(),
            vt_attrs: Vec::new(),
            self_dependent_types: Default::default(),
            check_bounds: checked,
        };
        attrs.retain(|attr| {
            let mut path_segments = attr.path.segments.iter();
            if path_segments.next().map_or(true, |s| s.ident != "stabby") {
                return true;
            }
            if path_segments.next().map_or(true, |s| s.ident != "vt_attr") {
                return true;
            }
            let vt_attr = syn::parse2::<SubAttr>(attr.tokens.clone()).unwrap();
            this.vt_attrs.push(vt_attr.inner);
            false
        });
        for item in items {
            match item {
                syn::TraitItem::Method(method) => {
                    let TraitItemMethod {
                        sig:
                            Signature {
                                asyncness,
                                unsafety,
                                ident,
                                generics,
                                inputs,
                                variadic,
                                output,
                                abi,
                                ..
                            },
                        ..
                    } = method;
                    if asyncness.is_some() {
                        panic!("stabby doesn't support async functions");
                    }
                    if variadic.is_some() {
                        panic!("stabby doesn't support variadics");
                    }
                    if generics
                        .params
                        .iter()
                        .any(|g| !matches!(g, syn::GenericParam::Lifetime(_)))
                    {
                        panic!("generic methods are not trait object safe")
                    }
                    let abi = match abi {
                        Some(syn::Abi { name: None, .. }) => {
                            quote!(extern "C")
                        }
                        Some(syn::Abi {
                            name: Some(name), ..
                        }) if [
                            "C", "system", "stdcall", "aapcs", "cdecl", "fastcall", "win64",
                            "sysv64", "C-unwind", "system-unwind"
                        ]
                        .contains(&name.value().as_str()) =>
                        {
                            quote!(#abi)
                        }
                        _ => panic!("stabby trait functions must use a stable calling convention, `{ident}` doesn't"),
                    };
                    let mut inputs = inputs.iter();
                    let receiver = match inputs.next() {
                        Some(syn::FnArg::Receiver(Receiver {
                            reference: Some(reference),
                            mutability,
                            self_token,
                            ..
                        })) => Receiver {
                            reference: Some(reference.clone()),
                            mutability: *mutability,
                            self_token: *self_token,
                            attrs: Vec::new(),
                        },
                        _ => panic!(
                            "methods must take &self or &mut self as first arg to be trait safe"
                        ),
                    };
                    let inputs = inputs.map(|input| match input {
                        syn::FnArg::Typed(PatType { ty, .. }) => ty.as_ref().into(),
                        _ => panic!("Receivers are only legal in first argument position"),
                    });
                    (if receiver.mutability.is_some() {
                        &mut this.mut_functions
                    } else {
                        &mut this.functions
                    })
                    .push(DynTraitFn {
                        ident,
                        generics,
                        unsafety: *unsafety,
                        abi,
                        receiver,
                        inputs: inputs.collect(),
                        output: if let syn::ReturnType::Type(_, ty) = output {
                            Some(ty.as_ref().into())
                        } else {
                            None
                        },
                    })
                }
                syn::TraitItem::Type(ty) => {
                    let TraitItemType {
                        ident,
                        generics,
                        bounds,
                        ..
                    } = ty;
                    let ty = Ty::Path {
                        segment: ident.clone(),
                        arguments: if generics.params.is_empty() {
                            Arguments::None
                        } else {
                            Arguments::AngleBracketed {
                                generics: generics
                                    .params
                                    .iter()
                                    .map(|g| match g {
                                        syn::GenericParam::Lifetime(_) => todo!("GALs are not supported YET"),
                                        syn::GenericParam::Type(_)
                                        | syn::GenericParam::Const(_) => {
                                            panic!("stabby dyn-traits do not support GATs, except for lifetimes")
                                        }
                                    })
                                    .collect(),
                            }
                        },
                        next: None,
                    };
                    for bound in bounds {
                        let target = Ty::SelfReferencial(Box::new(ty.clone()));
                        match bound {
                            syn::TypeParamBound::Trait(syn::TraitBound {
                                lifetimes,
                                path:
                                    syn::Path {
                                        leading_colon,
                                        segments,
                                    },
                                ..
                            }) => {
                                let mut bound = Ty::from_iter(segments.iter());
                                if leading_colon.is_some() {
                                    bound = Ty::LeadingColon {
                                        next: Box::new(bound),
                                    };
                                }
                                this.bounds.push(DynTraitBound {
                                    target,
                                    lifetimes: lifetimes.clone(),
                                    bound: Ok(bound),
                                })
                            }
                            syn::TypeParamBound::Lifetime(l) => this.bounds.push(DynTraitBound {
                                target,
                                lifetimes: None,
                                bound: Err(l.clone()),
                            }),
                        }
                    }
                    this.self_dependent_types.push(ty);
                }
                syn::TraitItem::Const(_) => panic!("associated consts are not trait object safe"),
                syn::TraitItem::Macro(_) => panic!("sabby can't see through macros in traits"),
                syn::TraitItem::Verbatim(tt) => {
                    panic!("stabby failed to parse this token stream {}", tt)
                }
                _ => panic!("unexpected element in trait"),
            }
        }
        this.self_dependent_types.extend(
            this.functions
                .iter()
                .chain(&this.mut_functions)
                .flat_map(|f| f.self_dependent_types()),
        );
        this
    }
}
impl DynTraitFn<'_> {
    fn self_dependent_types(&self) -> Vec<Ty> {
        let mut sdts = self
            .output
            .as_ref()
            .map_or(Vec::new(), |t| t.self_dependent_types());
        for input in &self.inputs {
            input.rec_self_dependent_types(&mut sdts);
        }
        sdts
    }
    fn ptr_signature(&self, self_as_trait: &TokenStream) -> TokenStream {
        let Self {
            ident: _,
            generics,
            abi,
            unsafety,
            receiver:
                Receiver {
                    reference: Some((_, lt)),
                    mutability,
                    ..
                },
            inputs,
            output,
        } = self
        else {
            unreachable!()
        };
        let forgen = (!generics.params.is_empty()).then(|| quote!(for));
        let receiver = quote!(& #lt #mutability StabbyArbitraryType);
        let output = output.as_ref().map(|ty| {
            let ty = ty.replace_self(self_as_trait);
            quote!(-> #ty)
        });
        let inputs = inputs.iter().map(|ty| ty.replace_self(self_as_trait));
        quote!(#forgen #generics #abi #unsafety fn(#receiver, #(#inputs),*) #output)
    }
    fn stability_cond(&self, sdts: &SelfDependentTypes) -> TokenStream {
        let st = crate::tl_mod();
        let Self { inputs, output, .. } = self;
        let mut cond = match output {
            Some(ty) => {
                let mut uty = sdts.unselfed(ty);
                if uty == *ty {
                    uty.elide_lifetime();
                    quote!(#uty)
                } else {
                    quote!(())
                }
            }
            None => quote!(()),
        };
        for ty in inputs {
            let mut uty = sdts.unselfed(ty);
            if uty == *ty {
                uty.elide_lifetime();
                cond = quote!(#st::Union<#cond, #uty>);
            }
        }
        cond
    }
    fn field_signature(&self, sdts: &SelfDependentTypes) -> TokenStream {
        let Self {
            ident: _,
            generics,
            abi,
            unsafety,
            receiver:
                Receiver {
                    reference: Some((_, lt)),
                    mutability,
                    ..
                },
            inputs,
            output,
        } = self
        else {
            unreachable!()
        };
        let receiver = quote!(& #lt #mutability ());
        let inputs = inputs.iter().map(|ty| sdts.unselfed(ty));
        let output = output.as_ref().map(|ty| {
            let ty = sdts.unselfed(ty);
            quote!(-> #ty)
        });
        let forgen = (!generics.params.is_empty()).then(|| quote!(for));
        quote!(#forgen #generics #abi #unsafety fn(#receiver, #(#inputs),*) #output)
    }
}
impl<'a> DynTraitDescription<'a> {
    fn vtid(&self) -> Ident {
        quote::format_ident!("StabbyVtable{}", self.ident)
    }
    fn vt_generics<const BOUNDED: bool>(&self) -> TokenStream {
        let mut generics = quote!();
        let mut sdt = Some(&self.self_dependent_types);
        for generic in &self.generics.params {
            generics = match generic {
                syn::GenericParam::Lifetime(lt) => quote!(#generics #lt, ),
                syn::GenericParam::Type(ty) => {
                    if let Some(sdt) = sdt.take() {
                        let sdts = sdt.unselfed_iter();
                        generics = quote!(#generics #(#sdts,)* )
                    }
                    if BOUNDED {
                        quote!(#generics #ty, )
                    } else {
                        let ty = &ty.ident;
                        quote!(#generics #ty, )
                    }
                }
                syn::GenericParam::Const(c) => {
                    quote!(#generics #c, )
                }
            };
        }
        if let Some(sdt) = sdt.take() {
            let sdts = sdt.unselfed_iter();
            generics = quote!(#generics #(#sdts,)* )
        }
        generics
    }
    fn vtable(&self) -> TokenStream {
        let st = crate::tl_mod();
        let vtid = self.vtid();
        let vis = self.vis;
        let vt_generics = self.vt_generics::<true>();
        let nbvt_generics = self.vt_generics::<false>();
        let vt_attrs = &self.vt_attrs;
        let vt_bounds = self
            .bounds
            .iter()
            .map(
                |DynTraitBound {
                     target,
                     lifetimes,
                     bound,
                 }| {
                    let target = self.self_dependent_types.unselfed(target);
                    match bound {
                        Ok(bound) => {
                            let bound = self.self_dependent_types.unselfed(bound);
                            quote!(#target: #lifetimes #bound,)
                        }
                        Err(l) => quote!(#target: #l),
                    }
                },
            )
            .collect::<Vec<_>>();
        let fns = &self.functions;
        let mut_fns = &self.mut_functions;
        let fn_args = fns
            .iter()
            .map(|f| {
                let mut r = quote!();
                for i in 0..f.inputs.len() {
                    let id = quote::format_ident!("_{i}");
                    r = quote!(#r #id,)
                }
                r
            })
            .collect::<Vec<_>>();
        let mut_fn_args = mut_fns
            .iter()
            .map(|f| {
                let mut r = quote!();
                for i in 0..f.inputs.len() {
                    let id = quote::format_ident!("_{i}");
                    r = quote!(#r #id,)
                }
                r
            })
            .collect::<Vec<_>>();
        let fn_ids = fns.iter().map(|f| f.ident).collect::<Vec<_>>();
        let mut_fn_ids = mut_fns.iter().map(|f| f.ident).collect::<Vec<_>>();
        let mut all_fn_ids = fn_ids.clone();
        all_fn_ids.extend(mut_fn_ids.iter().copied());
        let all_fn_fts = self
            .functions
            .iter()
            .chain(&self.mut_functions)
            .map(|f| f.field_signature(&self.self_dependent_types))
            .collect::<Vec<_>>();
        let all_fn_conds = self
            .functions
            .iter()
            .chain(&self.mut_functions)
            .map(|f| f.stability_cond(&self.self_dependent_types));
        let all_stabled_fns = all_fn_conds.zip(all_fn_fts).map(|(cond, fn_ft)| {
            let ret = quote!(#st::StableLike<#fn_ft, &'static ()>);
            if self.check_bounds {
                quote!(#st::StableIf<#ret, #cond>)
            } else {
                ret
            }
        });
        let trait_id = self.ident;
        let trait_generics = &self.generics.params;
        let trait_lts = trait_generics.lifetimes().collect::<Vec<_>>();
        let trait_types = trait_generics.types().collect::<Vec<_>>();
        let trait_consts = trait_generics.consts().collect::<Vec<_>>();
        let unbound_trait_lts = trait_generics.lifetimes().unbound().collect::<Vec<_>>();
        let unbound_trait_types = trait_generics.types().unbound().collect::<Vec<_>>();
        let unbound_trait_consts = trait_generics.consts().unbound().collect::<Vec<_>>();
        let dyntrait_types = self
            .self_dependent_types
            .unselfed_iter()
            .collect::<Vec<_>>();
        let sdts: &[_] = &self.self_dependent_types;
        let sdtbounds = sdts.iter().map(|ty| {
            self.bounds
                .iter()
                .fold(
                    None,
                    |acc,
                     DynTraitBound {
                         target,
                         lifetimes,
                         bound,
                     }| {
                        let Ty::SelfReferencial(target) = target else {
                            return acc;
                        };
                        if &**target == ty {
                            let bound = match bound {
                                Ok(bound) => quote!(#lifetimes #bound),
                                Err(lt) => quote!(#lt),
                            };
                            Some(match acc {
                                Some(acc) => quote!(#acc + #bound),
                                None => quote!(#bound),
                            })
                        } else {
                            acc
                        }
                    },
                )
                .map(|bounds| quote!(: #bounds))
        });
        let trait_to_vt_bindings = self
            .self_dependent_types
            .iter()
            .enumerate()
            .map(|(n, gen)| {
                let binding = SelfDependentTypes::unselfed_n(n);
                quote!(#gen = #binding)
            })
            .collect::<Vec<_>>();

        let vt_signature = quote! {
            #vtid <
                        #(#unbound_trait_lts,)*
                        #(#dyntrait_types,)*
                        #(#unbound_trait_types,)*
                    >
        };
        let self_as_trait = quote!(<StabbyArbitraryType as #trait_id <#(#unbound_trait_lts,)* #(#unbound_trait_types,)* #(#unbound_trait_consts,)*>>);
        let fn_ptrs = self
            .functions
            .iter()
            .chain(&self.mut_functions)
            .map(|f| f.ptr_signature(&self_as_trait))
            .collect::<Vec<_>>();
        let traitid_dyn = quote::format_ident!("{}Dyn", trait_id);
        let traitid_dynmut = quote::format_ident!("{}DynMut", trait_id);

        let vt_doc = format!("An stabby-generated item for [`{}`]", trait_id);
        let mut vtable_decl = quote! {
            #(#[#vt_attrs])*
            #vis struct #vtid < #vt_generics > where #(#vt_bounds)* {
                #(
                    #[doc = #vt_doc]
                    pub #all_fn_ids: #all_stabled_fns,
                )*
            }
        };
        let vtid_str = vtid.to_string();
        let all_fn_ids_str = all_fn_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>();
        vtable_decl = crate::stabby(proc_macro::TokenStream::new(), vtable_decl.into()).into();
        quote! {
            #[doc = #vt_doc]
            #vtable_decl
            impl< #vt_generics > Clone for #vtid < #nbvt_generics > where #(#vt_bounds)* {
                fn clone(&self) -> Self {
                    *self
                }
            }
            impl< #vt_generics > Copy for #vtid < #nbvt_generics > where #(#vt_bounds)* {}
            impl< #vt_generics > core::cmp::PartialEq for #vtid < #nbvt_generics > where #(#vt_bounds)*{
                fn eq(&self, other: &Self) -> bool {
                    #(core::ptr::eq((*unsafe{self.#all_fn_ids.as_ref_unchecked()}) as *const (), (*unsafe{other.#all_fn_ids.as_ref_unchecked()}) as *const _) &&)* true
                }
            }
            impl< #vt_generics > core::hash::Hash for #vtid < #nbvt_generics > where #(#vt_bounds)*{
                fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
                    #(self.#all_fn_ids.hash(state);)*
                }
            }
            impl< #vt_generics > core::fmt::Debug for #vtid < #nbvt_generics > where #(#vt_bounds)*{
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    let mut s = f.debug_struct(#vtid_str);
                    #(s.field(#all_fn_ids_str, &core::format_args!("{:p}", unsafe{self.#all_fn_ids.as_ref_unchecked()}));)*
                    s.finish()
                }
            }

            impl<
                'stabby_vt_lt, #(#trait_lts,)*
                StabbyArbitraryType,
                #(#dyntrait_types,)*
                #(#trait_types,)*
                #(#trait_consts,)*
            > #st::vtable::IConstConstructor<'stabby_vt_lt, StabbyArbitraryType> for #vt_signature
            where
                StabbyArbitraryType: #trait_id <#(#unbound_trait_lts,)* #(#unbound_trait_types,)* #(#unbound_trait_consts,)* #(#trait_to_vt_bindings,)* >,
                #(#vt_bounds)*
                #(#unbound_trait_types: 'stabby_vt_lt,)*
                #(#dyntrait_types: 'stabby_vt_lt,)* {
                #[doc = #vt_doc]
                #st::impl_vtable_constructor!(
                    const VTABLE_REF: &'stabby_vt_lt #vt_signature =  &#vtid {
                        #(#all_fn_ids: unsafe {core::mem::transmute(#self_as_trait::#all_fn_ids as #fn_ptrs)},)*
                    };=>
                    const VTABLE: #vt_signature =  #vtid {
                        #(#all_fn_ids: unsafe {core::mem::transmute(#self_as_trait::#all_fn_ids as #fn_ptrs)},)*
                    };
                );
            }

            impl<
                'stabby_vt_lt, #(#trait_lts,)*
                #(#dyntrait_types,)*
                #(#trait_types,)*
                #(#trait_consts,)*
            > #st::vtable::CompoundVt for dyn #trait_id <#(#unbound_trait_lts,)* #(#unbound_trait_types,)* #(#unbound_trait_consts,)* #(#trait_to_vt_bindings,)* > where #(#vt_bounds)* {
                #[doc = #vt_doc]
                type Vt<StabbyNextVtable> = #st::vtable::VTable<
                    #vt_signature,
                    StabbyNextVtable>;
            }

            #[doc = #vt_doc]
            #vis trait #traitid_dyn<
                #(#trait_lts,)*
                StabbyTransitiveDerefN,
                #(#dyntrait_types,)*
                #(#trait_types,)*
                #(#trait_consts,)*
            > where #(#vt_bounds)* {
                #(
                    #[doc = #vt_doc]
                    type #sdts #sdtbounds;
                )*
                #(
                    #[doc = #vt_doc]
                    #fns;
                )*
            }
            impl<
                #(#trait_lts,)*
                StabbyVtProvider: #st::vtable::TransitiveDeref<
                    #vt_signature,
                    StabbyTransitiveDerefN
                    > + Copy,
                StabbyTransitiveDerefN,
                #(#dyntrait_types,)*
                #(#trait_types,)*
                #(#trait_consts,)*
            >
            #traitid_dyn <
                #(#unbound_trait_lts,)*
                StabbyTransitiveDerefN,
                #(#dyntrait_types,)*
                #(#unbound_trait_types,)*
                #(#unbound_trait_consts,)*
            >
            for #st::DynRef<'_, StabbyVtProvider> where #(#vt_bounds)*
            {
                #(
                    #[doc = #vt_doc]
                    type #trait_to_vt_bindings;
                )*
                #(
                    #[doc = #vt_doc]
                    #fns {
                        unsafe{(self.vtable().tderef().#fn_ids.as_ref_unchecked())(self.ptr(), #fn_args)}
                    }
                )*
            }
            impl<
                #(#trait_lts,)*
                StabbyPtrProvider: #st::IPtrOwned + #st::IPtr,
                StabbyVtProvider: #st::vtable::HasDropVt + Copy + #st::vtable::TransitiveDeref<
                    #vt_signature,
                    StabbyTransitiveDerefN
                    >,
                StabbyTransitiveDerefN,
                #(#dyntrait_types,)*
                #(#trait_types,)*
                #(#trait_consts,)*
            >
            #traitid_dyn <
                #(#unbound_trait_lts,)*
                StabbyTransitiveDerefN,
                #(#dyntrait_types,)*
                #(#unbound_trait_types,)*
                #(#unbound_trait_consts,)*
            >
            for #st::Dyn<'_, StabbyPtrProvider, StabbyVtProvider> where #(#vt_bounds)*
            {
                #(
                    #[doc = #vt_doc]
                    type #trait_to_vt_bindings;
                )*
                #(
                    #[doc = #vt_doc]
                    #fns {
                        unsafe{(self.vtable().tderef().#fn_ids.as_ref_unchecked())(self.ptr().as_ref(), #fn_args)}
                    }
                )*
            }


            #[doc = #vt_doc]
            #vis trait #traitid_dynmut<
                #(#trait_lts,)*
                StabbyTransitiveDerefN,
                #(#dyntrait_types,)*
                #(#trait_types,)*
                #(#trait_consts,)*
            >: #traitid_dyn <
                #(#unbound_trait_lts,)*
                StabbyTransitiveDerefN,
                #(#dyntrait_types,)*
                #(#unbound_trait_types,)*
                #(#unbound_trait_consts,)*
            > where #(#vt_bounds)* {
                #(
                    #[doc = #vt_doc]
                    #mut_fns;
                )*
            }
            impl<
                #(#trait_lts,)*
                StabbyPtrProvider: #st::IPtrOwned + #st::IPtrMut,
                StabbyVtProvider: #st::vtable::HasDropVt + Copy + #st::vtable::TransitiveDeref<
                    #vt_signature,
                    StabbyTransitiveDerefN
                    >,
                StabbyTransitiveDerefN,
                #(#dyntrait_types,)*
                #(#trait_types,)*
                #(#trait_consts,)*
            >
            #traitid_dynmut <
                #(#unbound_trait_lts,)*
                StabbyTransitiveDerefN,
                #(#dyntrait_types,)*
                #(#unbound_trait_types,)*
                #(#unbound_trait_consts,)*
            >
            for #st::Dyn<'_, StabbyPtrProvider, StabbyVtProvider> where #(#vt_bounds)*
            {
                #(
                    #[doc = #vt_doc]
                    #mut_fns {
                        unsafe {(self.vtable().tderef().#mut_fn_ids.as_ref_unchecked())(self.ptr_mut().as_mut(), #mut_fn_args)}
                    }
                )*
            }
        }
    }
}

#[derive(Clone)]
enum Ty {
    Never,
    Unit,
    Arbitrary {
        prefix: TokenStream,
        next: Box<Self>,
    },
    SelfReferencial(Box<Self>),
    Reference {
        lifetime: Option<Lifetime>,
        mutability: Option<Mut>,
        elem: Box<Self>,
    },
    Ptr {
        const_token: Option<Const>,
        mutability: Option<Mut>,
        elem: Box<Self>,
    },
    Array {
        elem: Box<Self>,
        len: Expr,
    },
    BareFn {
        lifetimes: Option<BoundLifetimes>,
        unsafety: Option<Unsafe>,
        abi: Option<Abi>,
        inputs: Vec<Self>,
        output: Box<Self>,
    },
    Path {
        segment: Ident,
        arguments: Arguments,
        next: Option<Box<Self>>,
    },
    LeadingColon {
        next: Box<Self>,
    },
    Qualified {
        target: Box<Self>,
        as_trait: Box<Self>,
        next: Box<Self>,
    },
}
impl PartialEq for Ty {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Arbitrary {
                    prefix: l_prefix,
                    next: l_next,
                },
                Self::Arbitrary {
                    prefix: r_prefix,
                    next: r_next,
                },
            ) => l_prefix.to_string() == r_prefix.to_string() && l_next == r_next,
            (Self::SelfReferencial(l0), Self::SelfReferencial(r0)) => l0 == r0,
            (
                Self::Reference {
                    lifetime: l_lifetime,
                    mutability: l_mutability,
                    elem: l_elem,
                },
                Self::Reference {
                    lifetime: r_lifetime,
                    mutability: r_mutability,
                    elem: r_elem,
                },
            ) => {
                l_lifetime == r_lifetime
                    && l_mutability.is_some() == r_mutability.is_some()
                    && l_elem == r_elem
            }
            (
                Self::Ptr {
                    const_token: l_const_token,
                    mutability: l_mutability,
                    elem: l_elem,
                },
                Self::Ptr {
                    const_token: r_const_token,
                    mutability: r_mutability,
                    elem: r_elem,
                },
            ) => {
                l_const_token.is_some() == r_const_token.is_some()
                    && l_mutability.is_some() == r_mutability.is_some()
                    && l_elem == r_elem
            }
            (Self::Array { .. }, _) | (Self::BareFn { .. }, _) => false,
            (
                Self::Path {
                    segment: l_segment,
                    arguments: _,
                    next: l_next,
                },
                Self::Path {
                    segment: r_segment,
                    arguments: _,
                    next: r_next,
                },
            ) => l_segment == r_segment && l_next == r_next,
            (Self::LeadingColon { next: l_next }, Self::LeadingColon { next: r_next }) => {
                l_next == r_next
            }
            (
                Self::Qualified {
                    target: l_target,
                    as_trait: l_as_trait,
                    next: l_next,
                },
                Self::Qualified {
                    target: r_target,
                    as_trait: r_as_trait,
                    next: r_next,
                },
            ) => l_target == r_target && l_as_trait == r_as_trait && l_next == r_next,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}
impl core::fmt::Display for Ty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", quote!(#self))
    }
}
#[derive(Clone)]
enum Arguments {
    None,
    AngleBracketed { generics: Vec<GenericArgument> },
}
#[derive(Clone)]
enum GenericArgument {
    Lifetime(syn::Lifetime),
    Type(Ty),
    Const(syn::Expr),
    Binding(syn::Binding),
    Constraint(syn::Constraint),
}
impl quote::ToTokens for GenericArgument {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            GenericArgument::Lifetime(l) => l.to_tokens(tokens),
            GenericArgument::Type(ty) => ty.to_tokens(tokens),
            GenericArgument::Const(l) => l.to_tokens(tokens),
            GenericArgument::Binding(l) => l.to_tokens(tokens),
            GenericArgument::Constraint(l) => l.to_tokens(tokens),
        }
    }
}
impl quote::ToTokens for Arguments {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Arguments::None => {}
            Arguments::AngleBracketed { generics } => tokens.extend(quote!(::<#(#generics,)*>)),
        }
    }
}
impl quote::ToTokens for Ty {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Never => tokens.extend(quote!(!)),
            Self::Unit => tokens.extend(quote!(())),
            Self::Arbitrary { prefix, next } => tokens.extend(quote!(#prefix::#next)),
            Self::SelfReferencial(ty) => tokens.extend(quote!(Self::#ty)),
            Self::Reference {
                lifetime,
                mutability,
                elem,
            } => tokens.extend(quote!(& #lifetime #mutability #elem)),
            Self::Ptr {
                const_token,
                mutability,
                elem,
            } => tokens.extend(quote!(*#const_token #mutability #elem)),
            Self::Array { elem, len } => tokens.extend(quote!([#elem;#len])),
            Self::BareFn {
                lifetimes,
                unsafety,
                abi,
                inputs,
                output,
            } => tokens.extend(
                quote!(#lifetimes #unsafety # abi fn(#(#inputs,)*) -> #output
                ),
            ),
            Self::Path {
                segment,
                arguments,
                next,
            } => {
                let mut t = quote!(#segment #arguments);
                if let Some(next) = next {
                    t.extend(quote!(::#next))
                }
                tokens.extend(t)
            }
            Self::LeadingColon { next } => tokens.extend(quote!(::#next)),
            Self::Qualified {
                target,
                as_trait,
                next,
            } => tokens.extend(quote!(<#target as #as_trait>::#next)),
        }
    }
}
impl Ty {
    fn elide_lifetime(&mut self) {
        match self {
            Ty::Never | Ty::Unit => {}
            Ty::Reference { lifetime, elem, .. } => {
                if let Some(lt) = lifetime {
                    *lt = syn::Lifetime::new("'static", Span::call_site())
                }
                elem.elide_lifetime();
            }
            Ty::Array { elem, .. }
            | Ty::Ptr { elem, .. }
            | Ty::LeadingColon { next: elem }
            | Ty::Arbitrary { next: elem, .. }
            | Ty::SelfReferencial(elem) => elem.elide_lifetime(),
            Ty::BareFn { .. } => {}
            Ty::Path {
                arguments, next, ..
            } => {
                if let Some(e) = next {
                    e.elide_lifetime()
                }
                if let Arguments::AngleBracketed { generics } = arguments {
                    for arg in generics {
                        match arg {
                            GenericArgument::Lifetime(lt) => {
                                *lt = syn::Lifetime::new("'static", Span::call_site())
                            }
                            GenericArgument::Type(ty) => ty.elide_lifetime(),
                            _ => {}
                        }
                    }
                }
            }
            Ty::Qualified {
                target,
                as_trait,
                next,
            } => {
                target.elide_lifetime();
                as_trait.elide_lifetime();
                next.elide_lifetime();
            }
        }
    }
    fn from_iter<'a, T: Iterator<Item = &'a PathSegment> + ExactSizeIterator>(mut iter: T) -> Self {
        let PathSegment { ident, arguments } = iter.next().unwrap();
        if *ident == "Self" {
            return Self::SelfReferencial(Box::new(Self::from_iter(iter)));
        }
        let arguments = match arguments {
            PathArguments::None => Arguments::None,
            PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) => {
                let mut generics = Vec::new();
                for arg in args {
                    generics.push(match arg {
                        syn::GenericArgument::Lifetime(l) => GenericArgument::Lifetime(l.clone()),
                        syn::GenericArgument::Type(ty) => GenericArgument::Type(Self::from(ty)),
                        syn::GenericArgument::Const(c) => GenericArgument::Const(c.clone()),
                        syn::GenericArgument::Binding(b) => GenericArgument::Binding(b.clone()),
                        syn::GenericArgument::Constraint(c) => {
                            GenericArgument::Constraint(c.clone())
                        }
                    })
                }
                Arguments::AngleBracketed { generics }
            }
            PathArguments::Parenthesized(_) => todo!(),
        };
        Self::Path {
            segment: ident.clone(),
            arguments,
            next: (iter.len() != 0).then(move || Box::new(Self::from_iter(iter))),
        }
    }
    fn rec_self_dependent_types(&self, sdts: &mut Vec<Ty>) {
        match self {
            Ty::SelfReferencial(ty) => sdts.push(ty.as_ref().clone()),
            Ty::Never | Ty::Unit => {}
            Ty::LeadingColon { next: elem }
            | Ty::Arbitrary { next: elem, .. }
            | Ty::Reference { elem, .. }
            | Ty::Ptr { elem, .. }
            | Ty::Array { elem, .. } => elem.rec_self_dependent_types(sdts),
            Ty::BareFn { inputs, output, .. } => {
                output.rec_self_dependent_types(sdts);
                for i in inputs {
                    i.rec_self_dependent_types(sdts);
                }
            }
            Ty::Path {
                arguments, next, ..
            } => {
                match arguments {
                    Arguments::None => {}
                    Arguments::AngleBracketed { generics } => {
                        for g in generics {
                            if let GenericArgument::Type(ty) = g {
                                ty.rec_self_dependent_types(sdts);
                            }
                        }
                    }
                }
                if let Some(next) = next {
                    next.rec_self_dependent_types(sdts)
                }
            }
            Ty::Qualified {
                target,
                as_trait,
                next,
            } => {
                target.rec_self_dependent_types(sdts);
                as_trait.rec_self_dependent_types(sdts);
                next.rec_self_dependent_types(sdts);
            }
        }
    }
}
impl<'a> From<&'a Type> for Ty {
    fn from(value: &'a Type) -> Self {
        match value {
            Type::Path(TypePath {
                qself: None,
                path:
                    Path {
                        leading_colon,
                        segments,
                    },
            }) => {
                let this = Self::from_iter(segments.iter());
                if leading_colon.is_some() {
                    Self::LeadingColon {
                        next: Box::new(this),
                    }
                } else {
                    this
                }
            }
            Type::Path(TypePath {
                qself: Some(syn::QSelf { ty, position, .. }),
                path,
            }) => {
                let mut iter = path.segments.iter();
                Self::Qualified {
                    target: Box::new(ty.as_ref().into()),
                    as_trait: Box::new(Self::from_iter(iter.by_ref().take(*position))),
                    next: Box::new(Self::from_iter(iter)),
                }
            }
            Type::BareFn(TypeBareFn {
                lifetimes,
                unsafety,
                abi,
                inputs,
                variadic: None,
                output,
                ..
            }) => Self::BareFn {
                lifetimes: lifetimes.clone(),
                unsafety: *unsafety,
                abi: abi.clone(),
                inputs: inputs.iter().map(|i| (&i.ty).into()).collect(),
                output: Box::new(match output {
                    syn::ReturnType::Default => Self::Unit,
                    syn::ReturnType::Type(_, ty) => ty.as_ref().into(),
                }),
            },
            Type::Group(TypeGroup { elem, .. }) => elem.as_ref().into(),
            Type::Paren(TypeParen { .. }) => todo!("Type::Paren not supported yet"),
            Type::Array(TypeArray { elem, len, .. }) => Self::Array {
                elem: Box::new(elem.as_ref().into()),
                len: len.clone(),
            },
            Type::Ptr(TypePtr {
                const_token,
                mutability,
                elem,
                ..
            }) => Self::Ptr {
                const_token: *const_token,
                mutability: *mutability,
                elem: Box::new(elem.as_ref().into()),
            },
            Type::Reference(TypeReference {
                lifetime,
                mutability,
                elem,
                ..
            }) => Self::Reference {
                mutability: *mutability,
                lifetime: lifetime.clone(),
                elem: Box::new(elem.as_ref().into()),
            },
            Type::Tuple(TypeTuple { elems, .. }) if elems.is_empty() => Self::Unit,
            Type::Never(_) => Self::Never,
            Type::BareFn(TypeBareFn {
                variadic: Some(_), ..
            }) => panic!("stabby doesn't support variadic functions"),
            Type::Verbatim(t) => panic!("stabby couldn't parse {t}"),
            Type::Macro(_) => panic!(
                "stabby couldn't see through a macro in type position. Try using a type alias :)"
            ),
            Type::Infer(_) => panic!("type inference is not available in trait definitions"),
            Type::ImplTrait(_) => panic!("generic methods are not trait object safe"),
            Type::Slice(_) => panic!("slices are not ABI stable"),
            Type::TraitObject(_) => panic!("trait objects are not ABI stable"),
            Type::Tuple(_) => panic!("tuples are not ABI stable"),
            _ => {
                panic!("unknown element type in {}", quote!(#value))
            }
        }
    }
}
impl Ty {
    fn self_dependent_types(&self) -> Vec<Ty> {
        let mut sdts = Vec::new();
        self.rec_self_dependent_types(&mut sdts);
        sdts
    }
    fn replace_self(&self, with: &TokenStream) -> Self {
        match self {
            Ty::Never => Ty::Never,
            Ty::Unit => Ty::Unit,
            Ty::Arbitrary { prefix, next } => Ty::Arbitrary {
                prefix: prefix.clone(),
                next: Box::new(next.replace_self(with)),
            },
            Ty::SelfReferencial(ty) => Ty::Arbitrary {
                prefix: with.clone(),
                next: ty.clone(),
            },
            Ty::Reference {
                lifetime,
                mutability,
                elem,
            } => Ty::Reference {
                lifetime: lifetime.clone(),
                mutability: *mutability,
                elem: Box::new(elem.replace_self(with)),
            },
            Ty::Ptr {
                const_token,
                mutability,
                elem,
            } => Ty::Ptr {
                const_token: *const_token,
                mutability: *mutability,
                elem: Box::new(elem.replace_self(with)),
            },
            Ty::Array { elem, len } => Ty::Array {
                elem: Box::new(elem.replace_self(with)),
                len: len.clone(),
            },
            Ty::BareFn {
                lifetimes,
                unsafety,
                abi,
                inputs,
                output,
            } => Ty::BareFn {
                lifetimes: lifetimes.clone(),
                unsafety: *unsafety,
                abi: abi.clone(),
                inputs: inputs.iter().map(|elem| elem.replace_self(with)).collect(),
                output: Box::new(output.replace_self(with)),
            },
            Ty::Path {
                segment,
                arguments,
                next,
            } => Ty::Path {
                segment: segment.clone(),
                arguments: match arguments {
                    Arguments::None => Arguments::None,
                    Arguments::AngleBracketed { generics } => Arguments::AngleBracketed {
                        generics: generics
                            .iter()
                            .map(|g| match g {
                                GenericArgument::Type(ty) => {
                                    GenericArgument::Type(ty.replace_self(with))
                                }
                                g => g.clone(),
                            })
                            .collect(),
                    },
                },
                next: next.as_ref().map(|elem| Box::new(elem.replace_self(with))),
            },
            Ty::LeadingColon { next } => Ty::LeadingColon {
                next: Box::new(next.replace_self(with)),
            },
            Ty::Qualified {
                target,
                as_trait,
                next,
            } => Ty::Qualified {
                target: Box::new(target.replace_self(with)),
                as_trait: Box::new(as_trait.replace_self(with)),
                next: Box::new(next.replace_self(with)),
            },
        }
    }
}
pub fn stabby(
    mut item_trait: syn::ItemTrait,
    stabby_attrs: &proc_macro::TokenStream,
) -> TokenStream {
    let checked = match stabby_attrs.to_string().as_str() {
        "checked" => true,
        "" => false,
        _ => panic!("Unkown stabby attributes {stabby_attrs}"),
    };
    let description: DynTraitDescription = (&mut item_trait, checked).into();
    let vtable = description.vtable();
    quote! {
        #[deny(improper_ctypes_definitions)]
        #item_trait
        #vtable
    }
}
