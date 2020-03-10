use crate::handle::RefBox;
use crate::pool::PoolInner;
use std::ptr::NonNull;

pub(crate) type ElementPointer<A> = NonNull<RefBox<A>>;
pub(crate) type PoolPointer<A> = NonNull<PoolInner<A>>;
