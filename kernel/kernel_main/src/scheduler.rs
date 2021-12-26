// 0BSD

use crate::{
    cpu,
    process::{ProcessState, PROCESS_SCHED_QUEUE},
    timer_queue,
};

// Return the next PID to be run
pub fn schedule() -> usize {
    let mut process_sched_queue = PROCESS_SCHED_QUEUE.write();
    let mut pid = 0;

    // Using a vector would be too expensive to store
    // removed indices, so we'll just store a range between min idx and max idx

    // usize::MAX means no value
    let mut removed_index_min = usize::MAX;
    let mut removed_index_max = usize::MAX;

    for (idx, this_process) in process_sched_queue.iter().enumerate() {
        match this_process.upgrade() {
            // The process still exists
            Some(strong) => {
                let mut lock = strong.write();
                if lock.can_be_scheduled() {
                    pid = lock.trap_frame.pid;
                    // TODO maybe add a way to "reserve" this process to make it so that it doesn't execute?
                    lock.state = ProcessState::Scheduled;
                    break;
                }
            }
            // The process doesn't exist anymore. Remove it from the sched queue
            None => {
                if removed_index_min == usize::MAX {
                    removed_index_max = idx;
                    removed_index_min = idx;
                } else if removed_index_max == idx - 1 {
                    removed_index_max = idx;
                }
                // Else, it means that the processes that we want to remove are non-contiguous
                // It's probably more worth it to just wait until the next iteration
            }
        }
    }

    if removed_index_min != usize::MAX {
        // Each iteration, the index of the removed process will shift 1 lower
        // so we don't need to value of the iteration (the one that's replaced by an underscore here)
        for _ in removed_index_min..removed_index_max + 1 {
            process_sched_queue.remove(removed_index_min);
        }
    }

    if pid == 0 {
        // Don't schedule anything
        return 0;
    }

    process_sched_queue.rotate_left(1);

    pid
}

pub fn schedule_next_slice(slices: u64) {
    use timer_queue::{schedule_at, TimerEvent, TimerEventCause::*};
    schedule_at(TimerEvent {
        instant: cpu::get_time() + slices * 1_000_000,
        cause: ContextSwitch,
    });
}
