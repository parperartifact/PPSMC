use crate::{automata::BuchiAutomata, Bdd, BddManager};
use fsmbdd::FsmBdd;
use std::sync::{
    atomic::{AtomicI32, Ordering},
    Arc,
};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

#[derive(Debug)]
enum Message {
    Data(Bdd),
    Quit,
}

pub struct Worker {
    id: usize,
    manager: BddManager,
    pub fsmbdd: FsmBdd<BddManager>,
    sender: Vec<UnboundedSender<Message>>,
    receiver: UnboundedReceiver<Message>,
    active: Arc<AtomicI32>,
    forward: Vec<(usize, Bdd)>,
    backward: Vec<(usize, Bdd)>,
}

unsafe impl Sync for Worker {}

impl Worker {
    pub fn propagate_value(
        &self,
        mut reach: Bdd,
        data: Arc<Vec<Bdd>>,
        constraint: Bdd,
    ) -> (Bdd, Bdd) {
        let mut new_frontier = self.manager.constant(false);
        for (from, label) in self.forward.iter() {
            let mut update = &data[*from] & label & &constraint;
            update &= !&reach;
            new_frontier |= &update;
            reach |= update;
        }
        (reach, new_frontier)
    }

    async fn propagate(&mut self, forward: bool, data: Bdd) {
        if data.is_constant(false) {
            return;
        }
        let ba_trans = if forward {
            &self.forward
        } else {
            &self.backward
        };
        for (next, label) in ba_trans {
            let message = &data & label;
            if !message.is_constant(false) {
                self.active.fetch_add(1, Ordering::Relaxed);
                self.sender[*next].send(Message::Data(message)).unwrap();
            }
        }
    }

    fn quit(&mut self) {
        for i in 0..self.sender.len() {
            if i != self.id {
                self.sender[i].send(Message::Quit).unwrap();
            }
        }
    }

    pub async fn reset(&mut self) {
        while let Ok(message) = self.receiver.try_recv() {
            match message {
                Message::Quit => (),
                _ => todo!(),
            }
        }
        self.active.fetch_max(self.id as i32 + 1, Ordering::Relaxed);
    }

    pub async fn post_reachable(&mut self, init: Bdd) -> Bdd {
        let mut reach = init.clone();
        self.propagate(true, init).await;
        loop {
            if self.active.fetch_sub(1, Ordering::Relaxed) == 1 {
                self.quit();
                return reach;
            }
            let mut update = self.manager.constant(false);
            match self.receiver.recv().await.unwrap() {
                Message::Data(data) => {
                    update |= &data;
                }
                Message::Quit => return reach,
            }
            let mut num_update: i32 = 0;
            while let Ok(message) = self.receiver.try_recv() {
                match message {
                    Message::Data(data) => {
                        update |= &data;
                        num_update -= 1;
                    }
                    _ => panic!(),
                }
            }
            if !update.is_constant(false) {
                let mut update = self.fsmbdd.post_image(&update);
                update &= !&reach;
                reach |= &update;
                self.propagate(true, update).await;
            }
            self.active.fetch_add(num_update, Ordering::Relaxed);
        }
    }

    pub async fn pre_reachable(&mut self, init: Bdd, constraint: Bdd) -> Bdd {
        let mut reach = self.manager.constant(false);
        if init != self.manager.constant(false) {
            self.propagate(false, self.fsmbdd.pre_image(&init)).await;
        }
        loop {
            if self.active.fetch_sub(1, Ordering::Relaxed) == 1 {
                self.quit();
                return reach & init;
            }
            let mut update = self.manager.constant(false);
            match self.receiver.recv().await.unwrap() {
                Message::Data(data) => {
                    update |= &data;
                }
                Message::Quit => return reach & init,
            }
            let mut num_update: i32 = 0;
            while let Ok(message) = self.receiver.try_recv() {
                match message {
                    Message::Data(data) => {
                        update |= &data;
                        num_update -= 1;
                    }
                    _ => panic!(),
                }
            }
            update &= &constraint;
            update &= !&reach;
            reach |= &update;
            if !update.is_constant(false) {
                let update = self.fsmbdd.pre_image(&update);
                self.propagate(false, update).await;
            }
            self.active.fetch_add(num_update, Ordering::Relaxed);
        }
    }

    pub fn create_workers(fsmbdd: &FsmBdd<BddManager>, automata: &BuchiAutomata) -> Vec<Self> {
        let mut recievers = vec![];
        let mut senders = vec![];
        let mut workers = vec![];
        let active = Arc::new(AtomicI32::new(0));
        for _ in 0..automata.num_state() {
            let (sender, receiver) = unbounded_channel();
            recievers.push(receiver);
            senders.push(sender);
        }
        for (id, receiver) in recievers.into_iter().enumerate() {
            let fsmbdd = fsmbdd.clone_with_new_manager();
            let forward = automata.forward[id].clone();
            let backward = automata.backward[id].clone();
            workers.push(Self {
                id,
                manager: fsmbdd.manager.clone(),
                fsmbdd,
                sender: senders.clone(),
                receiver,
                active: active.clone(),
                forward,
                backward,
            })
        }
        workers
    }
}
