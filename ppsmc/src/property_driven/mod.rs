mod async_worker;
mod fair;
mod reachable;
mod statistic;
mod worker;

use self::{async_worker::AsyncWorker, statistic::Statistic, worker::Worker};
use crate::{automata::BuchiAutomata, command::Args, ltl::ltl_to_automata_preprocess, BddManager};
use arun::async_block_on;
use fsmbdd::FsmBdd;
use smv::{bdd::SmvBdd, Expr, Prefix, Smv};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use sylvan::{lace_run, Sylvan};

pub struct PPSMC {
    manager: BddManager,
    fsmbdd: FsmBdd<BddManager>,
    automata: BuchiAutomata,
    args: Args,
    statistic: Statistic,
    workers: Vec<Arc<Worker>>,
}

impl PPSMC {
    pub fn new(
        manager: BddManager,
        fsmbdd: FsmBdd<BddManager>,
        automata: BuchiAutomata,
        args: Args,
    ) -> Self {
        let workers = Worker::create_workers(&fsmbdd, &automata)
            .into_iter()
            .map(Arc::new)
            .collect();
        Self {
            manager,
            fsmbdd,
            automata,
            args,
            workers,
            statistic: Statistic::default(),
        }
    }

    pub fn check(&mut self) -> bool {
        let mut reach = vec![self.manager.constant(false); self.automata.num_state()];
        for init_state in self.automata.init_states.iter() {
            reach[*init_state] |= &self.fsmbdd.init;
        }
        let start = Instant::now();
        reach = if self.args.old_impl {
            lace_run(|context| self.lace_post_reachable(context, &reach))
        } else {
            async_block_on(self.new_parallel_post_reachable(&reach))
        };
        self.statistic.post_reachable_time += start.elapsed();
        let start = Instant::now();
        let fair_states = if self.args.old_impl {
            lace_run(|context| self.fair_states(context, &reach))
        } else {
            async_block_on(self.async_fair_states(&reach))
        };
        self.statistic.fair_cycle_time += start.elapsed();
        for accept in self.automata.accepting_states.iter() {
            if &reach[*accept] & &fair_states[*accept] != self.manager.constant(false) {
                return false;
            }
        }
        true
    }
}

pub fn get_ltl(smv: &Smv, extend_trans: &[usize], flatten: bool) -> Expr {
    dbg!(&smv.trans.len());
    dbg!(extend_trans);
    let smv = if flatten {
        smv.flatten_defines()
    } else {
        smv.clone()
    };
    for x in extend_trans.iter() {
        dbg!(&smv.trans[*x]);
    }
    let trans_ltl = extend_trans
        .iter()
        .fold(Expr::LitExpr(true), |fold, extend| {
            fold & Expr::PrefixExpr(Prefix::LtlGlobally, Box::new(smv.trans[*extend].clone()))
        });
    let mut fairness = Expr::LitExpr(true);
    for fair in smv.fairness.iter() {
        let fair = Expr::PrefixExpr(
            Prefix::LtlGlobally,
            Box::new(Expr::PrefixExpr(Prefix::LtlFinally, Box::new(fair.clone()))),
        );
        fairness = fairness & fair;
    }
    let ltl = smv.ltlspecs[0].clone();
    let ltl = !Expr::InfixExpr(
        smv::Infix::Imply,
        Box::new(trans_ltl & fairness),
        Box::new(ltl),
    );
    let ltl = ltl_to_automata_preprocess(&smv, ltl);
    println!("{}", ltl);
    ltl
}

pub fn check(manager: BddManager, smv: Smv, args: Args) -> (bool, Duration) {
    if !args.old_impl {
        AsyncWorker::create(args.parallel);
    }
    let smv_bdd = SmvBdd::new(&manager, &smv);
    let mut fsmbdd = smv_bdd.to_fsmbdd(args.trans_method.into());
    fsmbdd.justice.clear();
    dbg!(Sylvan::num_var());
    let mut ba = BuchiAutomata::from_ltl(
        get_ltl(&smv, &args.ltl_extend_trans, args.flatten_define),
        &manager,
        &smv_bdd.symbols,
        &smv_bdd.defines,
    );
    dbg!(ba.num_state());
    for var in args.ltl_extend_vars.iter() {
        ba = ba.partition(*var);
    }
    dbg!(ba.num_state());
    let mut ppsmc = PPSMC::new(manager, fsmbdd, ba, args);
    dbg!("property-driven smc start checking");
    let start = Instant::now();
    let res = ppsmc.check();
    let time = start.elapsed();
    dbg!(ppsmc.statistic);
    (res, time)
}
