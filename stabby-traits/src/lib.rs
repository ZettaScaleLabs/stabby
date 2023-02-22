#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
pub(crate) use std as alloc;
#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

pub use stabby_macros::stabby;

pub use type_layouts::{AssertStable, IStable as Stable};
pub mod type_layouts;

pub mod holes {
    include!(concat!(env!("OUT_DIR"), "/holes.rs"));
}
pub(crate) mod stabby_traits {
    pub use super::holes;
}

// pub mod typenum {
//     pub struct B0;
//     pub struct B1;
//     pub trait IEq<T> {
//         type Output: Bit;
//     }
//     impl IEq<B0> for B0 {
//         type Output = B1;
//     }
//     impl IEq<B1> for B1 {
//         type Output = B1;
//     }
//     impl IEq<B1> for B0 {
//         type Output = B0;
//     }
//     impl IEq<B0> for B1 {
//         type Output = B0;
//     }
//     pub trait Bit {
//         type BEq<Br>: Bit
//         where
//             Self: IEq<Br>;
//         type BNot: Bit;
//         type BAnd<Br>: Bit
//         where
//             Self: IEq<Br>;
//         type BOr<Br>: Bit
//         where
//             Self: IEq<Br>;
//     }
//     impl Bit for B0 {
//         type BEq<Br> = <Self as IEq<Br>>::Output
//         where
//             Self: IEq<Br>,
//             <Self as IEq<Br>>::Output: Bit;
//         type BNot = B1;
//         type BAnd<Br> = B0
//         where
//             Self: IEq<Br>;
//         type BOr<Br> = <Self::BEq<Br> as Bit>::BNot
//         where
//             Self: IEq<Br>;
//     }
//     impl Bit for B1 {
//         type BEq<Br> = <Self as IEq<Br>>::Output
//         where
//             Self: IEq<Br>,
//             <Self as IEq<Br>>::Output: Bit;
//         type BNot = B0;
//         type BAnd<Br> = Self::BEq<Br>
//         where
//             Self: IEq<Br>;
//         type BOr<Br> = B1
//         where
//             Self: IEq<Br>;
//     }
//     pub trait BitExt {
//         type XOr<Br>: Bit
//         where
//             Self: IEq<Br>;
//     }
//     impl<B: Bit> BitExt for B {
//         type XOr<Br> = <<B as Bit>::BEq<Br> as Bit>::BNot
//         where
//             Self: IEq<Br>;
//     }
//     pub type UTerm = B0;
//     pub struct UI<B, Cons>(B, Cons);
//     impl<B, Cons> IEq<UTerm> for UI<B, Cons> {
//         type Output = B0;
//     }
//     impl<B, Cons, Br, Consr> IEq<UI<Br, Consr>> for UI<B, Cons>
//     where
//         B: IEq<Br>,
//         (<B as IEq<Br>>::Output, Cons): IEq<Consr>,
//     {
//         type Output = <(<B as IEq<Br>>::Output, Cons) as IEq<Consr>>::Output;
//     }
//     impl<Cons, Consr> IEq<Consr> for (B0, Cons) {
//         type Output = B0;
//     }
//     impl<Cons: IEq<Consr>, Consr> IEq<Consr> for (B1, Cons) {
//         type Output = <Cons as IEq<Consr>>::Output;
//     }
// }

mod stable_impls;
