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
pub mod treap;
pub mod avl;

use std::iter::FromIterator;

use crate::data::*;
use crate::locators;

/// This trait is the top-level trait that the different trees implement.
/// Every tree that implements this trait can be used directly by the functions
/// immediately in this trait.
/// More advanced use can be achieved by using walkers, which must be implemented.
pub trait SomeTree<D : Data> :
    SomeEntry<D> + FromIterator<D::Value> + IntoIterator<Item=D::Value> + Default
    where for<'a> &'a mut Self : SomeTreeRef<D>
{
    /// Compute the summary of a subsegment.
    fn segment_summary<L>(&mut self, locator : L) -> D::Summary where
        L : locators::Locator<D>;

    /// Apply an action on a subsegment.
    fn act_segment<L>(&mut self, action : D::Action, locator : L) where
        L : locators::Locator<D>;
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
    /// return `Err(())` if it is in an empty spot.
    fn go_left(&mut self) -> Result<(), ()>;
    /// returns `Err(())` if it is in an empty spot.
    fn go_right(&mut self) -> Result<(), ()>;

    /// If successful, returns whether or not the previous current value was the left son.
    /// If already at the root of the tree, returns `Err(())`.
    fn go_up(&mut self) -> Result<bool, ()>;

    /// Goes to the root.
    /// May restructure the tree while doing so. For example, in splay trees,
    /// this splays the current node.
    fn go_to_root(&mut self) {
        while let Ok(_) = self.go_up()
            {}
    }


    /// Returns the current depth in the tree.
	/// The convention is, the root is at depth zero
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
}

/// Methods that ask to read the contents of the current tree/subtree.
/// These methods are common to the trees themselves and to the walkers.
pub trait SomeEntry<D : Data> {
    // TODO: reconsider
    // fn value_with_mut(&mut self) -> Option<&D::Value>;

    /// Lets you modify the value, and after you modified it, rebuilds the node.
    /// If the current position is empty, returns [`None`].
    fn with_value<F, R>(&mut self, f : F) -> Option<R> where 
        F : FnOnce(&mut D::Value) -> R;
    
    /// Returns [`true`] if the current tree/subtree is empty.
    fn is_empty(&self) -> bool {
        self.left_subtree_summary().is_none()
    }

    /// Returns a summary of just the current node.
    /// Returns the empty summary if at an empty position.
    fn node_summary(&self) -> D::Summary;

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

    /// Returns the summary of the subtree of this node's left son.
    fn left_subtree_summary(&self) -> Option<D::Summary>;
    /// Returns the summary of the subtree of this node's right son.
    fn right_subtree_summary(&self) -> Option<D::Summary>;

    /// Applies the action on the current node.
    fn act_node(&mut self, action : D::Action) -> Option<()>;

    /// Applies the given action on this subtree.
    fn act_subtree(&mut self, action : D::Action);

    /// Applies the given action on this node's left son.
    fn act_left_subtree(&mut self, action : D::Action) -> Option<()>;
    /// Applies the given action on this node's right son.
    fn act_right_subtree(&mut self, action : D::Action) -> Option<()>;
}






/// Trait for trees that can be modified, i.e., values can be inserted and deleted.
/// This trait is a workaround for current rust type inference limitations.
/// It allows to write generic code for a tree type that has a modifiable walker.
/// Intuitively it should've been enough to require
/// `T : SomeTree<D>, for<'a> &'a mut T : SomeTreeRef<D>, for<'a> <&'a mut T as SomeTreeRef<D>>::Walker : ModifiableWalker`.
/// However, that doesn't work. Instead, use `for<'a> &'a mut T : ModifiableTreeRef<D>`.
pub trait ModifiableTreeRef<D : Data> : SomeTreeRef<D, Walker = Self::ModifiableWalker> {
    /// Inner type that ideally shouldn't be used - just use `Self::Walker`.
    type ModifiableWalker : ModifiableWalker<D>;
}

/// This is a trait for walkers that allow inserting and deleting values.
pub trait ModifiableWalker<D : Data> : SomeWalker<D> {
    /// Inserts the value into the tree at the current empty position.
    /// If the current position is not empty, return [`None`].
    /// When the function returns, the walker will be at the position the node
    /// was inserted.
    fn insert(&mut self, value : D::Value) -> Option<()>;

    /// Removes the current value from the tree, and returns it.
    /// If currently at an empty position, returns [`None`].
    /// May end up at any possible location, depending on the tree type.
    fn delete(&mut self) -> Option<D::Value>;
}


/// Trait for trees that can concatenate.
/// I wanted this to be the same trait family as SplittableWalker, but the current rustc type solver didn't let me.
/// It's enough to only implement any one of the three methods - they're all implemented in terms of each other.
pub trait ConcatenableTree<D : Data> : SomeTree<D> where for<'a> &'a mut Self : SomeTreeRef<D>,
{
    /// Concatenates the two inputs into one tree.
    fn concatenate(mut left : Self, right : Self) -> Self {
        left.concatenate_right(right);
        left
    }

    /// Concatenates the other tree to the right of this tree.
    fn concatenate_right(&mut self, mut other : Self) {
        let left = std::mem::replace(self, Default::default());
        other.concatenate_left(left);
        *self = other;
    }

    /// Concatenates the other tree to the left of this tree.
    fn concatenate_left(&mut self, other : Self) {
        let right = std::mem::replace(self, Default::default());
        *self = Self::concatenate(other, right);
    }
}
/// Trait for trees that can be split and concatenated.
/// Require this kind of tree if you want to use reversal actions on segments of your tree.
pub trait SplittableTreeRef<D : Data> : SomeTreeRef<D, Walker=Self::SplittableWalker> + Sized
{
    /// Inner type that ideally shouldn't be used - just use the original tree type.
    type T;
    /// Inner type that ideally shouldn't be used - just use `Self::Walker`.
    type SplittableWalker : SplittableWalker<D,T=Self::T>;
}

/// Walkers that can split a tree into two.
pub trait SplittableWalker<D : Data> : ModifiableWalker<D> {
    type T;

    /// Split out everything to the right of the current position, if it is an empty position.
    /// Otherwise returns [`None`].
    fn split_right(&mut self) -> Option<Self::T>;

    /// Split out everything to the left of the current position, if it is an empty position.
    /// Otherwise returns [`None`].
    fn split_left(&mut self) -> Option<Self::T>;
}