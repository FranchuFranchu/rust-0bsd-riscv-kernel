#![no_std]

#[macro_export]
macro_rules! dont_recurse {
    ($e:block) => {
        static C: ::core::sync::atomic::AtomicUsize = ::core::sync::atomic::AtomicUsize::new(0);
        C.fetch_add(1, ::core::sync::atomic::Ordering::Release);
        if C.load(::core::sync::atomic::Ordering::Acquire) == 1 {
            $e
        }
        C.fetch_sub(1, ::core::sync::atomic::Ordering::Release);
    };
    ($e:stmt) => {
        ::kernel_util::dont_recurse!({ $e })
    };
    ($e:expr) => {
        ::kernel_util::dont_recurse!({ $e })
    };
}
