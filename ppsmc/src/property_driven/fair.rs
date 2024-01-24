use super::PPSMC;
use crate::Bdd;
use sylvan::LaceWorkerContext;

impl PPSMC {
    pub fn fair_states(&mut self, mut context: LaceWorkerContext, init_reach: &[Bdd]) -> Vec<Bdd> {
        let mut fair_states = vec![self.manager.constant(false); self.automata.num_state()];
        for state in self.automata.accepting_states.iter() {
            fair_states[*state] = init_reach[*state].clone();
        }
        let mut x = 0;
        loop {
            x += 1;
            if self.args.verbose {
                dbg!(x);
            }
            let backward = self.lace_pre_reachable(context, &fair_states, init_reach);
            fair_states.iter().zip(backward.iter()).for_each(|(x, y)| {
                let x = x.clone();
                let y = y.clone();
                context.lace_spawn(|_| x & y)
            });
            let new_fair_states: Vec<Bdd> = context.lace_sync_multi(fair_states.len());
            if fair_states == new_fair_states {
                break;
            }
            fair_states = new_fair_states;
        }
        fair_states
    }

    pub async fn async_fair_states(&mut self, init_reach: &[Bdd]) -> Vec<Bdd> {
        let mut fair_states = vec![self.manager.constant(false); self.automata.num_state()];
        for state in self.automata.accepting_states.iter() {
            fair_states[*state] = init_reach[*state].clone();
        }
        let mut x = 0;
        loop {
            x += 1;
            if self.args.verbose {
                dbg!(x);
            }
            let new_fair_states = self
                .new_parallel_pre_reachable(&fair_states, init_reach)
                .await;
            if fair_states == new_fair_states {
                break;
            }
            fair_states = new_fair_states;
        }
        fair_states
    }
}
