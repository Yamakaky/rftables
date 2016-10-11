use std::collections::HashSet;
use std::net;

#[derive(Debug)]
pub struct Table {
    pub name: String,
    pub family: Family,
    pub chains: Vec<Chain>,
    pub sets: Vec<Set>,
}

pub type Set = HashSet<Item>;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Item {
    Ipv4(net::Ipv4Addr),
    Ipv6(net::Ipv6Addr),
    String(String),
    // TODO: variable size
    Integer(u64),
    // TODO: variable size
    Bitmask(u64),
    Mac(String),
}

enum_from_primitive! {
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub enum Family {
        Inet = 1,
        Ipv4 = 2,
        Ipv6 = 10,
    }
}

#[derive(Debug, Default)]
pub struct Chain {
    pub name: String,
    pub rules: Vec<Rule>,
    pub hook: Option<Hook>,
    pub policy: Policy,
    pub bytes: u64,
    pub packets: u64,
}

#[derive(Debug)]
pub struct Rule {
    pub expressions: Vec<Expression>,
}

#[derive(Debug)]
pub enum Expression {
}

#[derive(Debug)]
pub struct Hook {
    pub type_: HookType,
    pub type2: HookType2,
    pub priority: i64,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum HookType {
    Filter,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum HookType2 {
    Input,
    Output,
    Forward,
}

enum_from_primitive! {
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub enum Policy {
        Drop = 0,
        Accept = 1,
    }
}

impl Default for Policy {
    fn default() -> Policy {
        Policy::Accept
    }
}
