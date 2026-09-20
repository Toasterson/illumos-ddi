//! `sys/errno.h`: error numbers returned by `cb_ops(9S)` entry points and
//! many `9F` functions. Generated from the gate header; do not edit by hand.

use crate::types::c_int;

/// Not super-user
pub const EPERM: c_int = 1;
/// No such file or directory
pub const ENOENT: c_int = 2;
/// No such process
pub const ESRCH: c_int = 3;
/// interrupted system call
pub const EINTR: c_int = 4;
/// I/O error
pub const EIO: c_int = 5;
/// No such device or address
pub const ENXIO: c_int = 6;
/// Arg list too long
pub const E2BIG: c_int = 7;
/// Exec format error
pub const ENOEXEC: c_int = 8;
/// Bad file number
pub const EBADF: c_int = 9;
/// No children
pub const ECHILD: c_int = 10;
/// Resource temporarily unavailable
pub const EAGAIN: c_int = 11;
/// Not enough core
pub const ENOMEM: c_int = 12;
/// Permission denied
pub const EACCES: c_int = 13;
/// Bad address
pub const EFAULT: c_int = 14;
/// Block device required
pub const ENOTBLK: c_int = 15;
/// Mount device busy
pub const EBUSY: c_int = 16;
/// File exists
pub const EEXIST: c_int = 17;
/// Cross-device link
pub const EXDEV: c_int = 18;
/// No such device
pub const ENODEV: c_int = 19;
/// Not a directory
pub const ENOTDIR: c_int = 20;
/// Is a directory
pub const EISDIR: c_int = 21;
/// Invalid argument
pub const EINVAL: c_int = 22;
/// File table overflow
pub const ENFILE: c_int = 23;
/// Too many open files
pub const EMFILE: c_int = 24;
/// Inappropriate ioctl for device
pub const ENOTTY: c_int = 25;
/// Text file busy
pub const ETXTBSY: c_int = 26;
/// File too large
pub const EFBIG: c_int = 27;
/// No space left on device
pub const ENOSPC: c_int = 28;
/// Illegal seek
pub const ESPIPE: c_int = 29;
/// Read only file system
pub const EROFS: c_int = 30;
/// Too many links
pub const EMLINK: c_int = 31;
/// Broken pipe
pub const EPIPE: c_int = 32;
/// Math arg out of domain of func
pub const EDOM: c_int = 33;
/// Math result not representable
pub const ERANGE: c_int = 34;
/// No message of desired type
pub const ENOMSG: c_int = 35;
/// Identifier removed
pub const EIDRM: c_int = 36;
/// Channel number out of range
pub const ECHRNG: c_int = 37;
/// Level 2 not synchronized
pub const EL2NSYNC: c_int = 38;
/// Level 3 halted
pub const EL3HLT: c_int = 39;
/// Level 3 reset
pub const EL3RST: c_int = 40;
/// Link number out of range
pub const ELNRNG: c_int = 41;
/// Protocol driver not attached
pub const EUNATCH: c_int = 42;
/// No CSI structure available
pub const ENOCSI: c_int = 43;
/// Level 2 halted
pub const EL2HLT: c_int = 44;
/// Deadlock condition.
pub const EDEADLK: c_int = 45;
/// No record locks available.
pub const ENOLCK: c_int = 46;
/// Operation canceled
pub const ECANCELED: c_int = 47;
/// Operation not supported
pub const ENOTSUP: c_int = 48;
/// Disc quota exceeded
pub const EDQUOT: c_int = 49;
/// invalid exchange
pub const EBADE: c_int = 50;
/// invalid request descriptor
pub const EBADR: c_int = 51;
/// exchange full
pub const EXFULL: c_int = 52;
/// no anode
pub const ENOANO: c_int = 53;
/// invalid request code
pub const EBADRQC: c_int = 54;
/// invalid slot
pub const EBADSLT: c_int = 55;
/// file locking deadlock error
pub const EDEADLOCK: c_int = 56;
/// bad font file fmt
pub const EBFONT: c_int = 57;
/// process died with the lock
pub const EOWNERDEAD: c_int = 58;
/// lock is not recoverable
pub const ENOTRECOVERABLE: c_int = 59;
/// Device not a stream
pub const ENOSTR: c_int = 60;
/// no data (for no delay io)
pub const ENODATA: c_int = 61;
/// timer expired
pub const ETIME: c_int = 62;
/// out of streams resources
pub const ENOSR: c_int = 63;
/// Machine is not on the network
pub const ENONET: c_int = 64;
/// Package not installed
pub const ENOPKG: c_int = 65;
/// The object is remote
pub const EREMOTE: c_int = 66;
/// the link has been severed
pub const ENOLINK: c_int = 67;
/// advertise error
pub const EADV: c_int = 68;
/// srmount error
pub const ESRMNT: c_int = 69;
/// Communication error on send
pub const ECOMM: c_int = 70;
/// Protocol error
pub const EPROTO: c_int = 71;
/// locked lock was unmapped
pub const ELOCKUNMAPPED: c_int = 72;
/// Facility is not active
pub const ENOTACTIVE: c_int = 73;
/// multihop attempted
pub const EMULTIHOP: c_int = 74;
/// trying to read unreadable message
pub const EBADMSG: c_int = 77;
/// path name is too long
pub const ENAMETOOLONG: c_int = 78;
/// value too large to be stored in data type
pub const EOVERFLOW: c_int = 79;
/// given log. name not unique
pub const ENOTUNIQ: c_int = 80;
/// f.d. invalid for this operation
pub const EBADFD: c_int = 81;
/// Remote address changed
pub const EREMCHG: c_int = 82;
/// Can't access a needed shared lib.
pub const ELIBACC: c_int = 83;
/// Accessing a corrupted shared lib.
pub const ELIBBAD: c_int = 84;
/// .lib section in a.out corrupted.
pub const ELIBSCN: c_int = 85;
/// Attempting to link in too many libs.
pub const ELIBMAX: c_int = 86;
/// Attempting to exec a shared library.
pub const ELIBEXEC: c_int = 87;
/// Illegal byte sequence.
pub const EILSEQ: c_int = 88;
/// Unsupported file system operation
pub const ENOSYS: c_int = 89;
/// Symbolic link loop
pub const ELOOP: c_int = 90;
/// Restartable system call
pub const ERESTART: c_int = 91;
/// if pipe/FIFO, don't sleep in stream head
pub const ESTRPIPE: c_int = 92;
/// directory not empty
pub const ENOTEMPTY: c_int = 93;
/// Too many users (for UFS)
pub const EUSERS: c_int = 94;
/// Socket operation on non-socket
pub const ENOTSOCK: c_int = 95;
/// Destination address required
pub const EDESTADDRREQ: c_int = 96;
/// Message too long
pub const EMSGSIZE: c_int = 97;
/// Protocol wrong type for socket
pub const EPROTOTYPE: c_int = 98;
/// Protocol not available
pub const ENOPROTOOPT: c_int = 99;
/// Protocol not supported
pub const EPROTONOSUPPORT: c_int = 120;
/// Socket type not supported
pub const ESOCKTNOSUPPORT: c_int = 121;
/// Operation not supported on socket
pub const EOPNOTSUPP: c_int = 122;
/// Protocol family not supported
pub const EPFNOSUPPORT: c_int = 123;
/// Address family not supported by
pub const EAFNOSUPPORT: c_int = 124;
/// Address already in use
pub const EADDRINUSE: c_int = 125;
/// Can't assign requested address
pub const EADDRNOTAVAIL: c_int = 126;
/// Network is down
pub const ENETDOWN: c_int = 127;
/// Network is unreachable
pub const ENETUNREACH: c_int = 128;
/// Network dropped connection because
pub const ENETRESET: c_int = 129;
/// Software caused connection abort
pub const ECONNABORTED: c_int = 130;
/// Connection reset by peer
pub const ECONNRESET: c_int = 131;
/// No buffer space available
pub const ENOBUFS: c_int = 132;
/// Socket is already connected
pub const EISCONN: c_int = 133;
/// Socket is not connected
pub const ENOTCONN: c_int = 134;
/// Can't send after socket shutdown
pub const ESHUTDOWN: c_int = 143;
/// Too many references: can't splice
pub const ETOOMANYREFS: c_int = 144;
/// Connection timed out
pub const ETIMEDOUT: c_int = 145;
/// Connection refused
pub const ECONNREFUSED: c_int = 146;
/// Host is down
pub const EHOSTDOWN: c_int = 147;
/// No route to host
pub const EHOSTUNREACH: c_int = 148;
/// operation already in progress
pub const EALREADY: c_int = 149;
/// operation now in progress
pub const EINPROGRESS: c_int = 150;
/// Stale NFS file handle
pub const ESTALE: c_int = 151;
