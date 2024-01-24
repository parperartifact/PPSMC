use arun::async_worker_run;
use std::{
    ops::SubAssign,
    sync::{Arc, Condvar, Mutex},
};
use sylvan::{lace_run_without_block, LaceWorkerContext};

#[derive(Clone)]
pub struct AsyncWorker {
    condvar: Arc<Condvar>,
    sync: Arc<Mutex<usize>>,
}

impl AsyncWorker {
    pub fn create(num: usize) {
        let condvar = Arc::new(Condvar::new());
        let sync = Arc::new(Mutex::new(num));
        let workers = vec![AsyncWorker { condvar, sync }; num];
        for worker in workers {
            lace_run_without_block(|c| worker.start(c))
        }
    }

    fn sync(&mut self) {
        let mut sync = self.sync.lock().unwrap();
        sync.sub_assign(1);
        if *sync == 0 {
            self.condvar.notify_all();
        } else {
            drop(self.condvar.wait(sync).unwrap());
        }
    }

    pub fn start(mut self, context: LaceWorkerContext) {
        self.sync();
        loop {
            context.steal_random();
            async_worker_run();
        }
    }
}
