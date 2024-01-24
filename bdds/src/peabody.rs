// use crate::{Bdd, BddManager};
// use peabody::Peabody;

// impl Bdd for peabody::Bdd {
//     fn is_constant(&self, val: bool) -> bool {
//         self.is_constant(val)
//     }

//     fn if_then_else(&self, _then: &Self, _else: &Self) -> Self {
//         (self & _then) | (!self & _else)
//     }

//     fn and_abstract<I: IntoIterator<Item = usize>>(&self, _f: &Self, _vars: I) -> Self {
//         todo!()
//     }

//     fn previous_state(&self) -> Self {
//         self.original_state()
//     }

//     fn next_state(&self) -> Self {
//         self.next_state()
//     }

//     fn pre_image(&self, trans: &Self) -> Self {
//         self.pre_image(trans)
//     }

//     fn post_image(&self, trans: &Self) -> Self {
//         self.post_image(trans)
//     }
// }

// impl BddManager for Peabody {
//     type Bdd = peabody::Bdd;

//     fn new() -> Self {
//         Peabody::new()
//     }

//     fn new_with_capacity(capacity: usize) -> Self {
//         Peabody::new_with_capacity(capacity)
//     }

//     fn constant(&self, val: bool) -> Self::Bdd {
//         self.constant(val)
//     }

//     fn ith_var(&self, var: usize) -> Self::Bdd {
//         self.ith_var(var)
//     }

//     fn num_var(&self) -> usize {
//         todo!()
//     }

//     fn translocate(&self, _bdd: &Self::Bdd) -> Self::Bdd {
//         todo!()
//     }
// }
