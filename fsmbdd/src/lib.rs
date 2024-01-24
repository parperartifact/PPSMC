mod trans;
pub use trans::*;

use bdds::BddManager;
use std::{
    collections::HashMap,
    ops::{BitAnd, BitOr, BitXor, Not},
};

#[derive(Clone, Debug)]
pub struct FsmBdd<BM: BddManager>
where
    for<'a, 'b> &'a BM::Bdd: Not<Output = BM::Bdd>
        + BitAnd<BM::Bdd, Output = BM::Bdd>
        + BitAnd<&'b BM::Bdd, Output = BM::Bdd>
        + BitOr<BM::Bdd, Output = BM::Bdd>
        + BitOr<&'b BM::Bdd, Output = BM::Bdd>
        + BitXor<BM::Bdd, Output = BM::Bdd>
        + BitXor<&'b BM::Bdd, Output = BM::Bdd>,
{
    pub symbols: HashMap<String, usize>,
    pub manager: BM,
    pub init: BM::Bdd,
    pub invariants: BM::Bdd,
    pub trans: Trans<BM>,
    pub justice: Vec<BM::Bdd>,
}

impl<BM: BddManager> FsmBdd<BM>
where
    for<'a, 'b> &'a BM::Bdd: Not<Output = BM::Bdd>
        + BitAnd<BM::Bdd, Output = BM::Bdd>
        + BitAnd<&'b BM::Bdd, Output = BM::Bdd>
        + BitOr<BM::Bdd, Output = BM::Bdd>
        + BitOr<&'b BM::Bdd, Output = BM::Bdd>
        + BitXor<BM::Bdd, Output = BM::Bdd>
        + BitXor<&'b BM::Bdd, Output = BM::Bdd>,
{
    pub fn product(&self, other: &Self) -> Self {
        assert!(self.manager == other.manager);
        let mut symbols = self.symbols.clone();
        symbols.extend(other.symbols.clone());
        let init = &self.init & &other.init;
        let invariants = &self.invariants & &other.invariants;
        let trans = self.trans.product(&other.trans);
        let mut justice = self.justice.clone();
        justice.extend(other.justice.clone());
        Self {
            symbols,
            manager: self.manager.clone(),
            init,
            trans,
            justice,
            invariants,
        }
    }

    pub fn pre_image(&self, state: &BM::Bdd) -> BM::Bdd {
        self.trans.pre_image(&(state & &self.invariants)) & &self.invariants
    }

    pub fn post_image(&self, state: &BM::Bdd) -> BM::Bdd {
        self.trans.post_image(&(state & &self.invariants)) & &self.invariants
    }

    pub fn reachable_with_constrain(
        &self,
        state: &BM::Bdd,
        forward: bool,
        contain_from: bool,
        constrain: &BM::Bdd,
    ) -> BM::Bdd {
        let mut frontier = state.clone() & constrain & &self.invariants;
        let mut reach = if contain_from {
            frontier.clone()
        } else {
            self.manager.constant(false)
        };
        // let mut x = 0;
        loop {
            // x += 1;
            // dbg!(x);
            let new_frontier = if forward {
                self.post_image(&frontier)
            } else {
                self.pre_image(&frontier)
            } & constrain;
            let new_frontier = new_frontier & !&reach;
            if new_frontier == self.manager.constant(false) {
                break reach;
            }
            reach |= &new_frontier;
            frontier = new_frontier;
        }
    }

    pub fn reachable(&self, state: &BM::Bdd, forward: bool, contain_from: bool) -> BM::Bdd {
        self.reachable_with_constrain(state, forward, contain_from, &self.manager.constant(true))
    }

    pub fn reachable_from_init(&self) -> BM::Bdd {
        self.reachable(&self.init, true, true)
    }

    pub fn fair_cycle_with_constrain(&self, constrain: &BM::Bdd) -> BM::Bdd {
        let mut res = constrain.clone();
        // let mut y = 0;
        loop {
            // y += 1;
            // dbg!(y);
            let mut new = res.clone();
            for fair in self.justice.iter() {
                let fair = fair & &res;
                let backward = self.reachable_with_constrain(&fair, false, false, constrain);
                new &= backward;
            }
            if new == res {
                break res;
            }
            res = new
        }
    }

    pub fn fair_cycle(&self) -> BM::Bdd {
        self.fair_cycle_with_constrain(&self.manager.constant(true))
    }

    pub fn clone_with_new_manager(&self) -> Self {
        let manager = BM::new();
        let trans = self.trans.clone_with_new_manager(&manager);
        let init = manager.translocate(&self.init);
        let justice = self
            .justice
            .iter()
            .map(|justice| manager.translocate(justice))
            .collect();
        let invariants = manager.translocate(&self.invariants);
        Self {
            symbols: self.symbols.clone(),
            manager,
            init,
            trans,
            justice,
            invariants,
        }
    }
}
