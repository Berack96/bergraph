use std::{fmt::Debug, hash::Hash, num::NonZero};

#[cfg(feature = "serde")]
use bergraph_derive::serde_feature;

#[cfg_attr(feature = "serde", serde_feature)]
pub trait UnsignedId: Debug + Copy + Eq + Hash {
    type NonZero: Debug + Copy + Eq + Hash + Ord;
    fn to_nz(v: usize) -> Self::NonZero;
    fn from_nz(nz: Self::NonZero) -> usize;
    /// The plain value `v`, `None` when it does not fit.
    fn try_from_usize(v: usize) -> Option<Self>;
    fn to_usize(self) -> usize;
}

macro_rules! impl_unsigned_id {
    ($($t:ty),*) => {
        $(
            impl UnsignedId for $t {
                type NonZero = NonZero<$t>;
                #[inline(always)] fn to_nz(v: usize) -> Self::NonZero {
                    <$t>::try_from(v).ok().and_then(|v| v.checked_add(1)).and_then(NonZero::new).expect("ID overflow")
                }
                #[inline(always)] fn from_nz(nz: Self::NonZero) -> usize {
                    usize::try_from(nz.get() - 1).expect("ID overflow")
                }
                #[inline(always)] fn try_from_usize(v: usize) -> Option<Self> {
                    <$t>::try_from(v).ok()
                }
                #[inline(always)] fn to_usize(self) -> usize {
                    usize::try_from(self).expect("ID overflow")
                }
            }
        )*
    };
}

impl_unsigned_id!(u8, u16, u32, u64, u128, usize);

#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", serde_feature)]
/// Dense index of a vertex, `0..vertex_count`. Stored as `id + 1` in a `NonZero`, so
/// `Option<VertexId>` costs no extra memory.
pub struct VertexId<I: UnsignedId>(I::NonZero);

impl<I: UnsignedId> VertexId<I> {
    /// Vertex id of the dense index `id`.
    ///
    /// # Panics
    ///
    /// Panics if `id + 1` does not fit the id type `I`.
    pub fn new(id: usize) -> Self {
        Self(I::to_nz(id))
    }

    /// The dense index of this vertex.
    pub fn id(&self) -> usize {
        I::from_nz(self.0)
    }
}

// Not derived: `#[derive(PartialOrd, Ord)]` would bound `I`, not the `I::NonZero` field it
// actually compares, and fail to compile: ordered by the same dense index `id()` returns.
impl<I: UnsignedId> PartialOrd for VertexId<I> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<I: UnsignedId> Ord for VertexId<I> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}
