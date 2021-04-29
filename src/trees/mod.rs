//! The trees module.
//! This module contains:
//! * Traits that all implementations of trees should implement
//! * Specific implementations of trees
//!
//! The [`SomeWalker`] trait implements walking through a tree. This includes dealing with the borrow
//! checking problems of recursive structures (using [`crate::telescope`]), and rebalancing the tree.
//! Therefore, walkers can't guarantee that the tree won't change as you walk through them.
//! 
//! Currently this module is limited to trees which are based on the [`basic_tree::BasicTree`] type.

pub mod methods;
pub mod basic_tree;
pub mod splay;

use crate::data::*;
pub trait SomeTree<D : Data> : SomeEntry<D> where
    for<'a> &'a mut Self : SomeTreeRef<D> {

    fn into_inner(self) -> basic_tree::BasicTree<D>;
    fn new() -> Self;
    fn from_inner(tree : basic_tree::BasicTree<D>) -> Self;

}

/// This is a workaround for not having Generic Associated Types in Rust yet.
/// Really, the type [`Self::Walker`] should have been defined in [`SomeTree`] and
/// should have been generic in a lifetime parameter.
pub trait SomeTreeRef<D : Data> {
    type Walker : SomeWalker<D>;
    fn walker(self) -> Self::Walker;
}



/// The Walker trait implements walking through a tree.
/// This includes dealing with the borrow checking problems of recursive structures (using Telescope),
/// and rebalancing the tree.
/// Therefore, walkers can't guarantee that the tree won't change as you walk through them.
///
/// The walker should be able to walk on any of the existing nodes, or any empty position just near them.
/// i.e., The walker can also be in the position of a son of an existing node, where there isn't
/// a node yet.
/// The method [`SomeEntry::is_empty()`] can tell whether you are at an empty position. Trying to move downward from an
/// empty position produces an error value.
pub trait SomeWalker<D : Data> : SomeEntry<D> {
    /// return [`Err(())`] if it is in an empty spot.
    fn go_left(&mut self) -> Result<(), ()>;
    /// returns [`Err(())`] if it is in an empty spot.
    fn go_right(&mut self) -> Result<(), ()>;

    /// If successful, returns whether or not the previous current value was the left son.
    /// If already at the root of the tree, returns `Err(())`.
    /// If you have a SplayTree, you shouldn't use this method too much, or you might lose the
    /// SplayTree's complexity properties - see documentation aboud splay tree.
    fn go_up(&mut self) -> Result<bool, ()>;


    /// Returns the current depth in the tree.
    fn depth(&self) -> usize;

    /// Returns a summary of all the values to the left of this point,
    /// That are not children of this point.
    fn far_left_summary(&self) -> D::Summary;
    /// Returns a summary of all the values to the right of this point,
    /// That are not children of this point.
    fn far_right_summary(&self) -> D::Summary;

    /// Returns a summary of all the values to the left of this point.
    /// If the walker is in a non empty spot, this does not include the current node.
    fn left_summary(&self) -> D::Summary {
        let left = self.far_left_summary();
        match self.left_subtree_summary() {
            Some(subtree) => left + subtree,
            None => left,
        }
    }
    /// Returns a summary of all the values to the right of this point.
    /// If the walker is in a non empty spot, this does not include the current node.
    fn right_summary(&self) -> D::Summary {
        let right = self.far_right_summary();
        match self.right_subtree_summary() {
            Some(subtree) => subtree + right,
            None => right,
        }
    }

    /// This function is here since only walkers can guarantee that the current value
    /// is clean.
    fn value(&self) -> Option<&D::Value>;

    // // TODO: consider switching this function to a function that
    // // returns the inner node directly.
    // fn inner(&self) -> &basic_tree::BasicTree<D>;
}

/// Methods that ask to read the contents of the current tree/position.
/// These methods are common to the trees themselves and to the walkers.
pub trait SomeEntry<D : Data> {
    // TODO: switch uses to `with_value`.
    /// Note: this function can be used to violate the invariant that the current node of a walker
    /// is "clean", which in fact means that the summary value is correct.
    /// To prevent this, call `self.rebuild()` after modifying, or use
    /// [`Self::with_value`] instead.
    fn value_mut(&mut self) -> Option<&mut D::Value>;

    // useful example implementation:
    // let res = f(self.value_mut()?);
    // self.access();
    // Some(res)
    fn with_value<F, R>(&mut self, f : F) -> Option<R> where 
        F : FnOnce(&mut D::Value) -> R;

        

    /// Returns a summary of just the current node.
    /// Returns the empty summary if at an empty position.
    fn node_summary(&self) -> D::Summary;
    
    /// Returns [`true`] if the current tree is empty.
    fn is_empty(&self) -> bool {
        self.left_subtree_summary().is_none()
    }

    /// Returns the summary of all values in this node's subtree.
	///```
    /// use orchard::*;
	/// use orchard::basic_tree::*;
	/// use orchard::example_data::StdNum;
    ///
	/// let tree : BasicTree<StdNum> = (1..=8).collect();
	/// let summary = tree.subtree_summary();
    ///
	/// assert_eq!(summary.size, 8);
	/// assert_eq!(summary.sum, 36);
	/// assert_eq!(summary.max, Some(8));
	/// assert_eq!(summary.min, Some(1));
	/// # tree.assert_correctness();
	///```
    fn subtree_summary(&self) -> D::Summary;

    fn left_subtree_summary(&self) -> Option<D::Summary>;
    fn right_subtree_summary(&self) -> Option<D::Summary>;

    /// only writes if it is in an empty position. if the position isn't empty,
    /// return Err(()).
    fn insert_new(&mut self, value : D::Value) -> Result<(), ()>;

    fn act_subtree(&mut self, action : D::Action);
}

