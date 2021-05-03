//! This module implements the tree traits for the [`BasicTree`] and [`BasicWalker`]
//! It is mostly a separate file from the main module file, since it's a private module, and its
//! contents are re-exported.

use super::*;
use super::super::*; // crate::trees::*
use crate::telescope::NO_VALUE_ERROR;

impl<D : Data> SomeTree<D> for BasicTree<D> {
    fn segment_summary<L>(&mut self, locator : L) -> D::Summary where
        L : Locator<D> {
		methods::segment_summary(self, &locator)
    }

    fn act_segment<L>(&mut self, action : D::Action, locator : L) where
        L : Locator<D> {
       methods::act_segment(self, action, &locator);
    }
}

impl<D : Data> Default for BasicTree<D> {
    fn default() -> Self {
        Empty
    }
}

impl<D : Data> std::iter::FromIterator<D::Value> for BasicTree<D> {
    fn from_iter<T: IntoIterator<Item = D::Value>>(iter: T) -> Self {
        iterators::build(iter.into_iter())
    }
}


impl<D : Data> IntoIterator for BasicTree<D> {
	type Item = D::Value;
	type IntoIter = iterators::OwningIterator<D, fn(D::Summary, &D::Value, D::Summary) -> methods::LocResult>;
	fn into_iter(self) -> Self::IntoIter {
		iterators::OwningIterator::new(self, methods::all::<D>)
	}
}


impl<'a, D : Data, T> SomeTreeRef<D> for &'a mut BasicTree<D, T> {
    type Walker = BasicWalker<'a, D, T>;

    fn walker(self) -> Self::Walker {
        BasicWalker::new(self)
    }
}

impl<'a, D : Data, T> SomeWalker<D> for BasicWalker<'a, D, T> {
	fn go_left(&mut self) -> Result<(), ()> {
		let mut frame = self.vals.last().expect(crate::telescope::NO_VALUE_ERROR).clone();
		let res = self.tel.extend_result( |tree| {
			if let Some(node) = tree.node_mut() {
				// update values
				frame.right = node.node_summary() + node.right.subtree_summary() + frame.right;
				node.left.access();
				Ok(&mut node.left)
			} else { Err(()) }
		}
		);
		// push side information
		if res.is_ok() {
			self.is_left.push(true); // went left
			self.vals.push(frame);
		}
		return res;
	}
	
	fn go_right(&mut self) -> Result<(), ()> {
		let mut frame = self.vals.last().expect(crate::telescope::NO_VALUE_ERROR).clone();
		let res = self.tel.extend_result( |tree| {
			if let Some(node) = tree.node_mut() {
				// update values
				frame.left = frame.left + node.left.subtree_summary() + node.node_summary();
				
				node.right.access();
				Ok(&mut node.right)
			} else { Err(()) }
		}
		);
		// push side information
		if res.is_ok() {
			self.is_left.push(false); // went right
			self.vals.push(frame);
		}
		return res;
	}

	fn go_up(&mut self) -> Result<bool, ()> {
		match self.is_left.pop() {
			None => Err(()),
			Some(b) => { 
				self.tel.pop().expect(NO_VALUE_ERROR);
				self.vals.pop().expect(NO_VALUE_ERROR);
				self.tel.rebuild();
				Ok(b)
			},
		}
	}

	fn depth(&self) -> usize {
		self.is_left.len()
	}

	fn far_left_summary(&self) -> D::Summary {
		self.vals.last().expect(NO_VALUE_ERROR).left
	}
	fn far_right_summary(&self) -> D::Summary {
		self.vals.last().expect(NO_VALUE_ERROR).right
	}

	// fn inner(&self) -> &BasicTree<A> {
    //     &*self.tel
    // }

	fn value(&self) -> Option<&D::Value> {
		let value = self.tel.node()?.node_value_clean();
		Some(value)
	}
}

impl<D : Data, T> SomeEntry<D> for BasicTree<D, T> {
	fn node_summary(&self) -> D::Summary {
		match self.node() {
			None => D::EMPTY,
			Some(node) => node.node_summary()
		}
	}

	fn subtree_summary(&self) -> D::Summary {
		if let Some(node) = self.node() {
			node.subtree_summary()
		} else { D::EMPTY }
	}

    fn left_subtree_summary(&self) -> Option<D::Summary> {
        let res = self.node()?.left.subtree_summary();
		Some(res)
    }

    fn right_subtree_summary(&self) -> Option<D::Summary> {
        let res = self.node()?.right.subtree_summary();
		Some(res)
    }

    fn with_value<F, R>(&mut self, f : F) -> Option<R> where 
        F : FnOnce(&mut D::Value) -> R {
		self.access();
		let value = &mut self.node_mut()?.node_value;
        let res = f(value);
    	self.rebuild();
    	Some(res)
    }

    fn act_subtree(&mut self, action : D::Action) {
        if let Some(node) = self.node_mut() {
			node.act(action);
		}
    }

    fn act_node(&mut self, action : D::Action) -> Option<()> {
        let node = self.node_mut()?;
		node.act_value(action);
		node.rebuild();
		Some(())
    }

    fn act_left_subtree(&mut self, action : D::Action) -> Option<()> {
        let node = self.node_mut()?;
		node.access();
		node.left.act_subtree(action);
		node.rebuild();
		Some(())
    }

    fn act_right_subtree(&mut self, action : D::Action) -> Option<()> {
        let node = self.node_mut()?;
		node.access();
		node.right.act_subtree(action);
		node.rebuild();
		Some(())
    }
}

impl<'a, D : Data, T> SomeEntry<D> for BasicWalker<'a, D, T> {
	fn node_summary(&self) -> D::Summary {
		self.tel.node_summary()
	}

    fn subtree_summary(&self) -> D::Summary {
        self.tel.subtree_summary()
    }

    fn left_subtree_summary(&self) -> Option<D::Summary> {
        self.tel.left_subtree_summary()
    }

    fn right_subtree_summary(&self) -> Option<D::Summary> {
        self.tel.right_subtree_summary()
    }

    fn with_value<F, R>(&mut self, f : F) -> Option<R> where 
        F : FnOnce(&mut D::Value) -> R {
        self.tel.with_value(f)
    }

    fn act_subtree(&mut self, action : D::Action) {
        self.tel.act_subtree(action);
		self.tel.access();
    }

    fn act_node(&mut self, action : D::Action) -> Option<()> {
		let node = self.tel.node_mut()?;
		D::act_value(action, &mut node.node_value);
		node.rebuild();
		Some(())
    }

    fn act_left_subtree(&mut self, action : D::Action) -> Option<()> {
        let node = self.tel.node_mut()?;
		node.left.act_subtree(action);
		node.rebuild();
		Some(())
    }

    fn act_right_subtree(&mut self, action : D::Action) -> Option<()> {
        let node = self.tel.node_mut()?;
		node.right.act_subtree(action);
		node.rebuild();
		Some(())
    }
}

impl<'a, D : Data> ModifiableWalker<D> for BasicWalker<'a, D> {
    fn insert(&mut self, value : D::Value) -> Option<()> {
		match *self.tel {
			Empty => {
				*self.tel = BasicTree::new(BasicNode::new(value));
				Some(())
			},
			_ => None,
		}
    }

    fn delete(&mut self) -> Option<D::Value> {
        let tree = std::mem::replace(&mut *self.tel, BasicTree::Empty);
		let mut boxed_node = match tree {
			Empty => return None,
			Root(boxed_node) => boxed_node,
		};
		match boxed_node.right.is_empty() {
			false => {
				let mut walker = boxed_node.right.walker();
				methods::next_filled(&mut walker).unwrap();
				let tree2 = std::mem::replace(&mut *walker.tel, BasicTree::Empty);
				drop(walker);

				let mut boxed_node2 = match tree2 {
					Empty => unreachable!(),
					Root(boxed_node) => boxed_node,
				};
				assert!(boxed_node2.left.is_empty());
				assert!(boxed_node2.right.is_empty());
				boxed_node2.left = boxed_node.left;
				boxed_node2.right = boxed_node.right;
				boxed_node2.rebuild();
				*self.tel = BasicTree::Root(boxed_node2);
			},
			true => {
				*self.tel = boxed_node.left;
			},
		}
		Some(boxed_node.node_value)
    }
}