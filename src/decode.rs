use std::ffi::CStr;
use std::ptr;

use enum_primitive::FromPrimitive;
use libc;
use libmnl_sys::nlmsghdr;

use types::*;

error_chain! {
    errors {
        Parse {
            description("parse error")
            display("{}", {
                unsafe{libc::perror(b"".as_ptr() as *const i8);}
                ""
            })
        }
    }
}

impl Table {
    pub fn decode(header: *const nlmsghdr) -> Result<Table> {
        use libnftnl_sys::table;

        let raw_table = unsafe {
            let raw_table = table::alloc();
            assert!(raw_table != ptr::null_mut());
            if table::nlmsg_parse(header, raw_table) < 0 {
                try!(Err(ErrorKind::Parse))
            }
            raw_table
        };
        let name = unsafe {
            CStr::from_ptr(table::get_str(raw_table, table::attr::NAME as u16))
                .to_str()
                .unwrap()
                .into()
        };
        let family = unsafe {
            Family::from_u32(table::get_u32(raw_table, table::attr::FAMILY as u16)).unwrap()
        };
        Ok(Table {
            name: name,
            family: family,
            chains: vec![],
            sets: vec![],
        })
    }
}

impl Chain {
    pub fn decode(header: *const nlmsghdr) -> Result<Chain> {
        use libnftnl_sys::chain;

        let raw_chain = unsafe {
            let raw_chain = chain::alloc();
            assert!(raw_chain != ptr::null_mut());
            if chain::nlmsg_parse(header, raw_chain) < 0 {
                try!(Err(ErrorKind::Parse))
            }
            raw_chain
        };
        let name = unsafe {
            CStr::from_ptr(chain::get_str(raw_chain, chain::chain_attr::NAME as u16))
                .to_str()
                .unwrap()
                .into()
        };
        let packets = unsafe { chain::get_u64(raw_chain, chain::chain_attr::PACKETS as u16) };
        let bytes = unsafe { chain::get_u64(raw_chain, chain::chain_attr::BYTES as u16) };
        let policy = Policy::from_u32(unsafe {
                chain::get_u32(raw_chain, chain::chain_attr::POLICY as u16)
            })
            .unwrap();
        Ok(Chain {
            name: name,
            packets: packets,
            bytes: bytes,
            policy: policy,
            rules: vec![],
            hook: None,
        })
    }
}
