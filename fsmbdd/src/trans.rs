use bdds::{Bdd, BddManager};
use ordered_float::NotNan;
use std::{
    collections::{BinaryHeap, HashSet},
    ops::{BitAnd, BitOr, BitXor, Not},
};

#[derive(Clone, Debug)]
pub enum TransBddMethod {
    Partition,
    Monolithic,
}

#[derive(Clone, Debug)]
pub struct Trans<BM: BddManager>
where
    for<'a, 'b> &'a BM::Bdd: Not<Output = BM::Bdd>
        + BitAnd<BM::Bdd, Output = BM::Bdd>
        + BitAnd<&'b BM::Bdd, Output = BM::Bdd>
        + BitOr<BM::Bdd, Output = BM::Bdd>
        + BitOr<&'b BM::Bdd, Output = BM::Bdd>
        + BitXor<BM::Bdd, Output = BM::Bdd>
        + BitXor<&'b BM::Bdd, Output = BM::Bdd>,
{
    manager: BM,
    pub(crate) trans: Vec<BM::Bdd>,
    pre_eliminate: Vec<Vec<usize>>,
    post_eliminate: Vec<Vec<usize>>,
}

impl<BM: BddManager> Trans<BM>
where
    for<'a, 'b> &'a BM::Bdd: Not<Output = BM::Bdd>
        + BitAnd<BM::Bdd, Output = BM::Bdd>
        + BitAnd<&'b BM::Bdd, Output = BM::Bdd>
        + BitOr<BM::Bdd, Output = BM::Bdd>
        + BitOr<&'b BM::Bdd, Output = BM::Bdd>
        + BitXor<BM::Bdd, Output = BM::Bdd>
        + BitXor<&'b BM::Bdd, Output = BM::Bdd>,
{
    fn build_schedule(trans: &[BM::Bdd], vars: Vec<usize>) -> Vec<Vec<usize>> {
        let mut res = Vec::new();
        let mut vars: HashSet<usize> = HashSet::from_iter(vars);
        for tran in trans.iter().rev() {
            res.push(Vec::from_iter(vars.iter().copied()));
            let support = tran.support_index();
            for v in support.iter() {
                vars.remove(v);
            }
        }
        res.reverse();
        res
    }

    fn build(manager: &BM, trans: Vec<BM::Bdd>) -> Self {
        println!("build num trans: {}", trans.len());
        let pre_eliminate = Self::build_schedule(&trans, manager.next_state_vars());
        let post_eliminate = Self::build_schedule(&trans, manager.state_vars());
        Self {
            manager: manager.clone(),
            pre_eliminate,
            trans,
            post_eliminate,
        }
    }

    fn monolithic_new(manager: &BM, trans: Vec<BM::Bdd>) -> Self {
        let mut res = manager.constant(true);
        for (i, tran) in trans.iter().enumerate() {
            dbg!(i);
            res &= tran;
        }
        Self::build(manager, vec![res])
    }

    fn compute_affinity(a: &BM::Bdd, b: &BM::Bdd) -> f64 {
        let a: HashSet<usize> = HashSet::from_iter(a.support_index());
        let b = HashSet::from_iter(b.support_index());
        let i = a.intersection(&b).count();
        let u = a.union(&b).count();
        i as f64 / u as f64
    }

    fn partition_new(manager: &BM, mut trans: Vec<BM::Bdd>) -> Self {
        assert!(trans.len() <= 100);
        const THRESHOLD: usize = 1000;
        let mut trans_exist = HashSet::new();
        let mut res = Vec::new();
        let mut affinity_heap = BinaryHeap::new();
        for i in 0..trans.len() {
            if trans[i].size() > THRESHOLD {
                res.push(trans[i].clone())
            } else {
                for exist in trans_exist.iter() {
                    let affinity = Self::compute_affinity(&trans[i], &trans[*exist]);
                    let affinity = NotNan::new(affinity).unwrap();
                    affinity_heap.push((affinity, *exist, i));
                }
                trans_exist.insert(i);
            }
        }
        while let Some((_, x, y)) = affinity_heap.pop() {
            if trans_exist.contains(&x) && trans_exist.contains(&y) {
                let xy = &trans[x] & &trans[y];
                assert!(trans_exist.remove(&x) && trans_exist.remove(&y));
                if xy.size() > THRESHOLD {
                    res.push(xy);
                } else {
                    let xy_index = trans.len();
                    trans.push(xy);
                    for exist in trans_exist.iter() {
                        let affinity = Self::compute_affinity(&trans[xy_index], &trans[*exist]);
                        let affinity = NotNan::new(affinity).unwrap();
                        affinity_heap.push((affinity, *exist, xy_index));
                    }
                    trans_exist.insert(xy_index);
                }
            }
        }
        if trans_exist.len() == 1 {
            let index = *trans_exist.iter().next().unwrap();
            res.push(trans[index].clone());
            trans_exist.remove(&index);
        }
        assert!(trans_exist.is_empty());
        Self::build(manager, res)
    }

    pub fn new(manager: &BM, trans: Vec<BM::Bdd>, method: TransBddMethod) -> Self {
        let trans = {
            let mut res = vec![];
            for tran in trans {
                if !res.contains(&tran) {
                    res.push(tran);
                }
            }
            res
        };
        dbg!(trans.len());
        match method {
            TransBddMethod::Partition => Self::partition_new(manager, trans),
            TransBddMethod::Monolithic => Self::monolithic_new(manager, trans),
        }
    }

    pub fn pre_image(&self, state: &BM::Bdd) -> BM::Bdd {
        if self.trans.len() == 1 {
            state.pre_image(&self.trans[0])
        } else {
            let mut res = state.next_state();
            for i in 0..self.pre_eliminate.len() {
                res = res.and_abstract(&self.trans[i], self.pre_eliminate[i].iter().copied());
            }
            res
        }
    }

    pub fn post_image(&self, state: &BM::Bdd) -> BM::Bdd {
        if self.trans.len() == 1 {
            state.post_image(&self.trans[0])
        } else {
            let mut res = state.clone();
            for i in 0..self.post_eliminate.len() {
                res = res.and_abstract(&self.trans[i], self.post_eliminate[i].iter().copied());
            }
            res.previous_state()
        }
    }

    pub fn product(&self, other: &Self) -> Self {
        assert!(self.manager == other.manager);
        let trans = if self.trans.len() == 1 && other.trans.len() == 1 {
            vec![&self.trans[0] & &other.trans[0]]
        } else {
            let mut trans = self.trans.clone();
            trans.extend(other.trans.clone());
            trans
        };
        Self::build(&self.manager, trans)
    }

    pub fn clone_with_new_manager(&self, manager: &BM) -> Self {
        let trans = self.trans.iter().map(|t| manager.translocate(t)).collect();
        Self {
            manager: manager.clone(),
            trans,
            pre_eliminate: self.pre_eliminate.clone(),
            post_eliminate: self.post_eliminate.clone(),
        }
    }
}
