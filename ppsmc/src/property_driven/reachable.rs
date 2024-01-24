use super::PPSMC;
use crate::Bdd;
use arun::async_spawn;
use std::{sync::Arc, time::Instant};
use sylvan::LaceWorkerContext;

impl PPSMC {
    pub fn lace_post_reachable(
        &mut self,
        mut context: LaceWorkerContext,
        from: &[Bdd],
    ) -> Vec<Bdd> {
        let mut frontier = from.to_vec();
        let partitioned_len = from.len();
        let mut reach = frontier.clone();
        let mut tmp_reach = vec![self.manager.constant(false); partitioned_len];
        let mut post_deep = 0;
        loop {
            post_deep += 1;
            if self.args.verbose {
                dbg!(post_deep);
            }
            let start = Instant::now();
            let mut tmp = vec![self.manager.constant(false); partitioned_len];
            for i in 0..partitioned_len {
                for (next, label) in self.automata.forward[i].iter() {
                    let update = &frontier[i] & label & !&tmp_reach[*next];
                    tmp[*next] |= &update;
                    tmp_reach[*next] |= update;
                }
            }
            if tmp.iter().all(|bdd| bdd.is_constant(false)) {
                break reach;
            }
            self.statistic.post_propagate_time += start.elapsed();
            let start = Instant::now();
            for i in 0..partitioned_len {
                let bdd = tmp[i].clone();
                let mut reach = reach[i].clone();
                let fsmbdd = self.workers[i].fsmbdd.clone();
                context.lace_spawn(move |_| {
                    let image = fsmbdd.post_image(&bdd);
                    reach |= &image;
                    (reach, image)
                });
            }
            let reach_update: Vec<(Bdd, Bdd)> = context.lace_sync_multi(partitioned_len);
            self.statistic.post_image_time += start.elapsed();
            frontier.clear();
            reach = Vec::new();
            for (reach_bdd, update) in reach_update {
                reach.push(reach_bdd);
                frontier.push(update);
            }
        }
    }

    fn lace_pre_iteration(
        &mut self,
        mut context: LaceWorkerContext,
        states: Vec<Bdd>,
        reach: &[Bdd],
        constraint: &[Bdd],
    ) -> (Vec<Bdd>, Vec<Bdd>) {
        let partitioned_len = states.len();
        let states = Arc::new(states);
        for i in 0..partitioned_len {
            let worker = self.workers[i].clone();
            let reach = reach[i].clone();
            let states = states.clone();
            let constraint = constraint[i].clone();
            context.lace_spawn(move |_| {
                let (reach, mut new_frontier) = worker.propagate_value(reach, states, constraint);
                if !new_frontier.is_constant(false) {
                    new_frontier = worker.fsmbdd.pre_image(&new_frontier);
                }
                (reach, new_frontier)
            })
        }
        let res = context.lace_sync_multi::<(Bdd, Bdd)>(partitioned_len);
        let mut reach = Vec::new();
        let mut new_frontier = Vec::new();
        for (r, f) in res.into_iter() {
            reach.push(r);
            new_frontier.push(f);
        }
        (reach, new_frontier)
    }

    pub fn lace_pre_reachable(
        &mut self,
        mut context: LaceWorkerContext,
        from: &[Bdd],
        constraint: &[Bdd],
    ) -> Vec<Bdd> {
        let partitioned_len = from.len();
        let mut frontier = from.to_vec();
        let mut reach = vec![self.manager.constant(false); partitioned_len];
        let mut y = 0;
        for i in 0..partitioned_len {
            let fsmbdd = self.workers[i].fsmbdd.clone();
            let x = frontier[i].clone();
            context.lace_spawn(move |_| fsmbdd.pre_image(&x));
        }
        frontier = context.lace_sync_multi(partitioned_len);
        loop {
            y += 1;
            if self.args.verbose {
                dbg!(y);
            }
            let start = Instant::now();
            let new_frontier;
            (reach, new_frontier) = self.lace_pre_iteration(context, frontier, &reach, constraint);
            self.statistic.pre_propagate_time += start.elapsed();
            if new_frontier.iter().all(|bdd| bdd.is_constant(false)) {
                break;
            }
            frontier = new_frontier;
        }
        reach
    }
}

impl PPSMC {
    pub async fn new_parallel_post_reachable(&mut self, from: &[Bdd]) -> Vec<Bdd> {
        let constraint = vec![self.manager.constant(true); from.len()];
        self.new_parallel_reachable(from, &constraint, true).await
    }

    pub async fn new_parallel_pre_reachable(
        &mut self,
        from: &[Bdd],
        constraint: &[Bdd],
    ) -> Vec<Bdd> {
        self.new_parallel_reachable(from, constraint, false).await
    }

    async fn new_parallel_reachable(
        &mut self,
        from: &[Bdd],
        constraint: &[Bdd],
        forward: bool,
    ) -> Vec<Bdd> {
        for worker in self.workers.iter_mut() {
            Arc::get_mut(worker).unwrap().reset().await
        }
        let mut joins = Vec::new();
        for i in 0..self.workers.len() {
            let init = from[i].clone();
            let constraint = constraint[i].clone();
            let mut worker = self.workers[i].clone();
            joins.push(async_spawn(async move {
                if forward {
                    unsafe { Arc::get_mut_unchecked(&mut worker) }
                        .post_reachable(init)
                        .await
                } else {
                    unsafe { Arc::get_mut_unchecked(&mut worker) }
                        .pre_reachable(init, constraint)
                        .await
                }
            }));
        }
        let mut res = Vec::new();
        for join in joins {
            res.push(join.await);
        }
        res
    }
}
