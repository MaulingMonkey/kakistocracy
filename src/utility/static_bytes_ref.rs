use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::ops::*;



#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct StaticBytesRef(pub &'static [u8]);

impl StaticBytesRef { fn cmp_data(&self) -> (*const u8, usize) { (self.0.as_ptr(), self.len()) } }

impl PartialEq  for StaticBytesRef { fn eq(&self, other: &Self) -> bool { self.cmp_data() == other.cmp_data() } }
impl Eq         for StaticBytesRef {}
impl PartialOrd for StaticBytesRef { fn partial_cmp(&self, other: &Self) -> Option<Ordering> { self.cmp_data().partial_cmp(&other.cmp_data()) } }
impl Ord        for StaticBytesRef { fn cmp(&self, other: &Self) -> Ordering { self.cmp_data().cmp(&other.cmp_data()) } }
impl Hash       for StaticBytesRef { fn hash<H: Hasher>(&self, state: &mut H) { self.cmp_data().hash(state) } }

impl AsRef<[u8]>    for StaticBytesRef { fn as_ref(&self) -> &[u8] { self.0 } }
impl Deref          for StaticBytesRef { fn deref (&self) -> &[u8] { self.0 } type Target = [u8]; }
