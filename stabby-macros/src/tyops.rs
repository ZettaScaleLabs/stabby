use proc_macro2::{Spacing, TokenStream, TokenTree};
use quote::quote;
use syn::Type;

#[derive(Clone)]
pub enum TyExpr {
    Type(Type),
    Not(Box<Self>),
    Ternary(Box<Self>, Box<Self>, Box<Self>),
    Add(Box<Self>, Box<Self>),
    Sub(Box<Self>, Box<Self>),
    Rem(Box<Self>, Box<Self>),
    BitOr(Box<Self>, Box<Self>),
    BitAnd(Box<Self>, Box<Self>),
    IsEqual(Box<Self>, Box<Self>),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TyOps {
    Type,
    Not,
    Ternary,
    Add,
    Sub,
    Rem,
    BitOr,
    BitAnd,
    IsEqual,
}
impl From<proc_macro::TokenStream> for TyExpr {
    fn from(tokens: proc_macro::TokenStream) -> Self {
        proc_macro2::TokenStream::from(tokens).into()
    }
}
impl From<proc_macro2::TokenStream> for TyExpr {
    fn from(tokens: proc_macro2::TokenStream) -> Self {
        let mut tokens = tokens.into_iter().peekable();
        let mut path = TokenStream::new();
        let mut accept_ident = true;
        let mut in_ternary = false;
        let mut operation = TyOps::Type;
        let mut set_op = |op| {
            if operation == TyOps::Type {
                operation = op
            } else {
                panic!("Operations must be surrounded by parentheses")
            }
        };
        let mut types: Vec<Self> = Vec::new();
        while let Some(token) = tokens.next() {
            match token {
                TokenTree::Group(group) => {
                    types.push(group.stream().into());
                    accept_ident = false
                }
                TokenTree::Ident(ident) => {
                    if accept_ident {
                        path.extend(Some(TokenTree::Ident(ident)));
                        accept_ident = false;
                    } else {
                        panic!("Identifier {ident} not accepted here")
                    }
                }
                TokenTree::Punct(p) => {
                    match p.as_char() {
                        ':' => {
                            if p.spacing() == Spacing::Joint {
                                let next = tokens.next().unwrap();
                                assert!(matches!(next, TokenTree::Punct(p) if p.as_char() == ':'));
                                path.extend(quote!(::).into_iter());
                                accept_ident = true;
                                continue;
                            } else if in_ternary {
                                in_ternary = false
                            } else {
                                panic!(": is only allowed in ternaries or as path separator in ::")
                            }
                        }
                        '!' => {
                            if p.spacing() == Spacing::Joint {
                                panic!("!= is not supported yet")
                            } else {
                                set_op(TyOps::Not);
                                assert!(path.is_empty());
                                continue;
                            }
                        }
                        '?' => {
                            set_op(TyOps::Ternary);
                            in_ternary = true
                        }
                        '+' => set_op(TyOps::Add),
                        '-' => set_op(TyOps::Sub),
                        '%' => set_op(TyOps::Rem),
                        '|' => set_op(TyOps::BitOr),
                        '&' => set_op(TyOps::BitAnd),
                        '=' => {
                            if p.spacing() == Spacing::Joint {
                                let next = tokens.next().unwrap();
                                assert!(matches!(next, TokenTree::Punct(p) if p.as_char() == '='));
                                set_op(TyOps::IsEqual)
                            } else {
                                panic!("Did you mean == ?")
                            }
                        }
                        '<' => {
                            let mut count = 1;
                            let braced: proc_macro2::TokenStream = tokens
                                .by_ref()
                                .take_while(|t| {
                                    if let TokenTree::Punct(p) = t {
                                        match p.as_char() {
                                            '<' => count += 1,
                                            '>' => count -= 1,
                                            _ => {}
                                        }
                                    }
                                    count != 0
                                })
                                .collect();
                            path = quote!(#path < #braced >);
                            continue;
                        }
                        c => panic!("{c} is not supported: {path}"),
                    }
                    if !path.is_empty() {
                        types.push(Self::Type(syn::parse2(path).expect("Failed to parse type")));
                        path = TokenStream::new();
                    }
                    accept_ident = true;
                }
                TokenTree::Literal(_) => panic!("Litterals can't be types"),
            }
        }
        if !path.is_empty() {
            types.push(Self::Type(
                syn::parse2(path).expect("Failed to parse final type"),
            ));
        }
        match operation {
            TyOps::Type => {
                assert_eq!(types.len(), 1, "Type");
                types.pop().unwrap()
            }
            TyOps::Not => {
                assert_eq!(types.len(), 1);
                Self::Not(Box::new(types.pop().unwrap()))
            }
            TyOps::Ternary => {
                assert_eq!(types.len(), 3);
                let f = Box::new(types.pop().unwrap());
                let t = Box::new(types.pop().unwrap());
                let cond = Box::new(types.pop().unwrap());
                Self::Ternary(cond, t, f)
            }
            TyOps::Add => {
                assert_eq!(types.len(), 2);
                let r = Box::new(types.pop().unwrap());
                let l = Box::new(types.pop().unwrap());
                Self::Add(l, r)
            }
            TyOps::Sub => {
                assert_eq!(types.len(), 2);
                let r = Box::new(types.pop().unwrap());
                let l = Box::new(types.pop().unwrap());
                Self::Sub(l, r)
            }
            TyOps::Rem => {
                assert_eq!(types.len(), 2);
                let r = Box::new(types.pop().unwrap());
                let l = Box::new(types.pop().unwrap());
                Self::Rem(l, r)
            }
            TyOps::BitOr => {
                assert_eq!(types.len(), 2);
                let r = Box::new(types.pop().unwrap());
                let l = Box::new(types.pop().unwrap());
                Self::BitOr(l, r)
            }
            TyOps::BitAnd => {
                assert_eq!(types.len(), 2);
                let r = Box::new(types.pop().unwrap());
                let l = Box::new(types.pop().unwrap());
                Self::BitAnd(l, r)
            }
            TyOps::IsEqual => {
                assert_eq!(types.len(), 2);
                let r = Box::new(types.pop().unwrap());
                let l = Box::new(types.pop().unwrap());
                Self::IsEqual(l, r)
            }
        }
    }
}

pub fn tyeval(tokens: &TyExpr) -> proc_macro2::TokenStream {
    match tokens {
        TyExpr::Type(ty) => quote!(#ty),
        TyExpr::Add(l, r) => {
            let l = tyeval(l);
            let r = tyeval(r);
            quote!(<#l as ::typenum2::Unsigned>::Add<#r>)
        }
        TyExpr::Sub(l, r) => {
            let l = tyeval(l);
            let r = tyeval(r);
            quote!(<#l as ::typenum2::Unsigned>::AbsSub<#r>)
        }
        TyExpr::Rem(l, r) => {
            let l = tyeval(l);
            let r = tyeval(r);
            quote!(<#l as ::typenum2::Unsigned>::Mod<#r>)
        }
        TyExpr::BitOr(l, r) => {
            let l = tyeval(l);
            let r = tyeval(r);
            quote!(<#l as ::typenum2::Unsigned>::BitOr<#r>)
        }
        TyExpr::BitAnd(l, r) => {
            let l = tyeval(l);
            let r = tyeval(r);
            quote!(<#l as ::typenum2::Unsigned>::BitAnd<#r>)
        }
        TyExpr::Not(ty) => {
            let ty = tyeval(ty);
            quote!(<#ty as ::core::ops::Not>::Output)
        }
        TyExpr::Ternary(cond, t, f) => {
            let cond = tyeval(cond);
            let t = tyeval(t);
            let f = tyeval(f);
            quote!(<#cond as Ternary<#t, #f>>::Output)
        }
        TyExpr::IsEqual(l, r) => {
            let l = tyeval(l);
            let r = tyeval(r);
            quote!(<#l as ::typenum2::Unsigned>::Equal<#r>)
        }
    }
}
