use std::ops::Deref;

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{
    AngleBracketedGenericArguments, ParenthesizedGenericArguments, PatType, Path, PathArguments,
    PathSegment, QSelf, Receiver, Signature, TraitItemMethod, TraitItemType, Type, TypeArray,
    TypeGroup, TypeParen, TypePath, TypePtr, TypeReference,
};
#[derive(Clone, Default)]
struct SelfDependentTypes {
    inner: Vec<Type>,
}
impl SelfDependentTypes {
    fn extend<'a>(&mut self, it: impl Iterator<Item = &'a Type>) {
        for ty in it {
            if self.find(ty).is_none() {
                self.inner.push(ty.clone())
            }
        }
    }
    fn find(&self, ty: &Type) -> Option<usize> {
        let tystr = quote!(#ty).to_string();
        self.iter().position(|t| quote!(#t).to_string() == tystr)
    }
    fn unselfed(&self, ty: &Type) -> TokenStream {
        let Some(i) = self.find(ty) else {return quote!(#ty)};
        let t = quote::format_ident!("_stabby_unselfed_{i}");
        quote!(#t)
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
    inputs: Vec<Type>,
    output: syn::ReturnType,
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
                        syn::FnArg::Typed(PatType { ty, .. }) => ty.as_ref().clone(),
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
                        output: output.clone(),
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
    fn self_dependent_types(&self) -> impl Iterator<Item = &Type> {
        let output = match &self.output {
            syn::ReturnType::Type(_, t) if t.is_assoc() => Some(t.as_ref()),
            _ => None,
        };
        self.inputs.iter().filter(|t| t.is_assoc()).chain(output)
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
        let output = match output {
            syn::ReturnType::Default => None,
            syn::ReturnType::Type(_, ty) => {
                let unselfed = sdts.unselfed(ty);
                Some(quote!(-> #unselfed))
            }
        };
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
pub fn stabby(item_trait: syn::ItemTrait) -> TokenStream {
    let st = crate::tl_mod();
    let description: DynTraitDescription = (&item_trait).into();
    let vtable = description.vtable();
    quote! {
        #item_trait
        #vtable
    }
}

trait IsAssoc {
    fn is_assoc(&self) -> bool;
}
impl IsAssoc for Type {
    fn is_assoc(&self) -> bool {
        is_assoc(self)
    }
}
fn is_assoc(elem: &Type) -> bool {
    match elem {
        Type::Path(TypePath {
            qself,
            path: Path { segments, .. },
        }) => {
            qself
                .as_ref()
                .map_or(false, |QSelf { ty, .. }| ty.is_assoc())
                || segments.iter().any(|PathSegment { ident, arguments }| {
                    if *ident == "Self" {
                        return true;
                    }
                    match arguments {
                        PathArguments::None => false,
                        PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                            args,
                            ..
                        }) => args.iter().any(|arg| match arg {
                            syn::GenericArgument::Type(elem) => is_assoc(elem),
                            syn::GenericArgument::Lifetime(_)
                            | syn::GenericArgument::Const(_)
                            | syn::GenericArgument::Binding(_)
                            | syn::GenericArgument::Constraint(_) => false,
                        }),
                        PathArguments::Parenthesized(ParenthesizedGenericArguments {
                            inputs,
                            output,
                            ..
                        }) => {
                            if let syn::ReturnType::Type(_, elem) = output {
                                if is_assoc(elem) {
                                    return true;
                                }
                            }
                            inputs.iter().any(is_assoc)
                        }
                    }
                })
        }
        Type::BareFn(_) => todo!("stabby doesn't support bare functions in method parameters yet"),
        Type::Paren(TypeParen { elem, .. })
        | Type::Ptr(TypePtr { elem, .. })
        | Type::Reference(TypeReference { elem, .. })
        | Type::Array(TypeArray { elem, .. })
        | Type::Group(TypeGroup { elem, .. }) => is_assoc(elem),
        Type::Macro(_) | Type::Never(_) | Type::Verbatim(_) => false,
        Type::Infer(_) => panic!("type inference is not available in trait definitions"),
        Type::ImplTrait(_) => panic!("generic methods are not trait object safe"),
        Type::Slice(_) => panic!("slices are not ABI stable"),
        Type::TraitObject(_) => panic!("trait objects are not ABI stable"),
        Type::Tuple(_) => panic!("tuples are not ABI stable"),
        _ => panic!("unknown element type in {}", quote!(#elem)),
    }
}
