use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    PatType, Path, PathArguments, PathSegment, QSelf, Receiver, Signature, TraitItemMethod,
    TraitItemType, Type, TypeArray, TypeGroup, TypeParen, TypePath, TypePtr, TypeReference,
};

pub fn stabby(
    syn::ItemTrait {
        attrs,
        vis,
        unsafety,
        auto_token,
        trait_token,
        ident,
        generics,
        colon_token,
        supertraits,
        brace_token: _,
        items,
    }: syn::ItemTrait,
    st: TokenStream,
) -> TokenStream {
    let mut vtable_fields: Vec<TokenStream> = Vec::new();
    let self_ty = quote!(());
    for item in &items {
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
                    syn::FnArg::Typed(PatType { ty, .. }) => replace_self::<false>(ty, &self_ty),
                    _ => panic!("fn (self, ...) is not trait safe"),
                });
                let output = match output {
                    syn::ReturnType::Default => quote!(),
                    syn::ReturnType::Type(_, ty) => {
                        let ty = replace_self::<true>(ty, &self_ty);
                        quote!(-> #ty)
                    }
                };
                let field = quote!(#ident: extern "C" #unsafety fn (#(#inputs),*) #output,);
                vtable_fields.push(field)
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
    let vtident = quote::format_ident!("Vt{ident}");
    let hasvtident = quote::format_ident!("HasVt{ident}");
    let unbound_generics = crate::unbound_generics(&generics.params);
    quote! {
        pub struct #vtident <  #unbound_generics> {
            drop: extern "C" fn (&mut ()),
            #(#vtable_fields)*
        }
        pub trait #hasvtident #generics : #ident < #unbound_generics > {
            const VTABLE: #vtident <  #unbound_generics >;
            fn vtable(&self) -> &'static #vtident <  #unbound_generics > {&Self::VTABLE}
        }
        #(#attrs)*
        #vis #unsafety #auto_token #trait_token #ident #generics #colon_token #supertraits {
            #(#items)*
        }
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
