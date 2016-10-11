use std::ffi::CStr;
use std::ptr;

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

impl Family {
    pub fn raw(&self) -> u16 {
        use libnftnl_sys::chain::NFPROTO;

        (match *self {
            Family::Inet => NFPROTO::INET,
            Family::Ipv4 => NFPROTO::IPV4,
            Family::Ipv6 => NFPROTO::IPV6,
        }) as u16
    }
}

impl Policy {
    pub fn from_raw(raw: u32) -> Policy {
        // TODO: use constants from netfilter.h
        match raw {
            0 => Policy::Drop,
            1 => Policy::Accept,
            _ => unreachable!(),
        }
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
        let policy = Policy::from_raw(unsafe {
            chain::get_u32(raw_chain, chain::chain_attr::POLICY as u16)
        });
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
