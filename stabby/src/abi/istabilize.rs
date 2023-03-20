use crate::abi::*;

pub trait IStabilize {
    type Stable: IStable;
    fn stable(self) -> Self::Stable;
}

impl<'a, T: IStable> IStabilize for &'a [T] {
    type Stable = crate::slice::Slice<'a, T>;
    fn stable(self) -> Self::Stable {
        self.into()
    }
}

impl<'a, T: IStable> IStabilize for &'a mut [T] {
    type Stable = crate::slice::SliceMut<'a, T>;
    fn stable(self) -> Self::Stable {
        self.into()
    }
}
impl<'a> IStabilize for &'a str {
    type Stable = crate::str::Str<'a>;
    fn stable(self) -> Self::Stable {
        self.into()
    }
}

impl<'a> IStabilize for &'a mut str {
    type Stable = crate::str::StrMut<'a>;
    fn stable(self) -> Self::Stable {
        self.into()
    }
}
impl<T> IStabilize for core::pin::Pin<T>
where
    T: core::ops::Deref + IStabilize,
    T::Stable: core::ops::Deref,
{
    type Stable = core::pin::Pin<T::Stable>;
    fn stable(self) -> Self::Stable {
        unsafe {
            let p = core::pin::Pin::into_inner_unchecked(self);
            core::pin::Pin::new_unchecked(p.stable())
        }
    }
}

#[cfg(feature = "alloc")]
mod alloc {
    use super::*;
    use ::alloc::boxed::Box;
    impl<T: IStable> IStabilize for Box<[T]> {
        type Stable = crate::boxed::BoxedSlice<T>;
        fn stable(self) -> Self::Stable {
            self.into()
        }
    }
    impl IStabilize for Box<str> {
        type Stable = crate::boxed::BoxedStr;
        fn stable(self) -> Self::Stable {
            self.into()
        }
    }
}
