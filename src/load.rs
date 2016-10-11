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

            let chain = chain::alloc();
            assert!(chain != ptr::null_mut());
            let table_cstr = CString::new(table).unwrap();
            chain::set(chain,
                       chain::chain_attr::TABLE as u16,
                       table_cstr.as_ptr() as *const c_void);
            let name_cstr = CString::new(name).unwrap();
            chain::set(chain,
                       chain::chain_attr::NAME as u16,
                       name_cstr.as_ptr() as *const c_void);
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
            chain.rules = try!(Rule::load(family, &table, &name));
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

            let chain = chain::alloc();
            assert!(chain != ptr::null_mut());
            let table_cstr = CString::new(table).unwrap();
            chain::set(chain,
                       chain::chain_attr::TABLE as u16,
                       table_cstr.as_ptr() as *const c_void);
            chain::nlmsg_build_payload(header, chain);
            chain::free(chain);


            let mut chains: Vec<Chain> = vec![];
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
            for chain in &mut chains {
                chain.rules = try!(Rule::load(family, &table, &chain.name));
            }
            Ok(chains)
        }
    }
}

impl Rule {
    pub fn load(family: Family, table: &str, chain: &str) -> Result<Vec<Rule>> {

        unsafe extern "C" fn rule_cb(header: *const libmnl_sys::nlmsghdr,
                                     rules: *mut c_void) -> i32 {
            let rules = &mut *(rules as *mut Vec<Rule>);
            rules.push(Rule::decode(header).unwrap());
            callback::CallbackResult::MNL_CB_OK as i32
        }

        unsafe {
            let seq = 0;
            let mut buf = [0; libmnl_sys::socket::BUFFER_SIZE];
            let header =
                common::nlmsg_build_hdr(buf.as_mut_ptr(),
                                        nf_tables::nf_tables_msg_types::NFT_MSG_GETRULE as u16,
                                        family as u16,
                                        libmnl_sys::socket::NLM_F_DUMP,
                                        seq);

            let selector = rule::alloc();
            assert!(selector != ptr::null_mut());
            let table_cstr = CString::new(table).unwrap();
            rule::set(selector,
                      rule::attr::TABLE as u16,
                      table_cstr.as_ptr() as *const c_void);
            let chain_cstr = CString::new(chain).unwrap();
            rule::set(selector,
                      rule::attr::CHAIN as u16,
                      chain_cstr.as_ptr() as *const c_void);
            rule::nlmsg_build_payload(header, selector);
            rule::free(selector);

            let mut rules = vec![];
            {
                let mut socket = try!(socket::Socket::open());
                let mut request = Request {
                    buf: &mut buf,
                    len: (*header).nlmsg_len as usize,
                    callback: Some(rule_cb),
                    data: &mut rules,
                };
                try!(socket.exec_request(&mut request));
            }
            Ok(rules)
        }
    }
}
