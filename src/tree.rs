//!

use std::marker::PhantomData;
pub mod nodes;

/// A [`Node`] represents a branch in the decision tree. A node has one method ([`decide`]) that
/// takes a single reference as input (`I`) and returns a [`Decision`]. The [`Decision`] can either
/// be an action (`A`), which means no further decision will be made, or it can be another [`Node`]
/// of the same type, in which case the process will begin again.
pub trait Node<I, A> {
    fn decide(&self, input: &I) -> &Decision<Self, I, A>;
}

/// The result of a [`Node`]'s [`Node.decide`] method.
pub enum Decision<N, I, A>
where
    N: Node<I, A> + ?Sized,
{
    /// A node, meaning that another decision needs to be made.
    Node(Box<N>),
    /// An action, meaning that no more decisions need to be made and this is the final result.
    Action(A),
    _InputMarker(PhantomData<I>),
}

impl<N, I, A> Decision<N, I, A>
where
    N: Node<I, A> + ?Sized,
{
    /// Creates an action.
    pub fn action(action: A) -> Self {
        Self::Action(action)
    }
}

impl<N, I, A> Decision<N, I, A>
where
    N: Node<I, A>,
{
    /// Creates a node. This is a convenience method so you don't have to use `Box::new(...)` every
    /// time.
    pub fn node(node: N) -> Self {
        Self::Node(Box::new(node))
    }
}

/// Fundamentally a tree is pretty simple: It has a root node, handles the logic for decisions
/// and that's it.
pub struct Tree<N, I, A>
where
    N: Node<I, A>,
{
    root: N,
    _marker: (PhantomData<I>, PhantomData<A>),
}

impl<N, I, A> Tree<N, I, A>
where
    N: Node<I, A>,
{
    pub fn new(root: N) -> Self {
        Self {
            root,
            _marker: (PhantomData, PhantomData),
        }
    }

    pub fn root_node(&self) -> &N {
        &self.root
    }

    pub fn decide(&self, input: I) -> &A {
        let mut node = &self.root;
        loop {
            match node.decide(&input) {
                Decision::Node(n) => node = n,
                Decision::Action(a) => return a,
                _ => unreachable!(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    type BinTestNode = nodes::BinaryNode<isize, BinTestAction>;
    type BinTestDecision = Decision<BinTestNode, isize, BinTestAction>;
    enum BinTestAction {
        LessThanNegativeTen,
        LessThanZero,
        Zero,
        GreaterThanZero,
        GreaterThanTen,
    }
    impl BinTestAction {
        fn get_condition(action: BinTestAction) -> Box<dyn Fn(&isize) -> bool> {
            match action {
                Self::LessThanNegativeTen => Box::new(|i: &isize| *i < -10),
                Self::LessThanZero => Box::new(|i: &isize| *i < 0),
                Self::Zero => Box::new(|i: &isize| *i == 0),
                Self::GreaterThanZero => Box::new(|i: &isize| *i > 0),
                Self::GreaterThanTen => Box::new(|i: &isize| *i > 10),
            }
        }
    }

    fn create_binary_tree() -> Tree<BinTestNode, isize, BinTestAction> {
        use nodes::BinaryNode;
        let lt0_node = BinaryNode::new(
            BinTestAction::get_condition(BinTestAction::LessThanNegativeTen),
            BinTestDecision::action(BinTestAction::LessThanNegativeTen),
            BinTestDecision::action(BinTestAction::LessThanZero),
        );
        let gt0_node = BinaryNode::new(
            BinTestAction::get_condition(BinTestAction::GreaterThanTen),
            BinTestDecision::action(BinTestAction::GreaterThanTen),
            BinTestDecision::action(BinTestAction::GreaterThanZero),
        );
        let gte0_node = BinaryNode::new(
            BinTestAction::get_condition(BinTestAction::Zero),
            BinTestDecision::action(BinTestAction::Zero),
            BinTestDecision::node(gt0_node),
        );
        let root = BinaryNode::new(
            Box::new(|n: &isize| *n >= 0),
            BinTestDecision::node(gte0_node),
            BinTestDecision::node(lt0_node),
        );
        Tree::new(root)
    }

    #[test]
    fn test_binary_tree() {
        let tree = create_binary_tree();
        assert!(matches!(
            tree.decide(-11),
            BinTestAction::LessThanNegativeTen
        ));
        assert!(matches!(tree.decide(-10), BinTestAction::LessThanZero));
        assert!(matches!(tree.decide(-1), BinTestAction::LessThanZero));
        assert!(matches!(tree.decide(0), BinTestAction::Zero));
        assert!(matches!(tree.decide(1), BinTestAction::GreaterThanZero));
        assert!(matches!(tree.decide(10), BinTestAction::GreaterThanZero));
        assert!(matches!(tree.decide(11), BinTestAction::GreaterThanTen));
    }
}
