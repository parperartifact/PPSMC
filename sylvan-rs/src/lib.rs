mod node;
pub use node::*;
mod lace;
pub use lace::*;

use sylvan_sys::{bdd::Sylvan_cube, common::*, lace::*, mtbdd::*, MTBDD_FALSE, MTBDD_TRUE};

static mut VAR_COUNT: usize = 0;
static mut INIT: bool = false;

#[derive(Debug, Clone, PartialEq)]
pub struct Sylvan;

impl Sylvan {
    pub fn new() -> Self {
        if unsafe { !INIT } {
            Self::init(16);
            unsafe { INIT = true };
        }
        Self
    }

    pub fn init(num_worker: usize) -> Self {
        unsafe {
            assert!(!INIT);
            Lace_start(num_worker as _, 0);
            Sylvan_set_limits(1024 * 1024 * 1024, 1, 5);
            Sylvan_init_package();
            Sylvan_init_mtbdd();
            Sylvan_gc_enable();
            INIT = true;
        };
        Self
    }

    pub fn constant(&self, val: bool) -> Bdd {
        let val = if val { MTBDD_TRUE } else { MTBDD_FALSE };
        Bdd::new(val)
    }

    pub fn ith_var(&self, i: usize) -> Bdd {
        unsafe {
            if VAR_COUNT < i + 1 {
                VAR_COUNT = i + 1;
            }
        }
        let node = unsafe { Sylvan_ithvar(i as _) };
        Bdd::new(node)
    }

    pub fn num_var() -> usize {
        unsafe { VAR_COUNT }
    }

    pub fn cube<I: IntoIterator<Item = (usize, bool)>>(&self, vars: I) -> Bdd {
        let mut vars: Vec<(usize, bool)> = vars.into_iter().collect();
        vars.sort_by_key(|(var, _)| *var);
        let mut set = unsafe { Sylvan_set_empty() };
        let mut cube = Vec::new();
        for (var, pol) in vars {
            cube.push(pol as u8);
            set = unsafe { Sylvan_set_add(set, var as _) };
        }
        Bdd::new(unsafe { Sylvan_cube(set, cube.as_ptr() as *mut u8) })
    }

    pub fn translocate(&self, bdd: &Bdd) -> Bdd {
        bdd.clone()
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
        let c = &a & &b;
        let d = !(!a | !b);
        assert_eq!(c, d);
        let a = Sylvan.constant(true);
        let b = Sylvan.constant(false);
        assert_eq!(&a & &c, c);
        assert_eq!(a, !b);
    }

    #[test]
    fn test_image() {
        Sylvan::init(1);
        let a = Sylvan.ith_var(0);
        let ap = Sylvan.ith_var(1);
        let b = Sylvan.ith_var(2);
        let bp = Sylvan.ith_var(3);
        let s = &a & &b;
        let t = &a & &b & !ap & !bp;
        let next = s.post_image(&t);
        assert_eq!(next, !&a & !&b);
        let s = !&a & !&b;
        let pre = s.pre_image(&t);
        assert_eq!(pre, &a & &b);
        let s = !&a & &b;
        let pre = s.pre_image(&t);
        assert_eq!(pre, Sylvan.constant(false));
    }

    #[test]
    fn test_next_state() {
        Sylvan::init(1);
        let a = Sylvan.ith_var(0);
        let ap = Sylvan.ith_var(1);
        let b = Sylvan.ith_var(2);
        let bp = Sylvan.ith_var(3);
        let s = &a & &b;
        assert_eq!(s.next_state(), &ap & &bp);
        let s = !&a & !&b;
        assert_eq!(s.next_state(), !&ap & !&bp);
        let sp = &ap & &bp;
        assert_eq!(sp.previous_state(), &a & &b);
        let sp = !&ap & !&bp;
        assert_eq!(sp.previous_state(), !&a & !&b);
    }

    #[test]
    fn test_support() {
        Sylvan::init(1);
        let a = Sylvan.ith_var(0);
        let _ = Sylvan.ith_var(1);
        let b = Sylvan.ith_var(2);
        let s = &a & !&b;
        assert_eq!(s.support(), &a & &b);
        assert_eq!(s.support_index(), vec![0, 2]);
    }

    #[test]
    fn test_cube() {
        Sylvan::init(1);
        let a = Sylvan.ith_var(0);
        let _ = Sylvan.ith_var(1);
        let b = Sylvan.ith_var(2);
        assert_eq!(Sylvan.cube([(0, true), (2, true)]), &a & &b);
    }

    #[test]
    fn test_bdd_size() {
        Sylvan::init(1);
        let a = Sylvan.ith_var(0);
        let _ = Sylvan.ith_var(1);
        let b = Sylvan.ith_var(2);
        assert_eq!((a & b).size(), 3);
    }
}
