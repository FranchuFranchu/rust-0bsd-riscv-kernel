
#[inline(always)]
pub unsafe fn do_syscall_0(number: usize) -> (usize, usize) {
    let mut ret: (usize, usize) = (0, 0);
    asm!(
        "ecall",
        in("a7") number,
        lateout("a0") ret.0,
        lateout("a1") ret.1,
    );
    return ret;
}


#[inline(always)]
pub unsafe fn do_syscall_1(number: usize, a0: usize) -> (usize, usize) {
    let mut ret: (usize, usize) = (0, 0);
    asm!(
        "ecall",
        in("a7") number,
        in("a0") a0,
        lateout("a0") ret.0,
        lateout("a1") ret.1,
    );
    return ret;
}

#[inline(always)]
pub unsafe fn do_syscall_2(number: usize, a0: usize, a1: usize) -> (usize, usize) {
    let mut ret: (usize, usize) = (0, 0);
    asm!(
        "ecall",
        in("a7") number,
        in("a0") a0,
        in("a1") a1,
        lateout("a0") ret.0,
        lateout("a1") ret.1,
    );
    return ret;
}

#[inline(always)]
pub unsafe fn do_syscall_3(number: usize, a0: usize, a1: usize, a2: usize,) -> (usize, usize) {
    let mut ret: (usize, usize) = (0, 0);
    asm!(
        "ecall",
        in("a7") number,
        in("a0") a0,
        in("a1") a1,
        in("a2") a2,
        lateout("a0") ret.0,
        lateout("a1") ret.1,
    );
    return ret;
}

#[inline(always)]
pub unsafe fn do_syscall_4(number: usize, a0: usize, a1: usize, a2: usize, a3: usize,) -> (usize, usize) {
    let mut ret: (usize, usize) = (0, 0);
    asm!(
        "ecall",
        in("a7") number,
        in("a0") a0,
        in("a1") a1,
        in("a2") a2,
        in("a3") a3,
        lateout("a0") ret.0,
        lateout("a1") ret.1,
    );
    return ret;
}

#[inline(always)]
pub unsafe fn do_syscall_7(number: usize, a0: usize, a1: usize, a2: usize, a3: usize, a4: usize, a5: usize, a6: usize) -> (usize, usize) {
    let mut ret: (usize, usize) = (0, 0);
    asm!(
        "ecall",
        in("a7") number,
        in("a0") a0,
        in("a1") a1,
        in("a2") a2,
        in("a3") a3,
        in("a4") a4,
        in("a5") a5,
        in("a6") a6,
        lateout("a0") ret.0,
        lateout("a1") ret.1,
    );
    return ret;
}

#[inline(always)]
pub unsafe fn do_syscall_slice(number: usize, arguments: &[usize; 7]) -> (usize, usize) {
    let mut ret: (usize, usize) = (0, 0);
    asm!(
        "ecall",
        in("a7") number,
        in("a0") arguments[0],
        in("a1") arguments[1],
        in("a2") arguments[2],
        in("a3") arguments[3],
        in("a4") arguments[4],
        in("a5") arguments[5],
        in("a6") arguments[6],
        lateout("a0") ret.0,
        lateout("a1") ret.1,
    );
    return ret;
}