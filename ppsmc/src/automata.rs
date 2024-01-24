use crate::{Bdd, BddManager};
use fsmbdd::{FsmBdd, Trans, TransBddMethod};
use logic_form::Expr;
use nom::{
    bytes::complete::{tag, take_until},
    character::complete::{line_ending, multispace0, space0},
    multi::many1,
    sequence::{delimited, terminated},
    IResult,
};
use std::{collections::HashMap, mem::take, process::Command};

#[derive(Debug, Clone)]
pub struct BuchiAutomata {
    pub manager: BddManager,
    pub symbols: HashMap<String, usize>,
    pub forward: Vec<Vec<(usize, Bdd)>>,
    pub backward: Vec<Vec<(usize, Bdd)>>,
    pub accepting_states: Vec<usize>,
    pub init_states: Vec<usize>,
}

impl BuchiAutomata {
    fn new(manager: BddManager) -> Self {
        Self {
            symbols: HashMap::new(),
            manager,
            forward: Vec::new(),
            backward: Vec::new(),
            accepting_states: Vec::new(),
            init_states: Vec::new(),
        }
    }

    pub fn num_state(&self) -> usize {
        self.forward.len()
    }

    fn extend_to(&mut self, to: usize) {
        while self.forward.len() <= to {
            self.forward.push(Vec::new());
            self.backward.push(Vec::new());
        }
    }

    pub fn add_edge(&mut self, from: usize, to: usize, label: Bdd) {
        self.extend_to(from);
        self.extend_to(to);
        assert!(from < self.num_state());
        assert!(to < self.num_state());
        self.forward[from].push((to, label.clone()));
        self.backward[to].push((from, label));
    }

    pub fn add_init_state(&mut self, state: usize) {
        self.init_states.push(state);
    }

    pub fn add_accepting_state(&mut self, state: usize) {
        self.accepting_states.push(state);
    }
}

fn skip_line(input: &str) -> IResult<&str, &str> {
    terminated(take_until("\n"), line_ending)(input)
}

fn parse_trans(input: &str) -> IResult<&str, (&str, &str)> {
    let (input, _) = delimited(space0, tag("::"), space0)(input)?;
    let (input, cond) = terminated(take_until(" -> goto "), tag(" -> goto "))(input)?;
    let (input, dist) = terminated(take_until("\n"), line_ending)(input)?;
    Ok((input, (cond, dist)))
}

fn parse_state(input: &str) -> IResult<&str, (&str, Vec<(&str, &str)>)> {
    let (input, ident) = terminated(take_until(":"), tag(":"))(input)?;
    let (input, _) = skip_line(input)?;
    if let Ok((input, _)) = delimited(
        space0::<&str, nom::error::Error<&str>>,
        tag("skip"),
        multispace0,
    )(input)
    {
        return Ok((input, (ident, vec![("(1)", ident)])));
    }
    let (input, _) = skip_line(input)?;
    let (input, trans) = many1(parse_trans)(input)?;
    let (input, _) = skip_line(input)?;
    Ok((input, (ident, trans)))
}

impl BuchiAutomata {
    fn state_ident_get_id<'a>(
        &mut self,
        map: &mut HashMap<&'a str, usize>,
        ident: &'a str,
    ) -> usize {
        map.get(ident)
            .copied()
            .or_else(|| {
                assert!(map.insert(ident, self.num_state()).is_none());
                self.extend_to(self.num_state());
                map.get(ident).copied()
            })
            .unwrap()
    }

    fn parse(input: &str, manager: &BddManager, symbols: &HashMap<String, Bdd>) -> Self {
        let mut ret = Self::new(manager.clone());
        let mut state_map = HashMap::new();
        let (input, _) = skip_line(input).unwrap();
        let (input, states) = many1(parse_state)(input).unwrap();
        let (input, _) = skip_line(input).unwrap();
        assert!(input.is_empty());
        for (ident, trans) in states {
            let state_id = ret.state_ident_get_id(&mut state_map, ident);
            if ident.starts_with("accept_") {
                ret.add_accepting_state(state_id);
            }
            if ident.ends_with("_init") {
                ret.add_init_state(state_id);
            }
            let mut cond = manager.constant(true);
            for i in 0..trans.len() {
                let edge = trans[i].0;
                let dist = trans[i].1;
                let dist = ret.state_ident_get_id(&mut state_map, dist);
                let edge = if edge == "(1)" {
                    Expr::LitExpr(true)
                } else {
                    Expr::from(edge)
                };
                let edge_bdd = edge.to_bdd(manager, symbols);
                cond &= !&edge_bdd;
                ret.add_edge(state_id, dist, edge_bdd);
            }
        }
        ret
    }

    pub fn from_ltl(
        ltl: smv::Expr,
        manager: &BddManager,
        symbols: &HashMap<String, usize>,
        defines: &HashMap<String, Bdd>,
    ) -> Self {
        println!("'{}'", ltl);
        // let ltl2dfa = Command::new("./ltl2ba/ltl2ba")
        //     .arg("-f")
        //     .arg(format!("{}", ltl))
        //     .output()
        //     .unwrap();
        let ltl2dfa = Command::new("/root/spot-2.11.5/bin/ltl2tgba")
            .arg("-s")
            .arg("-f")
            .arg(format!("{}", ltl))
            .output()
            .unwrap();
        let ba = String::from_utf8_lossy(&ltl2dfa.stdout);
        let mut defines = defines.clone();
        for (ident, id) in symbols {
            defines.insert(ident.clone(), manager.ith_var(*id));
        }
        let mut ba = BuchiAutomata::parse(ba.as_ref(), manager, &defines);
        dbg!(ba.num_state());
        ba.symbols = symbols.clone();
        ba
    }

    fn automata_state_encode(&self, base: usize, num_encode_var: usize, mut id: usize) -> Bdd {
        let mut res = self.manager.constant(true);
        for i in 0..num_encode_var {
            let var = self.manager.ith_var((base + i) * 2);
            if id % 2 == 0 {
                res &= !var;
            } else {
                res &= var;
            }
            id /= 2;
        }
        res
    }

    pub fn to_fsmbdd(&self) -> FsmBdd<BddManager> {
        let mut symbols = self.symbols.clone();
        let base = symbols.len();
        let num_encode_var = usize::BITS as usize - (self.num_state() - 1).leading_zeros() as usize;
        for encode_var in 0..num_encode_var {
            self.manager.ith_var((base + encode_var) * 2);
            self.manager.ith_var((base + encode_var) * 2 + 1);
            symbols.insert(format!("automata{}", encode_var), (base + encode_var) * 2);
        }
        let mut init = self.manager.constant(false);
        for init_state in self.init_states.iter() {
            init |= self.automata_state_encode(base, num_encode_var, *init_state);
        }
        let mut trans = self.manager.constant(false);
        for state in 0..self.num_state() {
            for (next, label) in self.forward[state].iter() {
                let next = self
                    .automata_state_encode(base, num_encode_var, *next)
                    .next_state();
                let state = self.automata_state_encode(base, num_encode_var, state);
                trans |= next & label & state;
            }
        }
        let trans = Trans::new(&self.manager, vec![trans], TransBddMethod::Monolithic);
        let mut fair = self.manager.constant(false);
        for fair_state in self.accepting_states.iter() {
            fair |= self.automata_state_encode(base, num_encode_var, *fair_state);
        }
        FsmBdd {
            symbols,
            manager: self.manager.clone(),
            init,
            invariants: self.manager.constant(true),
            trans,
            justice: vec![fair],
        }
    }

    pub fn partition(mut self, var: usize) -> Self {
        let var = var * 2;
        let num_states = self.num_state();
        for accept in self.accepting_states.clone() {
            self.accepting_states.push(accept + num_states);
        }
        for init in self.init_states.clone() {
            self.init_states.push(init + num_states);
        }
        self.backward.clear();
        let forward = take(&mut self.forward);
        for i in 0..num_states {
            for (next, bdd) in forward[i].iter() {
                self.add_edge(i, *next, bdd & !self.manager.ith_var(var));
                self.add_edge(i + num_states, *next, bdd & !self.manager.ith_var(var));
                self.add_edge(i, next + num_states, bdd & self.manager.ith_var(var));
                self.add_edge(
                    i + num_states,
                    next + num_states,
                    bdd & self.manager.ith_var(var),
                );
            }
        }
        self
    }
}
