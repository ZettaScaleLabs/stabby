/// BTrees built for sharing
///
/// ArcBTrees are build to be cheap to clone through atomic reference counts.
///
/// Mutating an ArcBTree that whose ownership is shared will result in every node from the insertion spot
/// to the root to be copied, while the remaining nodes will just see their reference counts increase.
pub mod arc_btree;
