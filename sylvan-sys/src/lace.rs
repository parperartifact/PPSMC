use libc::{c_int, c_uint, c_void, size_t};
use std::marker::{PhantomData, PhantomPinned};

/// An opaque C struct used to represent a Lace worker.
#[repr(C)]
pub struct WorkerP {
    _data: [u8; 0],
    _marker: PhantomData<(*mut u8, PhantomPinned)>,
}

/// An opaque C struct used to represent a Lace task.
#[repr(C)]
pub struct Task {
    _data: [u8; 0],
    _marker: PhantomData<(*mut u8, PhantomPinned)>,
}

extern "C" {
    pub fn Lace_set_verbosity(level: c_int) -> c_void;
    pub fn Lace_set_stacksize(stacksize: size_t) -> c_void;
    pub fn Lace_get_stacksize() -> size_t;
    pub fn Lace_get_pu_count() -> c_uint;
    pub fn Lace_start(n_workers: c_uint, dqsize: size_t) -> c_void;
    pub fn Lace_suspend() -> c_void;
    pub fn Lace_resume() -> c_void;
    pub fn Lace_stop() -> c_void;
    pub fn Lace_barrier() -> c_void;
    pub fn Lace_workers() -> c_uint;
    pub fn Lace_get_worker() -> *mut WorkerP;
    pub fn Lace_get_head(worker: *mut WorkerP) -> *mut Task;
    pub fn Lace_run_task(task: *mut Task) -> c_void;
    pub fn Lace_run_newframe(task: *mut Task) -> c_void;
    pub fn Lace_run_together(task: *mut Task) -> c_void;
    pub fn Lace_task_offset(task: *mut Task, offset: c_int) -> *mut Task;
    pub fn Lace_run_func(
        f: extern "C" fn(*mut WorkerP, *mut Task, *mut c_void) -> *mut c_void,
        arg: *mut c_void,
    ) -> *mut c_void;
    pub fn Lace_run_func_without_block(
        f: extern "C" fn(*mut WorkerP, *mut Task, *mut c_void) -> *mut c_void,
        arg: *mut c_void,
    ) -> *mut c_void;
    pub fn Lace_spawn_func(
        lace_worker: *mut WorkerP,
        lace_dq_head: *mut Task,
        f: extern "C" fn(*mut WorkerP, *mut Task, *mut c_void) -> *mut c_void,
        arg: *mut c_void,
    ) -> *mut Task;
    pub fn Lace_sync_func(lace_worker: *mut WorkerP, lace_dq_head: *mut Task) -> *mut c_void;
    pub fn Lace_yield_newframe(lace_worker: *mut WorkerP, lace_dq_head: *mut Task);
    pub fn Lace_steal_random(lace_worker: *mut WorkerP, lace_dq_head: *mut Task);
}
