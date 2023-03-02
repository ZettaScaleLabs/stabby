use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{
    PatType, Path, PathArguments, PathSegment, QSelf, Receiver, Signature, TraitItemMethod,
    TraitItemType, Type, TypeArray, TypeGroup, TypeParen, TypePath, TypePtr, TypeReference,
};

struct DynTraitDescription<'a> {
    ident: &'a Ident,
    vis: &'a syn::Visibility,
    generics: &'a syn::Generics,
    functions: Vec<DynTraitFn<'a>>,
    mut_functions: Vec<DynTraitFn<'a>>,
}
struct DynTraitFn<'a> {
    ident: &'a Ident,
    generics: &'a syn::Generics,
    abi: TokenStream,
    unsafety: Option<syn::token::Unsafe>,
    receiver: Receiver,
    inputs: Vec<Type>,
    output: Option<TokenStream>,
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
        };
        let self_ty = quote!(());
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
                    let output = match output {
                        syn::ReturnType::Default => None,
                        syn::ReturnType::Type(_, ty) => Some(replace_self::<true>(ty, &self_ty)),
                    };
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
                        output,
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
                    todo!("stabby doesn't support associated types YET")
                }
                syn::TraitItem::Const(_) => panic!("associated consts are not trait object safe"),
                syn::TraitItem::Macro(_) => panic!("sabby can't see through macros in traits"),
                syn::TraitItem::Verbatim(tt) => {
                    panic!("stabby failed to parse this token stream {}", tt)
                }
                _ => panic!("unexpected element in trait"),
            }
        }
        this
    }
}
impl DynTraitFn<'_> {}
impl<'a> DynTraitDescription<'a> {
    fn vtid(&self) -> Ident {
        quote::format_ident!("_vt{}", self.ident)
    }
    fn vtable(&self) -> TokenStream {
        let vtid = self.vtid();
        quote! {}
    }
}
pub fn stabby(item_trait: syn::ItemTrait) -> TokenStream {
    let st = crate::tl_mod();
    let description: DynTraitDescription = (&item_trait).into();
    let vtid = description.vtid();
    let vtable = description.vtable();
    quote! {
        #vtable
        impl <todo!()> #st::vtable::CompoundVt for dyn todo!() {
            type Vt<T> = #st::vtable::VTable<#vtid, T>
        }
        #item_trait
    }
}

fn replace_self<const OUTPUT_TYPE: bool>(elem: &Type, self_ty: &TokenStream) -> TokenStream {
    match elem {
        Type::Path(TypePath {
            qself: None,
            path: Path {
                leading_colon,
                segments,
            },
        }) => {
            let segments = segments
                .iter()
                .map(|PathSegment { ident, arguments }| -> TokenStream {
                    let ident = if *ident == "Self" {
                        quote!(#self_ty)
                    } else {
                        quote!(#ident)
                    };
                    let arguments = match arguments {
                        PathArguments::None => quote!(),
                        PathArguments::AngleBracketed(_) => todo!(),
                        PathArguments::Parenthesized(_) => todo!(),
                    };
                    quote!(#ident #arguments)
                });
            quote!(#leading_colon #(#segments)::*)
        }
        Type::Path(TypePath {
            qself: Some(QSelf { ty, position, .. }),
            path,
        }) => {
            todo!("{}", quote!(#ty as #path))
        }
        Type::BareFn(_) => todo!("stabby doesn't support bare functions in method parameters yet"),
        Type::Group(TypeGroup { elem, .. }) => {
            let elem = replace_self::<OUTPUT_TYPE>(elem, self_ty);
            quote!(#elem)
        }
        Type::Paren(TypeParen { elem, .. }) => {
            let elem = replace_self::<OUTPUT_TYPE>(elem, self_ty);
            quote!((#elem))
        }
        Type::Ptr(TypePtr {
            const_token,
            mutability,
            elem,
            ..
        }) => {
            let elem = replace_self::<OUTPUT_TYPE>(elem, self_ty);
            quote!(* #const_token #mutability #elem)
        }
        Type::Reference(TypeReference {
            mutability, elem, ..
        }) => {
            let elem = replace_self::<OUTPUT_TYPE>(elem, self_ty);
            let lifetime = if OUTPUT_TYPE {
                quote!('static)
            } else {
                quote!()
            };
            quote!(& #lifetime #mutability #elem)
        }
        Type::Array(TypeArray { elem, len, .. }) => {
            let elem = replace_self::<OUTPUT_TYPE>(elem, self_ty);
            quote!([#elem; #len])
        }
        Type::Macro(_) | Type::Never(_) | Type::Verbatim(_) => quote!(#elem),
        Type::Infer(_) => panic!("type inference is not available in trait definitions"),
        Type::ImplTrait(_) => panic!("generic methods are not trait object safe"),
        Type::Slice(_) => panic!("slices are not ABI stable"),
        Type::TraitObject(_) => panic!("trait objects are not ABI stable"),
        Type::Tuple(_) => panic!("tuples are not ABI stable"),
        _ => panic!("unknown element type in {}", quote!(#elem)),
    }
}
