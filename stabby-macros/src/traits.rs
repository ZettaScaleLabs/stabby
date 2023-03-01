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
}
struct DynTraitFn<'a> {
    ident: &'a Ident,
    generics: &'a syn::Generics,
    abi: TokenStream,
    unsafety: Option<syn::token::Unsafe>,
    inputs: Vec<TokenStream>,
    output: Option<TokenStream>,
}
impl<'a> From<&'a syn::ItemTrait> for DynTraitDescription<'a> {
    fn from(
        syn::ItemTrait {
            vis,
            ident,
            generics,
            supertraits,
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
                    let inputs = inputs.iter().map(|input| match input {
                        syn::FnArg::Receiver(Receiver {
                            reference: Some(_),
                            mutability,
                            ..
                        }) => {
                            quote!(&#mutability #self_ty)
                        }
                        syn::FnArg::Typed(PatType { ty, .. }) => {
                            replace_self::<false>(ty, &self_ty)
                        }
                        _ => panic!("fn (self, ...) is not trait safe"),
                    });
                    let output = match output {
                        syn::ReturnType::Default => None,
                        syn::ReturnType::Type(_, ty) => Some(replace_self::<true>(ty, &self_ty)),
                    };
                    this.functions.push(DynTraitFn {
                        ident,
                        generics,
                        unsafety: *unsafety,
                        abi: quote!(#abi),
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

pub fn stabby(item_trait: syn::ItemTrait) -> TokenStream {
    let st = crate::tl_mod();
    let description: DynTraitDescription = (&item_trait).into();
    let vtident = quote::format_ident!("Vt{ident}", ident = item_trait.ident);
    quote! {
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
