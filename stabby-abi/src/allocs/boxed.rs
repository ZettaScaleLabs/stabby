impl crate::IPtr for Box<()> {
    unsafe fn as_ref<U>(&self) -> &U {
        let this: &() = self;
        core::mem::transmute(this)
    }
}
impl crate::IPtrMut for Box<()> {
    unsafe fn as_mut<U>(&mut self) -> &mut U {
        let this: &mut () = self;
        core::mem::transmute(this)
    }
}
impl crate::IPtrOwned for Box<()> {
    fn drop(this: &mut core::mem::ManuallyDrop<Self>, drop: unsafe extern "C" fn(&mut ())) {
        unsafe {
            (drop)(this);
            core::mem::ManuallyDrop::drop(this);
        }
    }
}

impl<T> crate::IntoDyn for Box<T> {
    type Anonymized = Box<()>;
    type Target = T;
    fn anonimize(self) -> Self::Anonymized {
        unsafe { core::mem::transmute(self) }
    }
}
