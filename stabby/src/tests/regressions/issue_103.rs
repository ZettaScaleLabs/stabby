#[test]
fn main() {
    #[crate::stabby]
    pub trait Iface {
        extern "C" fn strike(&mut self);
    }

    #[repr(C)]
    struct Victim<'a> {
        ptr: crate::dynptr!(&'a mut dyn Iface),
        target: Box<u8>,
    }

    #[repr(C)]
    struct Attack {
        pad1: isize,
        pad2: isize,
        target: isize,
    }

    impl Iface for Attack {
        extern "C" fn strike(&mut self) {
            self.target = 0;
        }
    }

    let mut val = Attack {
        pad1: 0,
        pad2: 0,
        target: 0,
    };
    let mut vic = Victim {
        ptr: (&mut val).into(),
        target: Box::new(42),
    };
    vic.ptr.strike();
    println!("The answer is {}", vic.target);
}
