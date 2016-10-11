use std::ptr;
use std::os::raw::c_void;

use num::FromPrimitive;
use libc;
use libmnl_sys::{socket, callback};

error_chain! {
    errors {
        Errno {
            description("errno error")
            display("{}", {
                unsafe{libc::perror(b"".as_ptr() as *const i8);}
                ""
            })
        }
    }
}

pub trait Request {
    fn data(&mut self) -> *mut c_void;
    fn len(&self) -> usize;
    fn callback(&mut self) -> (callback::cb_t, *mut c_void);
}

pub struct Socket {
    handle: *mut socket::socket,
}

impl Socket {
    pub fn open() -> Result<Socket> {
        unsafe {
            let socket = Socket { handle: socket::open(socket::NETLINK_NETFILTER) };
            if socket.handle == ptr::null_mut() {
                try!(Err(ErrorKind::Errno))
            }
            if socket::bind(socket.handle, 0, 0) < 0 {
                try!(Err(ErrorKind::Errno))
            }
            Ok(socket)
        }
    }

    pub fn port_id(&self) -> u32 {
        unsafe { socket::get_portid(self.handle) }
    }

    pub fn exec_request<R: Request>(&mut self, req: &mut R) -> Result<()> {
        use libmnl_sys::callback::CallbackResult;

        unsafe {
            if socket::sendto(self.handle, req.data(), req.len()) < 0 {
                try!(Err(ErrorKind::Errno))
            }

            let mut buf = [0i8; socket::BUFFER_SIZE];
            let seq = 0;
            loop {
                let received =
                    socket::recvfrom(self.handle, buf.as_mut_ptr() as *mut c_void, buf.len());
                if received < 0 {
                    try!(Err(ErrorKind::Errno))
                } else if received == 0 {
                    break;
                }

                let (callback, data) = req.callback();
                let ret = callback::run(buf.as_ptr() as *const c_void,
                                        received as usize,
                                        seq,
                                        self.port_id(),
                                        callback,
                                        data);
                match CallbackResult::from_i32(ret) {
                    Some(CallbackResult::MNL_CB_OK) => (),
                    Some(CallbackResult::MNL_CB_STOP) => {
                        break;
                    }
                    Some(CallbackResult::MNL_CB_ERROR) => try!(Err(ErrorKind::Errno)),
                    _ => unreachable!(),
                }
            }
            Ok(())
        }
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        if self.handle != ptr::null_mut() {
            unsafe {
                socket::close(self.handle);
            }
            self.handle = ptr::null_mut();
        }
    }
}
