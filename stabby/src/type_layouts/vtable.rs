use crate as stabby;

/// Implementation detail for stabby's version of dyn traits.
/// Any type that implements a trait `ITrait` must implement `IConstConstructor<VtITrait>` for `stabby::dyn!(Ptr<ITrait>)::from(value)` to work.
pub trait IConstConstructor<Vt: 'static + Copy> {
    const VTABLE: &'static Vt;
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
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct VTable<Head, Tail = VtDrop> {
    head: Head,
    tail: Tail,
}

impl<T, Head: Copy + 'static, Tail: Copy + 'static> IConstConstructor<VTable<Head, Tail>> for T
where
    T: IConstConstructor<Head> + IConstConstructor<Tail>,
{
    const VTABLE: &'static VTable<Head, Tail> = &VTable {
        head: *T::VTABLE,
        tail: *T::VTABLE,
    };
}
impl<T> IConstConstructor<()> for T {
    const VTABLE: &'static () = &();
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
impl<T> HasSendVt for VtSync<VtSend<T>> {}
pub trait HasSyncVt {}
impl<T> HasSyncVt for VtSync<T> {}
impl<T> HasSyncVt for VtSend<VtSync<T>> {}

// DROP
/// The vtable to drop a value in place
#[stabby::stabby]
#[derive(Clone, Copy)]
pub struct VtDrop {
    pub drop: unsafe extern "C" fn(&mut ()),
}
impl PartialEq for VtDrop {
    fn eq(&self, other: &Self) -> bool {
        core::ptr::eq(
            self.drop as *const unsafe extern "C" fn(&mut ()),
            other.drop as *const unsafe extern "C" fn(&mut ()),
        )
    }
}
impl<T> IConstConstructor<VtDrop> for T {
    const VTABLE: &'static VtDrop = &VtDrop {
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
impl<Tail: TransitiveDeref<Vt, N>, Vt, N> TransitiveDeref<Vt, T<N>> for VtSend<Tail> {
    fn tderef(&self) -> &Vt {
        self.0.tderef()
    }
}
impl<Head, Tail> From<crate::type_layouts::vtable::VtSend<VTable<Head, Tail>>>
    for VTable<Head, Tail>
{
    fn from(value: VtSend<VTable<Head, Tail>>) -> Self {
        value.0
    }
}
impl<T: IConstConstructor<Vt> + Send, Vt: Copy + 'static> IConstConstructor<VtSend<Vt>> for T {
    const VTABLE: &'static VtSend<Vt> = &VtSend(*T::VTABLE);
}

/// A marker for vtables for types that are `Sync`
#[stabby::stabby]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct VtSync<T>(T);
impl<T: IConstConstructor<Vt> + Sync, Vt: Copy + 'static> IConstConstructor<VtSync<Vt>> for T {
    const VTABLE: &'static VtSync<Vt> = &VtSync(*T::VTABLE);
}
impl<Tail: TransitiveDeref<Vt, N>, Vt, N> TransitiveDeref<Vt, T<N>> for VtSync<Tail> {
    fn tderef(&self) -> &Vt {
        self.0.tderef()
    }
}
impl<Head, Tail> From<crate::type_layouts::vtable::VtSync<VtSend<VTable<Head, Tail>>>>
    for VTable<Head, Tail>
{
    fn from(value: VtSync<VtSend<VTable<Head, Tail>>>) -> Self {
        value.0 .0
    }
}
impl<Head, Tail> From<crate::type_layouts::vtable::VtSync<VtSend<VTable<Head, Tail>>>>
    for VtSend<VTable<Head, Tail>>
{
    fn from(value: VtSync<VtSend<VTable<Head, Tail>>>) -> Self {
        value.0
    }
}
impl<Head, Tail> From<crate::type_layouts::vtable::VtSync<VTable<Head, Tail>>>
    for VTable<Head, Tail>
{
    fn from(value: VtSync<VTable<Head, Tail>>) -> Self {
        value.0
    }
}
