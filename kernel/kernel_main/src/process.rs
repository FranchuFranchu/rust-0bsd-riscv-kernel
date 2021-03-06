//! Creating, switching to, and other functions related to process management (both kernel and user)
use alloc::{
    boxed::Box,
    collections::BTreeMap,
    string::String,
    sync::{Arc, Weak},
    vec::Vec,
};
use core::{
    arch::asm,
    future::Future,
    pin::Pin,
    sync::atomic::AtomicUsize,
    task::{RawWaker, RawWakerVTable, Waker},
};

use crate::{
    asm::do_supervisor_syscall_0,
    context_switch,
    cpu::{self, load_hartid, read_sscratch, Registers},
    handle::Handle,
    hart::get_this_hart_meta,
    lock::shared::RwLock,
    scheduler::schedule_next_slice,
    trap::{in_interrupt_context, use_boot_frame_if_necessary},
    trap_frame::{TrapFrame, TrapFrameExt},
};

pub const TASK_STACK_SIZE: usize = 4096 * 8;
pub const PROCESS_WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(
    /* clone */ Process::waker_clone,
    /* wake */ Process::waker_wake,
    /* wake_by_ref */ Process::waker_wake_by_ref,
    /* drop */ Process::waker_drop,
);
pub static PROCESSES: RwLock<BTreeMap<usize, PidSlot>> = RwLock::new(BTreeMap::new());
pub static PROCESS_SCHED_QUEUE: RwLock<Vec<Weak<RwLock<Process>>>> = RwLock::new(Vec::new());

pub enum PidSlot {
    Allocated,
    Used(Arc<RwLock<Process>>),
}

impl PidSlot {
    pub fn unwrap_ref(&self) -> Option<&Arc<RwLock<Process>>> {
        match self {
            PidSlot::Allocated => None,
            PidSlot::Used(p) => Some(p),
        }
    }
    pub fn unwrap_mut(&mut self) -> Option<&mut Arc<RwLock<Process>>> {
        match self {
            PidSlot::Allocated => None,
            PidSlot::Used(p) => Some(p),
        }
    }
    pub fn is_used(&self) -> bool {
        match self {
            PidSlot::Allocated => false,
            PidSlot::Used(_) => true,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ProcessState {
    // Currently running (right now)
    Running,
    // Waiting for a future (or any other blocking action)
    Yielded,
    // Not scheduled and not waiting for any future
    Pending,
    // schedule() has been called on this process, but it hasn't started running yet
    Scheduled,
}

#[derive(Debug)]
pub struct Process {
    /// The process ID of the process can be fetched by getting trap_frame.pid
    pub is_supervisor: bool,
    pub state: ProcessState,
    pub handles: BTreeMap<usize, Handle>,
    pub trap_frame: Pin<Box<TrapFrame>>,
    pub name: Option<String>,
    no_op_yield_count: AtomicUsize,

    /// For supervisor mode the kernel initially creates a small stack page for this process
    /// This is where it's stored
    pub kernel_allocated_stack: Option<Box<[u8; TASK_STACK_SIZE]>>,

    pub user_id: u64,
}

extern "C" {
    // Never returns, the current thread of execution is destroyed
    fn switch_to_supervisor_frame(trap_frame: *mut TrapFrame) -> !;
    fn switch_to_user_frame(trap_frame: *mut TrapFrame) -> !;
}

impl Process {
    pub fn has_read_access(&self, _address: usize, _size: usize) -> bool {
        if self.is_supervisor {
            return true;
        }
        false
    }
    pub fn has_write_access(&self, _address: usize, _size: usize) -> bool {
        if self.is_supervisor {
            return true;
        }
        false
    }
    pub fn can_be_scheduled(&self) -> bool {
        match self.state {
            ProcessState::Pending => true,
            _ => false,
        }
    }
    // Uses this hart to execute this process until the next context switch happens
    // This function essentially never returns because it runs until an interrupt happens
    pub fn run_once(&mut self) -> ! {
        if self.trap_frame.pc > 0x8000000 as usize && !self.is_supervisor {
            panic!("{:?}", "user process aren't really meant to do this");
        }
        // The hart ID that we will be executing in is the same one as the current one.
        self.trap_frame.hartid = load_hartid();

        // Use the same stack for interrupts
        self.trap_frame.interrupt_stack = unsafe { (*read_sscratch()).interrupt_stack };

        // Get a raw pointer to the Box's data (which is the trap frame)
        let frame_pointer =
            Pin::as_ref(&self.trap_frame).get_ref() as *const TrapFrame as *mut TrapFrame;
        /*
            let (time, perf, cycle): (usize, usize, usize);

            unsafe { asm!("csrr {0}, time", out(reg)(time),) };
            unsafe { asm!("csrr {0}, instret", out(reg)(perf),) };
            unsafe { asm!("csrr {0}, cycle", out(reg)(cycle),) };

            //debug!("{:?} {:?} {:?}", perf, cycle, time);
        */

        debug!(
            "Switch to frame at \x1b[32m{:?}\x1b[0m (PC {:x} NAME {:?} HART {})",
            frame_pointer,
            unsafe { (*frame_pointer).pc },
            self.name,
            self.trap_frame.hartid
        );

        //info!("{:?}", PROCESSES.read().iter().filter(|(k, v)| **k != self.trap_frame.pid).map(|(k, v)| v.read().name.clone()).collect() : Vec<_>);
        /*
        info!(
            "count {:?}",
            PROCESSES
                .read()
                .iter()
                .filter(|(k, v)| **k != self.trap_frame.pid)
                .filter(|(k, v)| if let ProcessState::Running = v.read().state {
                    true
                } else {
                    false
                })
                .count()
                + 1
        );
        */

        self.state = ProcessState::Running;

        self.trap_frame.flags &= !1;

        if self.is_supervisor {
            // Switch to the trap frame
            unsafe { switch_to_supervisor_frame(frame_pointer) };
        } else {
            unsafe { switch_to_user_frame(frame_pointer) };
        }
    }

    // These are the waker methods
    // They turn a process in Yielded state to a process in Pending state
    // The data parameter is the return value of into_raw for a Box<Weak<Process>>
    unsafe fn waker_clone(data: *const ()) -> RawWaker {
        let obj = Box::from_raw(data as *mut Weak<RwLock<Self>>);
        let new_waker = RawWaker::new(Box::into_raw(obj.clone()) as _, &PROCESS_WAKER_VTABLE);
        Box::leak(obj);
        new_waker
    }
    unsafe fn waker_wake(data: *const ()) {
        Self::waker_wake_by_ref(data);
        Self::waker_drop(data)
    }
    unsafe fn waker_wake_by_ref(data: *const ()) {
        // The box re-acquires ownership of the RwLock<Self>
        let process: Box<Weak<RwLock<Self>>> = Box::from_raw(data as _);
        let process_internal = process.upgrade().expect("Waited process is gone!");
        process_internal.write().make_pending_when_possible();
        // Make the box lose ownership of the RwLock<Self>
        Box::leak(process);
    }
    unsafe fn waker_drop(data: *const ()) {
        // Re-create the box for this waker and then drop it to prevent memory leaks
        drop(Box::from_raw(data as *mut Weak<RwLock<Self>>));
    }

    /// This process makes this process pending if it's yielded
    ///
    /// If it isn't, then it will "queue up" the wake-up signal to the process so that it can be "consumed" the next time the process should yield
    pub fn make_pending_when_possible(&mut self) {
        match self.state {
            ProcessState::Yielded => {
                self.state = ProcessState::Pending;
            }
            _ => {
                self.no_op_yield_count
                    .fetch_add(1, core::sync::atomic::Ordering::SeqCst);
            }
        }
    }

    /// This creates a Waker that makes this process a Pending process when woken
    ///
    /// The Pending process will be eventually scheduled
    pub fn construct_waker(&self) -> Waker {
        // Create a weak pointer to a RwLock<Self> and then erase its type
        let raw_pointer =
            Box::into_raw(Box::new(weak_get_process(&self.trap_frame.pid))) as *const ();
        // Create a waker with the pointer as the data
        unsafe { Waker::from_raw(RawWaker::new(raw_pointer, &PROCESS_WAKER_VTABLE)) }
    }

    /// Polls a future from this process. The waker is this processes' waker
    pub fn poll_future<T: Future>(
        &mut self,
        future: Pin<&mut T>,
    ) -> core::task::Poll<<T as Future>::Output> {
        let poll_result = future.poll(&mut core::task::Context::from_waker(
            &self.construct_waker(),
        ));

        if poll_result.is_pending() {
            // Mark the task as yielded
            // We'll be woken up eventually and this will be called again
            self.state = ProcessState::Yielded;
            schedule_next_slice(0);
        }

        poll_result
    }

    pub fn yield_maybe(&mut self) -> bool {
        if self
            .no_op_yield_count
            .load(core::sync::atomic::Ordering::Acquire)
            == 0
        {
            self.state = ProcessState::Yielded;
            true
        } else {
            self.no_op_yield_count
                .fetch_sub(1, core::sync::atomic::Ordering::AcqRel);
            false
        }
    }

    // Like yield_maybe, but does nothing and predicts the return value of yield_maybe
    pub fn try_yield_maybe(&mut self) -> bool {
        self.no_op_yield_count
            .load(core::sync::atomic::Ordering::Acquire)
            == 0
    }

    pub fn this_pid() -> usize {
        unsafe {
            cpu::read_sscratch()
                .as_ref()
                .expect("Not running on a process!")
                .pid
        }
    }

    pub fn this() -> Arc<RwLock<Process>> {
        try_get_process(&Self::this_pid())
    }
}
pub fn init() {}

// All functions after this are only safe when init() has been called
// (but init doesn't do anything yet, so nothing bad happens)

// Marks the process executed as the current hart as pending
pub fn finish_executing_process(pid: usize) {
    if pid == 0 || pid == 1 {
        // Boot process can't be stopped
        return;
    }
    try_get_process(&pid).write().state = ProcessState::Pending;
    debug!("Made process pending");
}

/// Finds an unused PID
pub fn allocate_pid() -> usize {
    allocate_pid_lockfree(&mut PROCESSES.write())
}

pub fn allocate_pid_lockfree(processes: &mut BTreeMap<usize, PidSlot>) -> usize {
    let mut pid = 2;
    for this_pid in pid.. {
        if !processes.contains_key(&this_pid) {
            processes.insert(this_pid, PidSlot::Allocated);
            pid = this_pid;
            break;
        }
    }
    pid
}

/// Creates a supervisor process and returns PID
/// SAFETY: Only when function is a valid function pointer
pub fn new_process(constructor: impl FnOnce(&mut Process)) -> usize {
    // Hold this guard for as much time as possible
    // to prevent a race condition on allocate_pid
    let mut guard = PROCESSES.write();
    let pid = allocate_pid_lockfree(&mut *guard);

    let trapframe_box = Box::new(TrapFrame::zeroed());
    let trapframe_box = Pin::new(trapframe_box);

    let mut process = Process {
        is_supervisor: false,
        handles: BTreeMap::new(),
        trap_frame: trapframe_box,
        state: ProcessState::Pending,
        kernel_allocated_stack: None,
        name: None,
        no_op_yield_count: AtomicUsize::new(0),
        user_id: 0,
    };

    constructor(&mut process);

    process.trap_frame.pid = pid;
    process.trap_frame.use_current_satp_as_kernel_satp();
    if process.is_supervisor {
        process.trap_frame.satp = process.trap_frame.kernel_satp;
    }
    process.trap_frame.hartid = 0xBADC0DE;

    // Wrap the process in a lock
    let process = RwLock::new(process);
    // Move the process into an Arc
    let process = Arc::new(process);

    // Schedule the process
    PROCESS_SCHED_QUEUE.write().push(Arc::downgrade(&process));

    guard.insert(pid, PidSlot::Used(process));

    pid
}

pub fn new_supervisor_process_int(function: usize, a0: usize) -> usize {
    new_process(|process| {
        process.is_supervisor = true;
        process.trap_frame.general_registers[Registers::Ra.idx()] =
            process_return_address_supervisor as usize;

        let process_stack = kernel_util::boxed_slice_with_alignment(4096, 4096, &0u8);
        process.trap_frame.interrupt_stack =
            process_stack.as_ptr() as usize + TASK_STACK_SIZE - 0x10;

        Box::leak(process_stack);

        let process_stack = kernel_util::boxed_slice_with_alignment(TASK_STACK_SIZE, 4096, &0u8);
        process.trap_frame.general_registers[Registers::Sp.idx()] =
            process_stack.as_ptr() as usize + TASK_STACK_SIZE - 0x10;

        process.trap_frame.general_registers[Registers::A0.idx()] = a0;
        process.trap_frame.pc = function;

        use core::convert::TryInto;
        process.kernel_allocated_stack = Some(
            process_stack
                .try_into()
                .expect("Process stack has incorrect length!"),
        );
    })
}

#[no_mangle]
pub extern "C" fn process_return_address_supervisor() {
    unsafe { crate::asm::do_supervisor_syscall_0(1) };
    debug!("{:?}", "Process return address");
    // Run a syscall that deletes the process
    unsafe {
        asm!(r"
			li a7, 1
			# Trigger a software interrupt
			csrr t0, sip
			# Set SSIP
			ori t0, t0, 2
			csrw sip, t0
		", out("a7") _, out("t0") _)
    }
}

pub fn new_supervisor_process(function: fn()) -> usize {
    new_supervisor_process_int(function as usize, 0 /* a0 doesn't matter */)
}

pub fn new_supervisor_process_argument(function: fn(usize), a0: usize) -> usize {
    new_supervisor_process_int(function as usize, a0)
}

pub fn new_supervisor_process_with_name(function: fn(), name: String) -> usize {
    let pid = new_supervisor_process(function);
    try_get_process(&pid).write().name = Some(name);
    pid
}

pub fn delete_process(pid: usize) {
    // If our trap frame is the same one as the process's trap frame,
    // change sscratch to use the boot trap frame
    // (since the current sscratch is held by the Process struct and will deallocated soon)
    use_boot_frame_if_necessary(&*try_get_process(&pid).read().trap_frame as _);
    // We don't need to remove from the sched queue here.
    // That gets done on context switching
    PROCESSES.write().remove(&pid);
}

// This returns an empty Weak if the process doesn't exist
pub fn weak_get_process(pid: &usize) -> Weak<RwLock<Process>> {
    PROCESSES
        .read()
        .get(pid)
        .map(|slot| {
            use PidSlot::*;
            match slot {
                Allocated => Weak::new(),
                Used(p) => Arc::downgrade(p),
            }
        })
        .unwrap_or_default()
}

// This assumes that the process exists and panics if it doesn't
// Also acts as a strong reference to the process
pub fn try_get_process(pid: &usize) -> Arc<RwLock<Process>> {
    if *pid == 1 {
        panic!("try_get_process: tried getting process 1! (the boot process)");
    }
    PROCESSES
        .read()
        .get(pid)
        .unwrap_or_else(|| panic!("Process with pid {} does not exist", pid))
        .unwrap_ref()
        .unwrap()
        .clone()
}

/// Gets the amount of processes that aren't idle processes and are still alive
/// Right now the way that it checks for idle processes is that it checks if their name starts with "Idle"
/// TODO use a better method
pub fn useful_process_count() -> usize {
    PROCESSES
        .read()
        .iter()
        .filter_map(|(k, v)| Some((k, PidSlot::unwrap_ref(v)?)))
        .filter(|(_k, v)| {
            v.read()
                .name
                .as_ref()
                .map(|s| !s.starts_with("Idle "))
                .unwrap_or(true)
        })
        .count()
}

pub fn idle_entry_point() {
    cpu::wfi();
    get_this_hart_meta().unwrap().set_idle_process(None);
    unsafe { do_supervisor_syscall_0(1) };
}

pub fn idle_forever_entry_point() {
    loop {
        cpu::wfi();
    }
}

/// Starts a process that wfi()s once, immediately switches to the process, then exits.
/// Must be called from an interrupt context.
pub fn idle() -> ! {
    let pid = {
        assert!(in_interrupt_context());
        use alloc::format;
        let this_process = weak_get_process(&Process::this_pid()).upgrade();

        if let Some(process) = this_process {
            let mut process = process.write();
            crate::trap::use_boot_frame_if_necessary(&*process.trap_frame as _);
            if process.state == ProcessState::Running {
                process.state = ProcessState::Pending;
            }
        }
        if let Some(pid) = get_this_hart_meta().unwrap().get_idle_process() {
            pid
        } else {
            let pid = new_supervisor_process_with_name(
                idle_entry_point,
                format!("Idle process for hart {}", load_hartid()),
            );
            get_this_hart_meta().unwrap().set_idle_process(Some(pid));
            pid
        }
    };
    assert!(try_get_process(&pid).read().is_supervisor == true);
    schedule_next_slice(1);
    context_switch::context_switch(&pid)
}
