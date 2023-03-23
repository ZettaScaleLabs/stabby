use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

struct Attrs {
    unsend: bool,
    unsync: bool,
    lt: syn::Lifetime,
}
enum Attr {
    Unsend,
    Unsync,
    Lt(syn::Lifetime),
}
impl syn::parse::Parse for Attr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(syn::Lifetime) {
            Ok(Attr::Lt(input.parse()?))
        } else {
            let ident: syn::Ident = input.parse()?;
            match ident.to_string().as_str() {
                "unsend" => Ok(Attr::Unsend),
                "unsync" => Ok(Attr::Unsync),
                _ => Err(syn::Error::new(ident.span(), "Unsupported attribute for `stabby` on functions: only lifetimes, `unsend` and `unsync` are supported"))
            }
        }
    }
}
impl syn::parse::Parse for Attrs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut this = Self {
            unsend: false,
            unsync: false,
            lt: syn::Lifetime {
                apostrophe: input.span(),
                ident: quote::format_ident!("static"),
            },
        };
        for attr in syn::punctuated::Punctuated::<Attr, syn::Token!(,)>::parse_terminated(input)? {
            match attr {
                Attr::Unsend => this.unsend = true,
                Attr::Unsync => this.unsync = true,
                Attr::Lt(lt) => this.lt = lt,
            }
        }
        Ok(this)
    }
}

pub fn stabby(attrs: proc_macro::TokenStream, fn_spec: syn::ItemFn) -> proc_macro2::TokenStream {
    let st = crate::tl_mod();
    fn assert_stable(st: &impl ToTokens, ty: impl ToTokens) -> proc_macro2::TokenStream {
        quote!(let _ = #st::AssertStable::<#ty>(::core::marker::PhantomData);)
    }
    let Attrs { unsend, unsync, lt } = syn::parse(attrs).unwrap();

    let syn::ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = fn_spec;
    let syn::Signature {
        abi,
        inputs,
        output,
        asyncness,
        generics,
        unsafety,
        constness,
        ident,
        ..
    } = &sig;
    assert!(
        abi.is_none(),
        "stabby will attribute a stable ABI to your function on its own"
    );
    let mut stable_asserts = Vec::new();
    if let syn::ReturnType::Type(_, ty) = output {
        stable_asserts.push(assert_stable(&st, ty));
    }
    stable_asserts.extend(inputs.iter().map(|i| match i {
        syn::FnArg::Receiver(_) => assert_stable(&st, quote!(Self)),
        syn::FnArg::Typed(syn::PatType { ty, .. }) => assert_stable(&st, ty),
    }));
    let (output, block) = if asyncness.is_some() {
        let mut future = match output {
            syn::ReturnType::Default => quote!(#st::future::Future<Output=()>),
            syn::ReturnType::Type(_, ty) => quote!(#st::future::Future<Output=#ty>),
        };
        if !unsend {
            future = quote!(#future + Send)
        }
        if !unsync {
            future = quote!(#future + Sync)
        }
        let vt: TokenStream = crate::vtable(future.into()).into();
        let output = quote!( -> #st::Dyn<#lt, Box<()>, #vt>);
        (output, quote!(Box::new(async {#block}).into()))
    } else {
        (quote!(#output), quote!(#block))
    };
    quote! {
        #(#attrs)*
        #vis #unsafety #constness extern "C" fn #ident #generics (#inputs) #output {
            #(#stable_asserts)*
            #block
        }
    }
}
