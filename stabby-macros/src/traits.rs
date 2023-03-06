use std::ops::Deref;

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{
    token::{Const, Mut, Unsafe},
    Abi, AngleBracketedGenericArguments, BoundLifetimes, Expr, Lifetime,
    ParenthesizedGenericArguments, PatType, Path, PathArguments, PathSegment, QSelf, Receiver,
    Signature, TraitItemMethod, TraitItemType, Type, TypeArray, TypeBareFn, TypeGroup, TypeParen,
    TypePath, TypePtr, TypeReference, TypeTuple,
};
#[derive(Clone, Default)]
struct SelfDependentTypes {
    inner: Vec<Type>,
}
impl SelfDependentTypes {
    fn push(&mut self, ty: Type) -> bool {
        if self.find(&ty).is_none() {
            self.inner.push(ty);
            true
        } else {
            false
        }
    }
    fn extend(&mut self, it: impl Iterator<Item = Type>) {
        for ty in it {
            self.push(ty);
        }
    }
    fn find(&self, ty: &Type) -> Option<usize> {
        let tystr = quote!(#ty).to_string();
        self.iter().position(|t| quote!(#t).to_string() == tystr)
    }
    fn unselfed(&self, ty: &Ty) -> TokenStream {
        todo!()
        // let Some(i) = self.find(ty) else {return quote!(#ty)};
        // let t = quote::format_ident!("_stabby_unselfed_{i}");
        // quote!(#t)
    }
    fn unselfed_iter(&self) -> impl Iterator<Item = Ident> {
        (0..self.inner.len()).map(|i| quote::format_ident!("_stabby_unselfed_{i}"))
    }
}
impl Deref for SelfDependentTypes {
    type Target = [Type];
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
impl<'a> From<&'a syn::ItemTrait> for DynTraitDescription<'a> {
    fn from(
        syn::ItemTrait {
            vis,
            ident,
            generics,
            brace_token: _,
            items,
            ..
        }: &'a syn::ItemTrait,
    ) -> Self {
        let mut this = DynTraitDescription {
            ident,
            vis,
            generics,
            functions: Vec::new(),
            mut_functions: Vec::new(),
            self_dependent_types: Default::default(),
        };
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
                        abi: quote!(#abi),
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
                        // attrs,
                        // ident,
                        // generics,
                        // colon_token,
                        // bounds,
                        // default,
                        ..
                    } = ty;
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
    fn self_dependent_types(&self) -> Vec<Type> {
        let mut sdts = self
            .output
            .as_ref()
            .map_or(Vec::new(), |t| t.self_dependent_types());
        for input in &self.inputs {
            sdts.extend_from_slice(&input.self_dependent_types());
        }
        sdts
    }
    fn ptr_signature(&self) -> TokenStream {
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
        } = self else {unreachable!()};
        let forgen = (!generics.params.is_empty()).then(|| quote!(for));
        let receiver = quote!(& #lt #mutability Self);
        let output = output.as_ref().map(|ty| quote!(-> #ty));
        quote!(#forgen #generics #abi #unsafety fn(#receiver, #(#inputs),*) #output)
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
        } = self else {unreachable!()};
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
        let fn_ids = self
            .functions
            .iter()
            .chain(&self.mut_functions)
            .map(|f| f.ident)
            .collect::<Vec<_>>();
        let fn_fts = self
            .functions
            .iter()
            .chain(&self.mut_functions)
            .map(|f| f.field_signature(&self.self_dependent_types));
        quote! {
            #vis struct #vtid < #vt_generics > {
                #(#fn_ids: #fn_fts,)*
            }
            // impl<'stabby_vt_lt, #vt_generics > #st::vtable::IConstConstructor<'stabby_vt_lt, #vtid < #nbvt_generics >> for T: todo!() {
            //     const VTABLE: &'stabby_vt_lt todo!() = &todo!();
            // }
            // impl <todo!()> #st::vtable::CompoundVt for dyn todo!() {
            //     type Vt<T> = #st::vtable::VTable<#vtid <todo!()>, T>;
            // }
            impl< #vt_generics > Clone for #vtid < #nbvt_generics > {
                fn clone(&self) -> Self {
                    Self {
                        #(#fn_ids: self.#fn_ids,)*
                    }
                }
            }
            impl< #vt_generics > Copy for #vtid < #nbvt_generics > {}
            impl< #vt_generics > core::cmp::PartialEq for #vtid < #nbvt_generics > {
                fn eq(&self, other: &Self) -> bool {
                    #(core::ptr::eq(self.#fn_ids as *const (), other.#fn_ids as *const _) &&)* true
                }
            }
            // pub trait DynMyTrait<N, Output> {
            //     extern "C" fn do_stuff<'a>(&'a self, with: &Output) -> &'a u8;
            // }
            // impl<Vt: TransitiveDeref<VtMyTrait<Output>, N>, Output, N> DynMyTrait<N, Output>
            //     for DynRef<'_, Vt>
            // {
            //     extern "C" fn do_stuff<'a>(&'a self, with: &Output) -> &'a u8 {
            //         (self.vtable.tderef().do_stuff)(self.ptr, with)
            //     }
            // }
            // impl<'c, P: IPtrOwned, Vt: HasDropVt + TransitiveDeref<VtMyTrait<Output>, N>, Output, N>
            //     DynMyTrait<N, Output> for Dyn<'c, P, Vt>
            // {
            //     extern "C" fn do_stuff<'a>(&'a self, with: &Output) -> &'a u8 {
            //         (self.vtable.tderef().do_stuff)(unsafe { self.ptr.as_ref() }, with)
            //     }
            // }
            // pub trait DynMutMyTrait<N, Output>: DynMyTrait<N, Output> {
            //     extern "C" fn gen_stuff(&mut self) -> Output;
            // }
            // impl<
            //         'a,
            //         P: IPtrOwned + IPtrMut,
            //         Vt: HasDropVt + TransitiveDeref<VtMyTrait<Output>, N>,
            //         Output,
            //         N,
            //     > DynMutMyTrait<N, Output> for Dyn<'a, P, Vt>
            // {
            //     extern "C" fn gen_stuff(&mut self) -> Output {
            //         (self.vtable.tderef().gen_stuff)(unsafe { self.ptr.as_mut() })
            //     }
            // }
        }
    }
}
enum Ty {
    Never,
    Unit,
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
}
impl quote::ToTokens for Ty {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Ty::Never => tokens.extend(quote!(!)),
            Ty::Unit => tokens.extend(quote!(())),
            Ty::Reference {
                lifetime,
                mutability,
                elem,
            } => tokens.extend(quote!(& #lifetime #mutability #elem)),
            Ty::Ptr {
                const_token,
                mutability,
                elem,
            } => tokens.extend(quote!(*#const_token #mutability #elem)),
            Ty::Array { elem, len } => tokens.extend(quote!([#elem;#len])),
            Ty::BareFn {
                lifetimes,
                unsafety,
                abi,
                inputs,
                output,
            } => todo!(),
        }
    }
}
impl<'a> From<&'a Type> for Ty {
    fn from(value: &'a Type) -> Self {
        match value {
            Type::Path(TypePath { qself: None, path }) => todo!(),
            Type::Path(TypePath {
                qself: Some(qself),
                path,
            }) => todo!(),
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
            Type::Macro(_) => panic!("stabby couldn't see through a macro in type position"),
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
    fn self_dependent_types(&self) -> Vec<Type> {
        // match self.inner {
        //     Type::Path(TypePath {
        //         qself,
        //         path: Path { segments, .. },
        //     }) => {
        //         let sdts = Vec::new();

        //         sdts
        //     }
        //     Type::BareFn(TypeBareFn { inputs, output, .. }) => {
        //         let mut sdts = match output {
        //             syn::ReturnType::Default => Vec::new(),
        //             syn::ReturnType::Type(_, ty) => Ty::from(ty.as_ref()).self_dependent_types(),
        //         };
        //         for syn::BareFnArg { ty, .. } in inputs {
        //             sdts.extend_from_slice(&Ty::from(ty).self_dependent_types());
        //         }
        //         sdts
        //     }
        //     Type::Paren(TypeParen { elem, .. })
        //     | Type::Ptr(TypePtr { elem, .. })
        //     | Type::Reference(TypeReference { elem, .. })
        //     | Type::Array(TypeArray { elem, .. })
        //     | Type::Group(TypeGroup { elem, .. }) => Ty::from(elem.as_ref()).self_dependent_types(),
        //     Type::Macro(_) | Type::Never(_) | Type::Verbatim(_) => Vec::new(),
        //     Type::Infer(_) => panic!("type inference is not available in trait definitions"),
        //     Type::ImplTrait(_) => panic!("generic methods are not trait object safe"),
        //     Type::Slice(_) => panic!("slices are not ABI stable"),
        //     Type::TraitObject(_) => panic!("trait objects are not ABI stable"),
        //     Type::Tuple(_) => panic!("tuples are not ABI stable"),
        //     _ => {
        //         let elem = self.inner;
        //         panic!("unknown element type in {}", quote!(#elem))
        //     }
        // }
        Vec::new()
    }
}
pub fn stabby(item_trait: syn::ItemTrait) -> TokenStream {
    let description: DynTraitDescription = (&item_trait).into();
    let vtable = description.vtable();
    quote! {
        #item_trait
        #vtable
    }
}
