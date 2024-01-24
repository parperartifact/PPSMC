use crate::{Bdd, BddManager};

impl Bdd for sylvan::Bdd {
    fn size(&self) -> usize {
        self.size()
    }

    fn is_constant(&self, val: bool) -> bool {
        self.is_constant(val)
    }

    fn if_then_else(&self, _then: &Self, _else: &Self) -> Self {
        self.if_then_else(_then, _else)
    }

    fn and_abstract<I: IntoIterator<Item = usize>>(&self, f: &Self, vars: I) -> Self {
        self.and_abstract(f, vars)
    }

    fn previous_state(&self) -> Self {
        self.previous_state()
    }

    fn next_state(&self) -> Self {
        self.next_state()
    }

    fn pre_image(&self, trans: &Self) -> Self {
        self.pre_image(trans)
    }

    fn post_image(&self, trans: &Self) -> Self {
        self.post_image(trans)
    }

    fn support(&self) -> Self {
        self.support()
    }

    fn support_index(&self) -> Vec<usize> {
        self.support_index()
    }
}

impl BddManager for sylvan::Sylvan {
    type Bdd = sylvan::Bdd;

    fn new() -> Self {
        Self::new()
    }

    fn new_with_capacity(_capacity: usize) -> Self {
        todo!()
    }

    fn constant(&self, val: bool) -> Self::Bdd {
        Self.constant(val)
    }

    fn ith_var(&self, var: usize) -> Self::Bdd {
        Self.ith_var(var)
    }

    fn num_var(&self) -> usize {
        Self::num_var()
    }

    fn translocate(&self, bdd: &Self::Bdd) -> Self::Bdd {
        bdd.clone()
    }
}
