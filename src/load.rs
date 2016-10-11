use std::ffi::CString;
use std::ptr;
use std::os::raw::*;

use libmnl_sys::{self, callback};
use libnftnl_sys::*;

use types::*;
use socket;

error_chain! {
    links {
        socket::Error, socket::ErrorKind, Socket;
    }
}

struct Request<'a, T: 'a> {
    buf: &'a mut [i8; libmnl_sys::socket::BUFFER_SIZE],
    len: usize,
    callback: callback::cb_t,
    data: &'a mut T,
}

impl<'a, T> socket::Request for Request<'a, T> {
    fn data(&mut self) -> *mut c_void {
        self.buf.as_mut_ptr() as *mut c_void
    }
    fn len(&self) -> usize {
        self.len
    }
    fn callback(&mut self) -> (callback::cb_t, *mut c_void) {
        (self.callback, self.data as *mut T as *mut c_void)
    }
}

impl Table {
    pub fn load_all() -> Result<Vec<Table>> {

        unsafe extern "C" fn table_cb(header: *const libmnl_sys::nlmsghdr,
                                      tables: *mut c_void)
                                      -> i32 {
            let tables = &mut *(tables as *mut Vec<Table>);
            tables.push(Table::decode(header).unwrap());
            callback::CallbackResult::MNL_CB_OK as i32
        }

        unsafe {
            let seq = 0;
            let mut buf = [0; libmnl_sys::socket::BUFFER_SIZE];
            let header =
                common::nlmsg_build_hdr(buf.as_mut_ptr(),
                                        nf_tables::nf_tables_msg_types::NFT_MSG_GETTABLE as u16,
                                        chain::NFPROTO::UNSPEC as u16,
                                        libmnl_sys::socket::NLM_F_DUMP,
                                        seq);

            let mut tables: Vec<Table> = vec![];
            {
                let mut socket = try!(socket::Socket::open());
                let mut request = Request {
                    buf: &mut buf,
                    len: (*header).nlmsg_len as usize,
                    callback: Some(table_cb),
                    data: &mut tables,
                };
                try!(socket.exec_request(&mut request));
            }
            for table in &mut tables {
                table.chains = try!(Chain::load_table(table.family, &table.name));
            }
            Ok(tables)
        }
    }
}

impl Chain {
    pub fn load(family: Family, table: &str, name: &str) -> Result<Chain> {

        unsafe extern "C" fn chain_cb(header: *const libmnl_sys::nlmsghdr,
                                      chain: *mut c_void)
                                      -> i32 {
            let chain = &mut *(chain as *mut Chain);
            *chain = Chain::decode(header).unwrap();
            callback::CallbackResult::MNL_CB_STOP as i32
        }

        unsafe {
            let seq = 0;
            let mut buf = [0; libmnl_sys::socket::BUFFER_SIZE];
            let header =
                common::nlmsg_build_hdr(buf.as_mut_ptr(),
                                        nf_tables::nf_tables_msg_types::NFT_MSG_GETCHAIN as u16,
                                        family as u16,
                                        libmnl_sys::socket::NLM_F_ACK,
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
                let mut socket = try!(socket::Socket::open());
                let mut request = Request {
                    buf: &mut buf,
                    len: (*header).nlmsg_len as usize,
                    callback: Some(chain_cb),
                    data: &mut chain,
                };
                try!(socket.exec_request(&mut request));
            }
            Ok(chain)
        }
    }

    pub fn load_table(family: Family, table: &str) -> Result<Vec<Chain>> {

        unsafe extern "C" fn chain_cb(header: *const libmnl_sys::nlmsghdr,
                                      chain: *mut c_void)
                                      -> i32 {
            let chains = &mut *(chain as *mut Vec<Chain>);
            chains.push(Chain::decode(header).unwrap());
            callback::CallbackResult::MNL_CB_OK as i32
        }

        unsafe {
            let seq = 0;
            let mut buf = [0; libmnl_sys::socket::BUFFER_SIZE];
            let header =
                common::nlmsg_build_hdr(buf.as_mut_ptr(),
                                        nf_tables::nf_tables_msg_types::NFT_MSG_GETCHAIN as u16,
                                        family as u16,
                                        libmnl_sys::socket::NLM_F_DUMP,
                                        seq);
            let table = CString::new(table).unwrap();

            let chain = chain::alloc();
            assert!(chain != ptr::null_mut());
            chain::set(chain,
                       chain::chain_attr::TABLE as u16,
                       table.as_ptr() as *const c_void);
            chain::nlmsg_build_payload(header, chain);
            chain::free(chain);


            let mut chains = vec![];
            {
                let mut socket = try!(socket::Socket::open());
                let mut request = Request {
                    buf: &mut buf,
                    len: (*header).nlmsg_len as usize,
                    callback: Some(chain_cb),
                    data: &mut chains,
                };
                try!(socket.exec_request(&mut request));
            }
            Ok(chains)
        }
    }
}
