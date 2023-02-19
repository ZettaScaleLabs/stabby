use quote::quote;
use syn::{parse::Parse, Token, Type};

#[derive(Clone)]
pub enum TyExpr {
    Type(Type),
    Add(Box<Self>, Box<Self>),
    Sub(Box<Self>, Box<Self>),
    Rem(Box<Self>, Box<Self>),
    BitOr(Box<Self>, Box<Self>),
    BitAnd(Box<Self>, Box<Self>),
    Not(Box<Self>),
}
impl Parse for TyExpr {
    fn parse(tokens: syn::parse::ParseStream) -> syn::Result<Self> {
        let res = if tokens.peek(syn::token::Paren) {
            let content;
            syn::parenthesized!(content in tokens);
            content.parse()?
        } else if tokens.peek(Token!(!)) {
            tokens.parse::<Token!(!)>().unwrap();
            Self::Not(Box::new(tokens.parse()?))
        } else if tokens.peek(Token!(+)) {
            tokens.parse::<Token!(+)>().unwrap();
            Self::Add(Box::new(tokens.parse()?), Box::new(tokens.parse()?))
        } else if tokens.peek(Token!(-)) {
            tokens.parse::<Token!(-)>().unwrap();
            Self::Sub(Box::new(tokens.parse()?), Box::new(tokens.parse()?))
        } else if tokens.peek(Token!(%)) {
            tokens.parse::<Token!(%)>().unwrap();
            Self::Rem(Box::new(tokens.parse()?), Box::new(tokens.parse()?))
        } else if tokens.peek(Token!(|)) {
            tokens.parse::<Token!(|)>().unwrap();
            Self::BitOr(Box::new(tokens.parse()?), Box::new(tokens.parse()?))
        } else if tokens.peek(Token!(&)) {
            tokens.parse::<Token!(&)>().unwrap();
            Self::BitAnd(Box::new(tokens.parse()?), Box::new(tokens.parse()?))
        } else {
            Self::Type(tokens.parse()?)
        };
        Ok(res)
    }
}

pub fn tyeval(tokens: &TyExpr) -> proc_macro2::TokenStream {
    match tokens {
        TyExpr::Type(ty) => quote!(#ty),
        TyExpr::Add(l, r) => {
            let l = tyeval(l);
            let r = tyeval(r);
            quote!(<#l as ::core::ops::Add<#r>>::Output)
        }
        TyExpr::Sub(l, r) => {
            let l = tyeval(l);
            let r = tyeval(r);
            quote!(<#l as ::core::ops::Sub<#r>>::Output)
        }
        TyExpr::Rem(l, r) => {
            let l = tyeval(l);
            let r = tyeval(r);
            quote!(<#l as ::core::ops::Rem<#r>>::Output)
        }
        TyExpr::BitOr(l, r) => {
            let l = tyeval(l);
            let r = tyeval(r);
            quote!(<#l as ::core::ops::BitOr<#r>>::Output)
        }
        TyExpr::BitAnd(l, r) => {
            let l = tyeval(l);
            let r = tyeval(r);
            quote!(<#l as ::core::ops::BitAnd<#r>>::Output)
        }
        TyExpr::Not(ty) => {
            let ty = tyeval(ty);
            quote!(<#ty as ::core::ops::Not>::Output)
        }
    }
}

pub fn tybound(tokens: &TyExpr) -> proc_macro2::TokenStream {
    match tokens {
        TyExpr::Type(_) => quote!(),
        TyExpr::Add(l, r) => {
            let lbound = tybound(l);
            let rbound = tybound(r);
            let l = tyeval(l);
            let r = tyeval(r);
            quote!(<#lbound #rbound #l: ::core::ops::Add<#r>>,)
        }
        TyExpr::Sub(l, r) => {
            let lbound = tybound(l);
            let rbound = tybound(r);
            let l = tyeval(l);
            let r = tyeval(r);
            quote!(<#lbound #rbound #l: ::core::ops::Sub<#r>>,)
        }
        TyExpr::Rem(l, r) => {
            let lbound = tybound(l);
            let rbound = tybound(r);
            let l = tyeval(l);
            let r = tyeval(r);
            quote!(<#lbound #rbound #l: ::core::ops::Rem<#r>>,)
        }
        TyExpr::BitOr(l, r) => {
            let lbound = tybound(l);
            let rbound = tybound(r);
            let l = tyeval(l);
            let r = tyeval(r);
            quote!(<#lbound #rbound #l: ::core::ops::BitOr<#r>>,)
        }
        TyExpr::BitAnd(l, r) => {
            let lbound = tybound(l);
            let rbound = tybound(r);
            let l = tyeval(l);
            let r = tyeval(r);
            quote!(<#lbound #rbound #l: ::core::ops::BitAnd<#r>>,)
        }
        TyExpr::Not(ty) => {
            let bound = tybound(ty);
            let ty = tyeval(ty);
            quote!(#bound <#ty: ::core::ops::Not>,)
        }
    }
}
