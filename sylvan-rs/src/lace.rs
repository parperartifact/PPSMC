use std::{ffi::c_void, iter::repeat_with, mem::forget};
use sylvan_sys::lace::{
    Lace_get_head, Lace_get_worker, Lace_run_func, Lace_run_func_without_block, Lace_spawn_func,
    Lace_steal_random, Lace_sync_func, Lace_task_offset, Lace_yield_newframe, Task, WorkerP,
};

#[derive(Clone, Copy)]
pub struct LaceWorkerContext {
    lace_worker: *mut WorkerP,
    lace_dq_head: *mut Task,
}

unsafe impl Send for LaceWorkerContext {}

unsafe impl Sync for LaceWorkerContext {}

impl LaceWorkerContext {
    pub fn new(lace_worker: *mut WorkerP, lace_dq_head: *mut Task) -> Self {
        Self {
            lace_worker,
            lace_dq_head,
        }
    }

    fn add_dq_head(&mut self) {
        self.lace_dq_head = unsafe { Lace_task_offset(self.lace_dq_head, 1) };
    }

    fn sub_dq_head(&mut self) {
        self.lace_dq_head = unsafe { Lace_task_offset(self.lace_dq_head, -1) };
    }
}

extern "C" fn lace_callback_hook<F, R>(
    worker: *mut WorkerP,
    task: *mut Task,
    f: *mut c_void,
) -> *mut c_void
where
    F: FnOnce(LaceWorkerContext) -> R,
    F: Send,
    R: Send,
{
    let f = unsafe { Box::from_raw(f as *mut F) };
    let context = LaceWorkerContext::new(worker, task);
    let mut res = Box::new(f(context));
    let res_ptr = res.as_mut() as *mut _;
    forget(res);
    res_ptr as _
}

pub fn lace_run<F, R>(f: F) -> R
where
    F: FnOnce(LaceWorkerContext) -> R,
    F: Send,
    R: Send + 'static,
{
    let mut f = Box::new(f);
    let res = unsafe { Lace_run_func(lace_callback_hook::<F, R>, f.as_mut() as *mut _ as _) };
    forget(f);
    *unsafe { Box::from_raw(res as *mut R) }
}

pub fn lace_run_without_block<F>(f: F)
where
    F: FnOnce(LaceWorkerContext),
    F: Send,
{
    let mut f = Box::new(f);
    unsafe { Lace_run_func_without_block(lace_callback_hook::<F, ()>, f.as_mut() as *mut _ as _) };
    forget(f);
}

impl LaceWorkerContext {
    pub fn get() -> Self {
        let lace_worker = unsafe { Lace_get_worker() };
        Self {
            lace_worker,
            lace_dq_head: unsafe { Lace_get_head(lace_worker) },
        }
    }

    pub fn lace_spawn<F, R>(&mut self, f: F)
    where
        F: FnOnce(LaceWorkerContext) -> R,
        F: Send + 'static,
        R: Send + 'static,
    {
        let mut f = Box::new(f);
        unsafe {
            Lace_spawn_func(
                self.lace_worker,
                self.lace_dq_head,
                lace_callback_hook::<F, R>,
                f.as_mut() as *mut _ as _,
            );
        };
        forget(f);
        self.add_dq_head();
    }

    pub fn lace_sync<R>(&mut self) -> R
    where
        R: Send + 'static,
    {
        let res = unsafe { Lace_sync_func(self.lace_worker, self.lace_dq_head) };
        let res = *unsafe { Box::from_raw(res as *mut R) };
        self.sub_dq_head();
        res
    }

    pub fn lace_sync_multi<R>(&mut self, num: usize) -> Vec<R>
    where
        R: Send + 'static,
    {
        let mut res: Vec<R> = repeat_with(|| self.lace_sync::<R>()).take(num).collect();
        res.reverse();
        res
    }

    pub fn yield_newframe(&self) {
        unsafe { Lace_yield_newframe(self.lace_worker, self.lace_dq_head) }
    }

    pub fn steal_random(&self) {
        unsafe { Lace_steal_random(self.lace_worker, self.lace_dq_head) }
    }
}
