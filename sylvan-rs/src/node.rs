use crate::Sylvan;
use std::{
    fmt::Debug,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not},
};
use sylvan_sys::{
    bdd::{
        Sylvan_and, Sylvan_and_exists, Sylvan_compose, Sylvan_ite, Sylvan_not, Sylvan_or,
        Sylvan_relnext, Sylvan_relprev, Sylvan_xor,
    },
    mtbdd::{
        Sylvan_map_add, Sylvan_map_empty, Sylvan_nodecount, Sylvan_protect, Sylvan_support,
        Sylvan_unprotect,
    },
    *,
};

pub struct Bdd {
    pub(crate) node: Box<u64>,
}

unsafe impl Sync for Bdd {}

unsafe impl Send for Bdd {}

impl Bdd {
    pub(crate) fn new(node: MTBDD) -> Self {
        let mut node = Box::new(node);
        unsafe {
            Sylvan_protect(node.as_mut());
        }
        Self { node }
    }
}

impl Drop for Bdd {
    fn drop(&mut self) {
        unsafe {
            Sylvan_unprotect(self.node.as_mut());
        }
    }
}

impl AsRef<Bdd> for Bdd {
    fn as_ref(&self) -> &Bdd {
        self
    }
}

impl AsMut<Bdd> for Bdd {
    fn as_mut(&mut self) -> &mut Bdd {
        self
    }
}

impl Clone for Bdd {
    fn clone(&self) -> Self {
        Self::new(*self.node)
    }
}

impl Debug for Bdd {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // write!(f, "{:?}", self.get_all_minterms())
        todo!()
    }
}

impl PartialEq for Bdd {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}

impl Eq for Bdd {}

impl Not for Bdd {
    type Output = Bdd;

    fn not(self) -> Self::Output {
        let node = unsafe { Sylvan_not(*self.node) };
        Bdd::new(node)
    }
}

impl Not for &Bdd {
    type Output = Bdd;

    fn not(self) -> Self::Output {
        let node = unsafe { Sylvan_not(*self.node) };
        Bdd::new(node)
    }
}

impl<T: AsRef<Bdd>> BitAnd<T> for Bdd {
    type Output = Bdd;

    fn bitand(self, rhs: T) -> Self::Output {
        let res = unsafe { Sylvan_and(*self.node, *rhs.as_ref().node) };
        Bdd::new(res)
    }
}

impl<T: AsRef<Bdd>> BitAnd<T> for &Bdd {
    type Output = Bdd;

    fn bitand(self, rhs: T) -> Self::Output {
        let res = unsafe { Sylvan_and(*self.node, *rhs.as_ref().node) };
        Bdd::new(res)
    }
}

impl<T: AsRef<Bdd>> BitAndAssign<T> for Bdd {
    fn bitand_assign(&mut self, rhs: T) {
        *self = self.as_ref() & rhs.as_ref();
    }
}

impl<T: AsRef<Bdd>> BitOr<T> for Bdd {
    type Output = Bdd;

    fn bitor(self, rhs: T) -> Self::Output {
        let res = unsafe { Sylvan_or(*self.node, *rhs.as_ref().node) };
        Bdd::new(res)
    }
}

impl<T: AsRef<Bdd>> BitOr<T> for &Bdd {
    type Output = Bdd;

    fn bitor(self, rhs: T) -> Self::Output {
        let res = unsafe { Sylvan_or(*self.node, *rhs.as_ref().node) };
        Bdd::new(res)
    }
}

impl<T: AsRef<Bdd>> BitOrAssign<T> for Bdd {
    fn bitor_assign(&mut self, rhs: T) {
        *self = self.as_ref() | rhs.as_ref();
    }
}

impl<T: AsRef<Bdd>> BitXor<T> for Bdd {
    type Output = Bdd;

    fn bitxor(self, rhs: T) -> Self::Output {
        let res = unsafe { Sylvan_xor(*self.node, *rhs.as_ref().node) };
        Bdd::new(res)
    }
}

impl<T: AsRef<Bdd>> BitXor<T> for &Bdd {
    type Output = Bdd;

    fn bitxor(self, rhs: T) -> Self::Output {
        let res = unsafe { Sylvan_xor(*self.node, *rhs.as_ref().node) };
        Bdd::new(res)
    }
}

impl<T: AsRef<Bdd>> BitXorAssign<T> for Bdd {
    fn bitxor_assign(&mut self, rhs: T) {
        *self = self.as_ref() ^ rhs.as_ref();
    }
}

impl Bdd {
    pub fn is_constant(&self, value: bool) -> bool {
        *self == Sylvan.constant(value)
    }

    pub fn size(&self) -> usize {
        unsafe { Sylvan_nodecount(*self.node) }
    }

    pub fn if_then_else(&self, _then: &Bdd, _else: &Bdd) -> Self {
        let res = unsafe { Sylvan_ite(*self.node, *_then.node, *_else.node) };
        Bdd::new(res)
    }

    pub fn and_abstract<I: IntoIterator<Item = usize>>(&self, x: &Bdd, cube: I) -> Self {
        let cube = cube.into_iter().map(|x| (x, true));
        let cube = Sylvan.cube(cube);
        let res = unsafe { Sylvan_and_exists(*self.node, *x.node, *cube.node) };
        Bdd::new(res)
    }

    pub fn support(&self) -> Self {
        let res = unsafe { Sylvan_support(*self.node) };
        Bdd::new(res)
    }

    pub fn support_index(&self) -> Vec<usize> {
        let mut res = Vec::new();
        let support = self.support();
        for i in 0..Sylvan::num_var() {
            if (&support & !Sylvan.ith_var(i)).is_constant(false) {
                res.push(i);
            }
        }
        res
    }

    pub fn next_state(&self) -> Self {
        let map = unsafe { Sylvan_map_empty() };
        let mut map = Self::new(map);
        for i in (0..Sylvan::num_var()).step_by(2) {
            let var = *Sylvan.ith_var(i + 1).node;
            let node = unsafe { Sylvan_map_add(*map.node, i as _, var) };
            map = Self::new(node);
        }
        let res = unsafe { Sylvan_compose(*self.node, *map.node) };
        Self::new(res)
    }

    pub fn previous_state(&self) -> Self {
        let map = unsafe { Sylvan_map_empty() };
        let mut map = Self::new(map);
        for i in (1..Sylvan::num_var()).step_by(2) {
            let var = *Sylvan.ith_var(i - 1).node;
            let node = unsafe { Sylvan_map_add(*map.node, i as _, var) };
            map = Self::new(node);
        }
        let res = unsafe { Sylvan_compose(*self.node, *map.node) };
        Self::new(res)
    }

    pub fn post_image(&self, tran: &Bdd) -> Self {
        let res = unsafe { Sylvan_relnext(*self.node, *tran.node, SYLVAN_FALSE) };
        Bdd::new(res)
    }

    pub fn pre_image(&self, tran: &Bdd) -> Self {
        let res = unsafe { Sylvan_relprev(*tran.node, *self.node, SYLVAN_FALSE) };
        Bdd::new(res)
    }
}

impl Bdd {
    pub unsafe fn get_raw(&self) -> u64 {
        *self.node
    }

    pub unsafe fn new_from_raw(raw: u64) -> Self {
        Self::new(raw)
    }
}

#[cfg(test)]
mod tests {
    use crate::Sylvan;

    #[test]
    fn test_basic() {
        Sylvan::init(1);
        let a = Sylvan.ith_var(0);
        let b = Sylvan.ith_var(1);
        let c = (&a & &b) | (!&a & !&b);
        dbg!(c);
    }
}
