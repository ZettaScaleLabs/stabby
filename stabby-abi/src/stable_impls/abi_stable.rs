use core::ptr::NonNull;

use crate::{typenum2::*, End, IStable, Tuple};

unsafe impl<'a, T: IStable> IStable for abi_stable::RRef<'a, T> {
    same_as!(core::ptr::NonNull<T>);
    primitive_report!("abi_stable::RRef", T);
}
check!(abi_stable::RRef<u8>);

unsafe impl<'a, T: IStable> IStable for abi_stable::RMut<'a, T> {
    same_as!(core::ptr::NonNull<T>);
    primitive_report!("abi_stable::RMut", T);
}
check!(abi_stable::RMut<u8>);

unsafe impl<T: IStable> IStable for abi_stable::std_types::RVec<T> {
    type Size = <<core::ptr::NonNull<T> as IStable>::Size as Unsigned>::Mul<U4>;
    type Align = <core::ptr::NonNull<T> as IStable>::Align;
    type ForbiddenValues = <core::ptr::NonNull<T> as IStable>::ForbiddenValues;
    type UnusedBits = End;
    type HasExactlyOneNiche = B1;
    primitive_report!("abi_stable::std_types::RVec", T);
}
check!(abi_stable::std_types::RVec<u8>);

unsafe impl IStable for abi_stable::std_types::RString {
    same_as!(abi_stable::std_types::RVec<u8>);
    primitive_report!("abi_stable::std_types::RString");
}
check!(abi_stable::std_types::RString);

unsafe impl<'a, T: IStable> IStable for abi_stable::std_types::RSlice<'a, T> {
    type Size = <<core::ptr::NonNull<T> as IStable>::Size as Unsigned>::Mul<U2>;
    type Align = <core::ptr::NonNull<T> as IStable>::Align;
    type ForbiddenValues = End;
    type UnusedBits = End;
    type HasExactlyOneNiche = B0;
    primitive_report!("abi_stable::std_types::RSlice", T);
}
check!(abi_stable::std_types::RSlice<u8>);

unsafe impl<'a> IStable for abi_stable::std_types::RStr<'a> {
    same_as!(abi_stable::std_types::RSlice<'a, u8>);
    primitive_report!("abi_stable::std_types::RStr");
}
check!(abi_stable::std_types::RStr);

unsafe impl<'a, T: IStable> IStable for abi_stable::std_types::RSliceMut<'a, T> {
    type Size = <<core::ptr::NonNull<T> as IStable>::Size as Unsigned>::Mul<U2>;
    type Align = <core::ptr::NonNull<T> as IStable>::Align;
    type ForbiddenValues = End;
    type UnusedBits = End;
    type HasExactlyOneNiche = B0;
    primitive_report!("abi_stable::std_types::RSliceMut", T);
}
check!(abi_stable::std_types::RSliceMut<u8>);

unsafe impl<K, V> IStable for abi_stable::std_types::RHashMap<K, V>
where
    Tuple<K, V>: IStable,
{
    type Size = <<core::ptr::NonNull<()> as IStable>::Size as Unsigned>::Mul<U3>;
    type Align = <core::ptr::NonNull<()> as IStable>::Align;
    type ForbiddenValues = <core::ptr::NonNull<()> as IStable>::ForbiddenValues;
    type UnusedBits = End;
    type HasExactlyOneNiche = B1;
    primitive_report!("abi_stable::std_types::RHashMap", Tuple<K, V>);
}
check!(abi_stable::std_types::RHashMap<u8, u64>);

unsafe impl IStable for abi_stable::std_types::RDuration {
    same_as!(Tuple<u64, u32>);
    primitive_report!("abi_stable::std_types::RDuration");
}
check!(abi_stable::std_types::RDuration);

unsafe impl<T: IStable> IStable for abi_stable::std_types::RBox<T> {
    type Size = <<core::ptr::NonNull<T> as IStable>::Size as Unsigned>::Mul<U2>;
    type Align = <core::ptr::NonNull<T> as IStable>::Align;
    type ForbiddenValues = <core::ptr::NonNull<T> as IStable>::ForbiddenValues;
    type UnusedBits = End;
    type HasExactlyOneNiche = B1;
    primitive_report!("abi_stable::std_types::RBox", T);
}
check!(abi_stable::std_types::RBox<u8>);

unsafe impl IStable for abi_stable::std_types::RBoxError {
    type Size = <<core::ptr::NonNull<()> as IStable>::Size as Unsigned>::Mul<U3>;
    type Align = <core::ptr::NonNull<()> as IStable>::Align;
    type ForbiddenValues = <core::ptr::NonNull<()> as IStable>::ForbiddenValues;
    type UnusedBits = End;
    type HasExactlyOneNiche = B1;
    primitive_report!("abi_stable::std_types::RBoxError");
}
check!(abi_stable::std_types::RBoxError);

unsafe impl IStable for abi_stable::std_types::SendRBoxError {
    same_as!(abi_stable::std_types::RBoxError);
    primitive_report!("abi_stable::std_types::SendRBoxError");
}
check!(abi_stable::std_types::SendRBoxError);

unsafe impl IStable for abi_stable::std_types::UnsyncRBoxError {
    same_as!(abi_stable::std_types::RBoxError);
    primitive_report!("abi_stable::std_types::UnsyncRBoxError");
}
check!(abi_stable::std_types::UnsyncRBoxError);

unsafe impl<T: IStable> IStable for abi_stable::std_types::Tuple1<T> {
    same_as!(T);
    primitive_report!("abi_stable::std_types::Tuple1", T);
}
check!(abi_stable::std_types::Tuple1<u8>);

unsafe impl<T, U> IStable for abi_stable::std_types::Tuple2<T, U>
where
    Tuple<T, U>: IStable,
{
    same_as!(Tuple<T, U>);
    primitive_report!("abi_stable::std_types::Tuple2", Tuple<T, U>);
}
check!(abi_stable::std_types::Tuple2<u8, u64>);

unsafe impl<T: IStable> IStable for abi_stable::std_types::RArc<T> {
    same_as!(Tuple<*const (), NonNull<()>>);
    primitive_report!("abi_stable::std_types::RArc", T);
}
check!(abi_stable::std_types::RArc<u8>);

mod seal {
    use core::cell::UnsafeCell;

    #[crate::stabby]
    pub struct RMutex<T> {
        opaque_mutex: *const (),
        value: UnsafeCell<T>,
        vtable: &'static (),
    }
}

unsafe impl<T: IStable> IStable for abi_stable::external_types::RMutex<T>
where
    seal::RMutex<T>: IStable,
{
    same_as!(seal::RMutex<T>);
    primitive_report!("abi_stable::external_types::RMutex", T);
}
check!(abi_stable::external_types::RMutex<u8>);

unsafe impl<T: IStable> IStable for abi_stable::external_types::RRwLock<T>
where
    seal::RMutex<T>: IStable,
{
    same_as!(seal::RMutex<T>);
    primitive_report!("abi_stable::external_types::RRwLock", T);
}
check!(abi_stable::external_types::RRwLock<u8>);

unsafe impl IStable for abi_stable::external_types::ROnce {
    same_as!(Tuple<*const (), NonNull<()>>);
    primitive_report!("abi_stable::external_types::ROnce");
}
check!(abi_stable::external_types::ROnce);

#[cfg(feature = "abi_stable-channels")]
mod channels {
    use super::*;

    unsafe impl<T: IStable> IStable for abi_stable::external_types::crossbeam_channel::RReceiver<T> {
        same_as!(Tuple<abi_stable::std_types::RBox<T>, NonNull<()>>);
        primitive_report!(
            "abi_stable::external_types::crossbeam_channel::RReceiver",
            T
        );
    }
    check!(abi_stable::external_types::crossbeam_channel::RReceiver<u8>);

    unsafe impl<T: IStable> IStable for abi_stable::external_types::crossbeam_channel::RSender<T> {
        same_as!(Tuple<abi_stable::std_types::RBox<T>, NonNull<()>>);
        primitive_report!("abi_stable::external_types::crossbeam_channel::RSender", T);
    }
    check!(abi_stable::external_types::crossbeam_channel::RSender<u8>);
}
