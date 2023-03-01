use quote::quote;
use syn::{ConstParam, GenericParam, LifetimeDef, TypeParam};

#[derive(Clone, Default)]
pub(crate) struct SeparatedGenerics {
    pub lifetimes: Vec<proc_macro2::TokenStream>,
    pub types: Vec<proc_macro2::TokenStream>,
    pub consts: Vec<proc_macro2::TokenStream>,
}
impl quote::ToTokens for SeparatedGenerics {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        for l in &self.lifetimes {
            tokens.extend(quote!(#l,));
        }
        for l in &self.types {
            tokens.extend(quote!(#l,));
        }
        for l in &self.consts {
            tokens.extend(quote!(#l,));
        }
    }
}
pub(crate) fn unbound_generics<'a>(
    generics: impl IntoIterator<Item = &'a GenericParam>,
) -> SeparatedGenerics {
    let mut this = SeparatedGenerics::default();
    for g in generics {
        match g {
            GenericParam::Type(TypeParam { ident, .. }) => this.types.push(quote!(#ident)),
            GenericParam::Lifetime(LifetimeDef { lifetime, .. }) => {
                this.lifetimes.push(quote!(#lifetime))
            }
            GenericParam::Const(ConstParam { ident, .. }) => this.consts.push(quote!(#ident)),
        }
    }
    this
}
pub(crate) fn generics_without_defaults<'a>(
    generics: impl IntoIterator<Item = &'a GenericParam>,
) -> SeparatedGenerics {
    let mut this = SeparatedGenerics::default();
    for g in generics {
        match g {
            GenericParam::Type(TypeParam { ident, bounds, .. }) => {
                this.types.push(quote!(#ident: #bounds))
            }
            GenericParam::Lifetime(LifetimeDef {
                lifetime, bounds, ..
            }) => this.lifetimes.push(quote!(#lifetime: #bounds)),
            GenericParam::Const(ConstParam { ident, .. }) => this.consts.push(quote!(#ident)),
        }
    }
    this
}
