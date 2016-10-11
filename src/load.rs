use std::ffi::CString;
use std::ptr;
use std::os::raw::*;

use libmnl_sys::{self, callback};
use libnftnl_sys::*;

use types::*;
use socket;

impl Chain {
    pub fn load(family: Family, table: &str, name: &str) -> Result<Chain, ()> {

        unsafe extern "C" fn chain_cb(header: *const libmnl_sys::nlmsghdr, chain: *mut c_void) -> i32 {
            let chain = &mut *(chain as *mut Chain);
            *chain = Chain::decode(header).unwrap();
            libmnl_sys::callback::CallbackResult::MNL_CB_STOP as i32
        }

        unsafe {
            struct Request<'a> {
                buf: [i8; libmnl_sys::socket::BUFFER_SIZE],
                len: usize,
                chain: &'a mut Chain,
            }

            impl<'a> socket::Request for Request<'a> {
                fn data(&mut self) -> *mut c_void {
                    self.buf.as_mut_ptr() as *mut c_void
                }
                fn len(&self) -> usize {
                    self.len
                }
                fn callback(&mut self) -> (callback::cb_t, *mut c_void) {
                    (Some(chain_cb), self.chain as *mut Chain as *mut c_void)
                }
            }

            let seq = 0;
            let mut buf = [0; libmnl_sys::socket::BUFFER_SIZE];
            let header =
                common::nlmsg_build_hdr(buf.as_mut_ptr(),
                nf_tables::nf_tables_msg_types::NFT_MSG_GETCHAIN as u16,
                family.raw(),
                4,
                seq);
            let table = CString::new(table).unwrap();

            let chain = chain::alloc();
            assert!(chain != ptr::null_mut());
            chain::set(chain,
                       chain::chain_attr::TABLE as u16,
                       table.as_ptr() as *const c_void);
            let name = CString::new(name).unwrap();
            chain::set(chain,
                       chain::chain_attr::NAME as u16,
                       name.as_ptr() as *const c_void);
            chain::nlmsg_build_payload(header, chain);
            chain::free(chain);


            let mut chain = Chain::default();
            {
                let mut socket = socket::Socket::open().unwrap();
                let mut request = Request {
                    buf: buf,
                    len: (*header).nlmsg_len as usize,
                    chain: &mut chain,
                };
                socket.exec_request(&mut request).unwrap();
            }
            Ok(chain)
        }
    }

}
