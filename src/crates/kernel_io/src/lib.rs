#![cfg_attr(not(std), no_std)]
#![feature(associated_type_defaults, generic_associated_types)]

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::string::String;

use async_trait::async_trait;
use kernel_as_register::AsRegister;


#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, AsRegister)]
#[non_exhaustive]
pub enum ErrorKind {
    /// An entity was not found, often a file.
    NotFound,
    /// The operation lacked the necessary privileges to complete.
    PermissionDenied,
    /// The connection was refused by the remote server.
    ConnectionRefused,
    /// The connection was reset by the remote server.
    ConnectionReset,
    /// The remote host is not reachable.
    HostUnreachable,
    /// The network containing the remote host is not reachable.
    NetworkUnreachable,
    /// The connection was aborted (terminated) by the remote server.
    ConnectionAborted,
    /// The network operation failed because it was not connected yet.
    NotConnected,
    /// A socket address could not be bound because the address is already in
    /// use elsewhere.
    AddrInUse,
    /// A nonexistent interface was requested or the requested address was not
    /// local.
    AddrNotAvailable,
    /// The system's networking is down.
    NetworkDown,
    /// The operation failed because a pipe was closed.
    BrokenPipe,
    /// An entity already exists, often a file.
    AlreadyExists,
    /// The operation needs to block to complete, but the blocking operation was
    /// requested to not occur.
    WouldBlock,
    /// A filesystem object is, unexpectedly, not a directory.
    ///
    /// For example, a filesystem path was specified where one of the intermediate directory
    /// components was, in fact, a plain file.
    NotADirectory,
    /// The filesystem object is, unexpectedly, a directory.
    ///
    /// A directory was specified when a non-directory was expected.
    IsADirectory,
    /// A non-empty directory was specified where an empty directory was expected.
    DirectoryNotEmpty,
    /// The filesystem or storage medium is read-only, but a write operation was attempted.
    ReadOnlyFilesystem,
    /// Loop in the filesystem or IO subsystem; often, too many levels of symbolic links.
    ///
    /// There was a loop (or excessively long chain) resolving a filesystem object
    /// or file IO object.
    ///
    /// On Unix this is usually the result of a symbolic link loop; or, of exceeding the
    /// system-specific limit on the depth of symlink traversal.
    FilesystemLoop,
    /// Stale network file handle.
    ///
    /// With some network filesystems, notably NFS, an open file (or directory) can be invalidated
    /// by problems with the network or server.
    StaleNetworkFileHandle,
    /// A parameter was incorrect.
    InvalidInput,
    /// Data not valid for the operation were encountered.
    ///
    /// Unlike [`InvalidInput`], this typically means that the operation
    /// parameters were valid, however the error was caused by malformed
    /// input data.
    ///
    /// For example, a function that reads a file into a string will error with
    /// `InvalidData` if the file's contents are not valid UTF-8.
    ///
    /// [`InvalidInput`]: ErrorKind::InvalidInput
    InvalidData,
    /// The I/O operation's timeout expired, causing it to be canceled.
    TimedOut,
    /// An error returned when an operation could not be completed because a
    /// call to [`write`] returned [`Ok(0)`].
    ///
    /// This typically means that an operation could only succeed if it wrote a
    /// particular number of bytes but only a smaller number of bytes could be
    /// written.
    ///
    /// [`write`]: crate::io::Write::write
    /// [`Ok(0)`]: Ok
    WriteZero,
    /// The underlying storage (typically, a filesystem) is full.
    ///
    /// This does not include out of quota errors.
    StorageFull,
    /// Seek on unseekable file.
    ///
    /// Seeking was attempted on an open file handle which is not suitable for seeking - for
    /// example, on Unix, a named pipe opened with `File::open`.
    NotSeekable,
    /// Filesystem quota was exceeded.
    FilesystemQuotaExceeded,
    /// File larger than allowed or supported.
    ///
    /// This might arise from a hard limit of the underlying filesystem or file access API, or from
    /// an administratively imposed resource limitation.  Simple disk full, and out of quota, have
    /// their own errors.
    FileTooLarge,
    /// Resource is busy.
    ResourceBusy,
    /// Executable file is busy.
    ///
    /// An attempt was made to write to a file which is also in use as a running program.  (Not all
    /// operating systems detect this situation.)
    ExecutableFileBusy,
    /// Deadlock (avoided).
    ///
    /// A file locking operation would result in deadlock.  This situation is typically detected, if
    /// at all, on a best-effort basis.
    Deadlock,
    /// Cross-device or cross-filesystem (hard) link or rename.
    CrossesDevices,
    /// Too many (hard) links to the same filesystem object.
    ///
    /// The filesystem does not support making so many hardlinks to the same file.
    TooManyLinks,
    /// Filename too long.
    ///
    /// The limit might be from the underlying filesystem or API, or an administratively imposed
    /// resource limit.
    FilenameTooLong,
    /// Program argument list too long.
    ///
    /// When trying to run an external program, a system or process limit on the size of the
    /// arguments would have been exceeded.
    ArgumentListTooLong,
    /// This operation was interrupted.
    ///
    /// Interrupted operations can typically be retried.
    Interrupted,

    /// A custom error that does not fall under any other I/O error kind.
    ///
    /// This can be used to construct your own [`Error`]s that do not match any
    /// [`ErrorKind`].
    ///
    /// This [`ErrorKind`] is not used by the standard library.
    ///
    /// Errors from the standard library that do not fall under any of the I/O
    /// error kinds cannot be `match`ed on, and will only match a wildcard (`_`) pattern.
    /// New [`ErrorKind`]s might be added in the future for some of those.
    Other,

    /// An error returned when an operation could not be completed because an
    /// "end of file" was reached prematurely.
    ///
    /// This typically means that an operation could only succeed if it read a
    /// particular number of bytes but only a smaller number of bytes could be
    /// read.
    UnexpectedEof,

    /// This operation is unsupported on this platform.
    ///
    /// This means that the operation can never succeed.
    Unsupported,

    /// An operation could not be completed, because it failed
    /// to allocate enough memory.
    OutOfMemory,

    /// Any I/O error from the standard library that's not part of this list.
    ///
    /// Errors that are `Uncategorized` now may move to a different or a new
    /// [`ErrorKind`] variant in the future. It is not recommended to match
    /// an error against `Uncategorized`; use a wildcard match (`_`) instead.
    Uncategorized,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, AsRegister)]
pub enum Error {
    Os(i32),
    Simple(ErrorKind),
    // &str is a fat pointer, but &&str is a thin pointer.
    SimpleMessage(ErrorKind/*, &'static &'static str*/),
}

use core::str::Utf8Error;

impl From<Utf8Error> for Error {
    fn from(e: Utf8Error) -> Error {
        Error::new_simple(ErrorKind::InvalidData)
    }
}


impl ErrorKind {
    pub(crate) fn as_str(&self) -> &'static str {
        use ErrorKind::*;
        match *self {
            AddrInUse => "address in use",
            AddrNotAvailable => "address not available",
            AlreadyExists => "entity already exists",
            ArgumentListTooLong => "argument list too long",
            BrokenPipe => "broken pipe",
            ResourceBusy => "resource busy",
            ConnectionAborted => "connection aborted",
            ConnectionRefused => "connection refused",
            ConnectionReset => "connection reset",
            CrossesDevices => "cross-device link or rename",
            Deadlock => "deadlock",
            DirectoryNotEmpty => "directory not empty",
            ExecutableFileBusy => "executable file busy",
            FilenameTooLong => "filename too long",
            FilesystemQuotaExceeded => "filesystem quota exceeded",
            FileTooLarge => "file too large",
            HostUnreachable => "host unreachable",
            Interrupted => "operation interrupted",
            InvalidData => "invalid data",
            InvalidInput => "invalid input parameter",
            IsADirectory => "is a directory",
            NetworkDown => "network down",
            NetworkUnreachable => "network unreachable",
            NotADirectory => "not a directory",
            StorageFull => "no storage space",
            NotConnected => "not connected",
            NotFound => "entity not found",
            Other => "other error",
            OutOfMemory => "out of memory",
            PermissionDenied => "permission denied",
            ReadOnlyFilesystem => "read-only filesystem or storage medium",
            StaleNetworkFileHandle => "stale network file handle",
            FilesystemLoop => "filesystem loop or indirection limit (e.g. symlink loop)",
            NotSeekable => "seek on unseekable file",
            TimedOut => "timed out",
            TooManyLinks => "too many links",
            Uncategorized => "uncategorized error",
            UnexpectedEof => "unexpected end of file",
            Unsupported => "unsupported",
            WouldBlock => "operation would block",
            WriteZero => "write zero",
        }
    }
}

pub type Result<T> = core::result::Result<T, Error>;

impl Error {
    pub(crate) const fn new_const(kind: ErrorKind, message: &'static &'static str) -> Error {
        Self::SimpleMessage(kind)
    }
    pub(crate) const fn new_simple(kind: ErrorKind) -> Error {
        Self::Simple(kind)
    }
    pub fn kind(&self) -> &ErrorKind {
        match self {
            Self::Simple(kind) => kind,
            Self::SimpleMessage(kind) => kind,
            _ => { panic!("not an error with a kind") }
        }
    }
}

#[async_trait]
pub trait Read {
    type Error: Send + Sync;
    async fn read(&mut self, buf: &mut [u8]) -> core::result::Result<usize, Self::Error>;
    async fn read_vectored(&mut self, bufs: &mut [&mut [u8]]) -> core::result::Result<usize, Self::Error> {
        let mut read = 0;
        for i in bufs {
            read += self.read(i).await?;
        }
        Ok(read)
    }
    async fn read_to_end(&mut self, buf: &mut Vec<u8>) -> core::result::Result<usize, Self::Error> {
        let mut read = 0;
        loop {
            buf.resize(read + 512, 0);
            let mut buf_here = &mut buf[read..];
            let read_here = self.read(buf_here).await?;
            read += read_here;
            if read_here == 0 {
                buf.resize(read, 0);
                return Ok(read);
            }
        }
    }
    async fn read_to_end_new(&mut self) -> core::result::Result<(usize, Vec<u8>), Self::Error> {
        let mut v = alloc::vec::Vec::new();
        Ok((self.read_to_end(&mut v).await?, v))
    }
    async fn read_to_string_new(&mut self) -> Result<core::result::Result<(usize, String), Self::Error>> {
        let mut v = alloc::string::String::new();
        let result = self.read_to_string(&mut v).await?;
        let result = match result {
            Ok(e) => e,
            Err(e) => return Ok(Err(e))
        };
        Ok(Ok((result, v)))
    }
    async fn read_to_end_exact(&mut self, len: usize) -> Result<core::result::Result<Vec<u8>, Self::Error>> {
        let mut v = alloc::vec::Vec::new();
        v.resize(len, 0);
        let result = self.read_exact(&mut v).await?;
        let result = match result {
            Ok(e) => e,
            Err(e) => return Ok(Err(e))
        };
        Ok(Ok(v))
    }
    async fn read_to_string(&mut self, buf: &mut String) -> Result<core::result::Result<usize, Self::Error>> {
        let mut vec = alloc::vec::Vec::new();
        let read = self.read_to_end(&mut vec).await;
        let read = match read {
            Ok(e) => e,
            Err(e) => {
                return Ok(Err(e))
            },
        };
        buf.insert_str(buf.len(), core::str::from_utf8(&vec)?);
        Ok(Ok(read))
    }
    async fn read_exact(&mut self, mut buf: &mut [u8]) -> Result<core::result::Result<(), Self::Error>> {
            
        while !buf.is_empty() {
            match self.read(buf).await {
                Ok(0) => break,
                Ok(n) => {
                    let tmp = buf;
                    buf = &mut tmp[n..];
                }
                Err(e) => {
                    return Ok(Err(e));
                }
            }
        }
        if !buf.is_empty() {
            Err(Error::new_simple(ErrorKind::UnexpectedEof))
        } else {
            Ok(Ok(()))
        }
    }
    
}