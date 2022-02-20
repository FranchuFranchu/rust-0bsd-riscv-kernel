use kernel_as_register::AsRegister;

pub type SyscallErrorData = [usize; 2];

pub type SyscallReturnValueData = (usize, SyscallErrorData);

#[derive(Debug)]
pub struct SyscallReturnValue {
    data: (usize, SyscallErrorData),
}

pub type SyscallResultEnum = Result<usize, SyscallErrorData>;

impl From<(usize, usize, usize)> for SyscallReturnValue {
    fn from(data: (usize, usize, usize)) -> Self {
        Self {
            data: (data.0, [data.1, data.2]),
        }
    }
}

impl SyscallReturnValue {
    pub fn as_result<T: AsRegister>(&self) -> Result<usize, T> {
        self.as_generic_result().map_err(|s| s.as_result())
    }
    pub fn as_generic_result(&self) -> Result<usize, (usize, SyscallErrorData)> {
        //kernel_util::dont_recurse!(crate::println_crate!("Data {:?}", self.data));
        if self.data.1[0] != 0 {
            Err((self.data.0, self.data.1))
        } else {
            Ok(self.data.0)
        }
    }

    // The Err case applies if a0 is null
    pub fn as_generic_result_nonnull(&self) -> Result<usize, (usize, SyscallErrorData)> {
        //kernel_util::dont_recurse!(crate::println_crate!("Data {:?}", self.data));
        if self.data.0 == 0 {
            Err((self.data.0, self.data.1))
        } else {
            Ok(self.data.0)
        }
    }
    pub fn data_value(&self) -> usize {
        self.data.0
    }
}

pub trait AsResult {
    fn as_result<T: AsRegister>(&self) -> T;
}

impl AsResult for (usize, SyscallErrorData) {
    fn as_result<T: AsRegister>(&self) -> T {
        T::from_register(&(self.0, &self.1))
    }
}
