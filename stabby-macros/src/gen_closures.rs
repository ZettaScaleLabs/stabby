//
// Copyright (c) 2023 ZettaScale Technology
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   Pierre Avital, <pierre.avital@me.com>
//

use quote::quote;
pub fn gen_closures() -> proc_macro2::TokenStream {
    let st = crate::tl_mod();
    let generator = (0..10).map(|i| {
        let c = quote::format_ident!("Call{i}");
        let cm = quote::format_ident!("CallMut{i}");
        let co = quote::format_ident!("CallOnce{i}");
        let com = quote::format_ident!("call_once_{i}");
        let cvt = quote::format_ident!("StabbyVtableCall{i}");
        let cmvt = quote::format_ident!("StabbyVtableCallMut{i}");
        let covt = quote::format_ident!("StabbyVtableCallOnce{i}");
        let cod = quote::format_ident!("CallOnceDyn{i}");
		let argtys = (0..i).map(|i| quote::format_ident!("I{i}")).collect::<Vec<_>>();
		let args = (0..i).map(|i| quote::format_ident!("_{i}")).collect::<Vec<_>>();
        quote! {
			#[cfg(feature = "alloc")]
			pub use #com::*;
			#[cfg(feature = "alloc")]
			mod #com {
				use crate::{
					vtable::{HasDropVt, TransitiveDeref},
					StableIf, StableLike,
				};
				pub trait #co<O #(, #argtys)* > {
					extern "C" fn call_once(self: #st::boxed::Box<Self> #(, #args: #argtys)*) -> O;
				}
				impl<O #(, #argtys)* , F: FnOnce(#(#argtys,)*) -> O> #co<O #(, #argtys)*> for F {
					extern "C" fn call_once(self: #st::boxed::Box<Self> #(, #args: #argtys)*) -> O {
						self(#(#args,)*)
					}
				}

				#[crate::stabby]
				pub struct #covt<O #(, #argtys)* > {
					call_once: StableIf<StableLike<unsafe extern "C" fn(#st::boxed::Box<()>  #(, #argtys)* ) -> O, &'static ()>, O>,
				}
				impl<O #(, #argtys)* > Copy for #covt<O #(, #argtys)* > {}
				impl<O #(, #argtys)* > Clone for #covt<O #(, #argtys)* > {
					fn clone(&self) -> Self {
						*self
					}
				}
				pub trait #cod<O #(, #argtys)* , N> {
					fn call_once(self #(, _: #argtys)* ) -> O;
				}
				impl<'a, O #(, #argtys)* , Vt: TransitiveDeref<#covt<O #(, #argtys)* >, N> + HasDropVt, N> #cod<O #(, #argtys)* , N>
					for crate::Dyn<'a, Box<()>, Vt>
				{
					fn call_once(self #(, #args: #argtys)*) -> O {
						let o =
							unsafe { (self.vtable().tderef().call_once.into_inner_unchecked())(core::ptr::read(self.ptr()) #(, #args)*)};
						core::mem::forget(self);
						o
					}
				}

				impl<O #(, #argtys)* > crate::vtable::CompoundVt for dyn FnOnce(#(#argtys, )*) -> O {
					type Vt<T> = crate::vtable::VTable<#covt<O #(, #argtys)* >, T>;
				}
				impl<'a, O: 'a #(, #argtys: 'a)* , F: FnOnce(#(#argtys, )*) -> O> crate::vtable::IConstConstructor<'a, F>
					for #covt<O #(, #argtys)* >
				{
					const VTABLE: &'a Self = &Self {
						call_once: unsafe {
							core::mem::transmute(<F as #co< O #(, #argtys)* >>::call_once as extern "C" fn(#st::boxed::Box<F> #(, #argtys)* ) -> O)
						},
					};
				}
			}


			#[cfg(feature = "alloc")]
			#[crate::stabby]
			pub trait #cm<O #(, #argtys)* >: #co<O #(, #argtys)* > {
				extern "C" fn call_mut(&mut self #(, #args: #argtys)*) -> O;
			}
			#[cfg(not(feature = "alloc"))]
			#[crate::stabby]
			pub trait #cm<O #(, #argtys)* > {
				extern "C" fn call_mut(&mut self #(, #args: #argtys)*) -> O;
			}
			impl<O #(, #argtys)* , F: FnMut(#(#argtys,)*) -> O> #cm<O #(, #argtys)*> for F {
				extern "C" fn call_mut(&mut self #(, #args: #argtys)*) -> O {
					self(#(#args,)*)
				}
			}
			impl<O #(, #argtys)* > crate::vtable::CompoundVt for dyn FnMut(#(#argtys, )*) -> O {
				type Vt<T> = crate::vtable::VTable<#cmvt<O #(, #argtys)* >, T>;
			}
			#[crate::stabby]
			pub trait #c<O #(, #argtys)* >: #cm<O #(, #argtys)* > {
				extern "C" fn call(&self #(, #args: #argtys)*) -> O;
			}
			impl<O #(, #argtys)* , F: Fn(#(#argtys,)*) -> O> #c<O #(, #argtys)*> for F {
				extern "C" fn call(&self #(, #args: #argtys)*) -> O {
					self(#(#args,)*)
				}
			}
			impl<O #(, #argtys)* > crate::vtable::CompoundVt for dyn Fn(#(#argtys, )*) -> O {
				type Vt<T> = crate::vtable::VTable<#cvt<O #(, #argtys)* >, T>;
			}
		}
    });
    quote!(#(#generator)*)
}
