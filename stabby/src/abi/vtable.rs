use crate as stabby;

/// Implementation detail for stabby's version of dyn traits.
/// Any type that implements a trait `ITrait` must implement `IConstConstructor<VtITrait>` for `stabby::dyn!(Ptr<ITrait>)::from(value)` to work.
pub trait IConstConstructor<'a, Vt: 'a + Copy> {
    const VTABLE: &'a Vt;
}

/// Implementation detail for stabby's version of dyn traits.
pub trait TransitiveDeref<Head, N> {
    fn tderef(&self) -> &Head;
}
/// Implementation detail for stabby's version of dyn traits.
pub struct H;
/// Implementation detail for stabby's version of dyn traits.
pub struct T<T>(T);
/// A recursive type to define sets of v-tables.
/// You should _always_ use `stabby::vtable!(Trait1 + Trait2 + ...)` to generate this type,
/// as this macro will ensure that traits are ordered consistently in the vtable.
#[stabby::stabby]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct VTable<Head, Tail = VtDrop> {
    head: Head,
    tail: Tail,
}

pub trait CompoundVt {
    type Vt<T>;
}

impl<'a, T, Head: Copy + 'a, Tail: Copy + 'a> IConstConstructor<'a, VTable<Head, Tail>> for T
where
    T: IConstConstructor<'a, Head> + IConstConstructor<'a, Tail>,
{
    const VTABLE: &'a VTable<Head, Tail> = &VTable {
        head: *T::VTABLE,
        tail: *T::VTABLE,
    };
}
impl<'a, T> IConstConstructor<'a, ()> for T {
    const VTABLE: &'a () = &();
}
impl<Head, Tail> TransitiveDeref<Head, H> for VTable<Head, Tail> {
    fn tderef(&self) -> &Head {
        &self.head
    }
}
impl<Head, Tail: TransitiveDeref<Vt, N>, Vt, N> TransitiveDeref<Vt, T<N>> for VTable<Head, Tail> {
    fn tderef(&self) -> &Vt {
        self.tail.tderef()
    }
}

/// Allows extracting the dropping vtable from a vtable
pub trait HasDropVt {
    fn drop_vt(&self) -> &VtDrop;
}
impl HasDropVt for VtDrop {
    fn drop_vt(&self) -> &VtDrop {
        self
    }
}
impl<Head, Tail: HasDropVt> HasDropVt for VTable<Head, Tail> {
    fn drop_vt(&self) -> &VtDrop {
        self.tail.drop_vt()
    }
}
impl<T: HasDropVt> HasDropVt for VtSend<T> {
    fn drop_vt(&self) -> &VtDrop {
        self.0.drop_vt()
    }
}
impl<T: HasDropVt> HasDropVt for VtSync<T> {
    fn drop_vt(&self) -> &VtDrop {
        self.0.drop_vt()
    }
}

pub trait HasSendVt {}
impl<T> HasSendVt for VtSend<T> {}
impl<T: HasSendVt> HasSendVt for VtSync<T> {}
impl<Head, Tail: HasSyncVt> HasSendVt for VTable<Head, Tail> {}
pub trait HasSyncVt {}
impl<T> HasSyncVt for VtSync<T> {}
impl<T: HasSyncVt> HasSyncVt for VtSend<T> {}
impl<Head, Tail: HasSyncVt> HasSyncVt for VTable<Head, Tail> {}

// DROP
/// The vtable to drop a value in place
#[stabby::stabby]
#[derive(Clone, Copy)]
pub struct VtDrop {
    pub drop: crate::abi::StableLike<unsafe extern "C" fn(&mut ()), core::num::NonZeroUsize>,
}
impl PartialEq for VtDrop {
    fn eq(&self, other: &Self) -> bool {
        core::ptr::eq(
            (*self.drop) as *const unsafe extern "C" fn(&mut ()),
            (*other.drop) as *const unsafe extern "C" fn(&mut ()),
        )
    }
}
impl<'a, T> IConstConstructor<'a, VtDrop> for T {
    const VTABLE: &'a VtDrop = &VtDrop {
        drop: unsafe {
            core::mem::transmute({
                unsafe extern "C" fn drop<T>(this: &mut T) {
                    core::ptr::drop_in_place(this)
                }
                drop::<T>
            } as unsafe extern "C" fn(&mut T))
        },
    };
}

/// A marker for vtables for types that are `Send`
#[stabby::stabby]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct VtSend<T>(T);
impl CompoundVt for dyn Send {
    type Vt<T> = VtSend<T>;
}
impl<Tail: TransitiveDeref<Vt, N>, Vt, N> TransitiveDeref<Vt, T<N>> for VtSend<Tail> {
    fn tderef(&self) -> &Vt {
        self.0.tderef()
    }
}
impl<Head, Tail> From<crate::abi::vtable::VtSend<VTable<Head, Tail>>> for VTable<Head, Tail> {
    fn from(value: VtSend<VTable<Head, Tail>>) -> Self {
        value.0
    }
}
impl<'a, T: IConstConstructor<'a, Vt> + Send, Vt: Copy + 'a> IConstConstructor<'a, VtSend<Vt>>
    for T
{
    const VTABLE: &'a VtSend<Vt> = &VtSend(*T::VTABLE);
}

/// A marker for vtables for types that are `Sync`
#[stabby::stabby]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct VtSync<T>(T);
impl CompoundVt for dyn Sync {
    type Vt<T> = VtSync<T>;
}
impl<'a, T: IConstConstructor<'a, Vt> + Sync, Vt: Copy + 'a> IConstConstructor<'a, VtSync<Vt>>
    for T
{
    const VTABLE: &'a VtSync<Vt> = &VtSync(*T::VTABLE);
}
impl<Tail: TransitiveDeref<Vt, N>, Vt, N> TransitiveDeref<Vt, T<N>> for VtSync<Tail> {
    fn tderef(&self) -> &Vt {
        self.0.tderef()
    }
}
impl<Head, Tail> From<crate::abi::vtable::VtSync<VtSend<VTable<Head, Tail>>>>
    for VTable<Head, Tail>
{
    fn from(value: VtSync<VtSend<VTable<Head, Tail>>>) -> Self {
        value.0 .0
    }
}
impl<Head, Tail> From<crate::abi::vtable::VtSync<VtSend<VTable<Head, Tail>>>>
    for VtSend<VTable<Head, Tail>>
{
    fn from(value: VtSync<VtSend<VTable<Head, Tail>>>) -> Self {
        value.0
    }
}
impl<Head, Tail> From<crate::abi::vtable::VtSync<VTable<Head, Tail>>> for VTable<Head, Tail> {
    fn from(value: VtSync<VTable<Head, Tail>>) -> Self {
        value.0
    }
}
