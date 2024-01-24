use crate::{
    automata::BuchiAutomata, command::Args, ltl::ltl_to_automata_preprocess,
    property_driven::get_ltl, BddManager,
};
use smv::{bdd::SmvBdd, Smv};
use std::time::{Duration, Instant};
use sylvan::lace_run;

pub fn check(manager: BddManager, smv: Smv, args: Args) -> (bool, Duration) {
    let smvbdd = SmvBdd::new(&manager, &smv);
    let mut fsmbdd = smvbdd.to_fsmbdd(args.trans_method.into());
    let ltl = if args.generalize_automata {
        ltl_to_automata_preprocess(&smv, !smv.ltlspecs[0].clone())
    } else {
        fsmbdd.justice.clear();
        get_ltl(&smv, &[], args.flatten_define)
    };
    let ltl_fsmbdd =
        BuchiAutomata::from_ltl(ltl, &manager, &smvbdd.symbols, &smvbdd.defines).to_fsmbdd();
    let product = fsmbdd.product(&ltl_fsmbdd);
    dbg!(product.justice.len());
    println!("traditional smc begin");
    let start = Instant::now();
    let forward = product.reachable_from_init();
    let fair_cycle = lace_run(|_| product.fair_cycle_with_constrain(&forward));
    ((fair_cycle & forward).is_constant(false), start.elapsed())
}
