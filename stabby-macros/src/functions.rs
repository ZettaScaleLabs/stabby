use quote::{quote, ToTokens};

pub fn stabby(fn_spec: syn::ItemFn) -> proc_macro2::TokenStream {
    let st = crate::tl_mod();
    fn assert_stable(st: &impl ToTokens, ty: impl ToTokens) -> proc_macro2::TokenStream {
        quote!(let _ = #st::AssertStable::<#ty>(::core::marker::PhantomData);)
    }

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
    quote! {
        #(#attrs)*
        #vis extern "C" #sig {
            #(#stable_asserts)*
            #block
        }
    }
}
