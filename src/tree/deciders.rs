//! Various simple (non-learning) [`Decider`]s to use.

use super::{Decider, Decision, Node, OwnedNode};
use std::{
    borrow::Borrow,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet, LinkedList, VecDeque},
    hash::Hash,
    ops::{Range, RangeFrom, RangeInclusive, RangeTo, RangeToInclusive},
};

type DecisionFn<I, A> = fn(&I) -> Node<I, A>;

type Predicate<I> = fn(&I) -> bool;

/// About as basic of a [`Decider`] as ypu can get, this simply wraps a function that takes input
/// and returns the [`Node`] to use.
pub struct SimpleDecider<I, A>(DecisionFn<I, A>);
impl<I, A> SimpleDecider<I, A> {
    pub fn new(func: DecisionFn<I, A>) -> Self {
        Self(func)
    }
}

impl<I, A> Decider<I, A> for SimpleDecider<I, A> {
    fn decide<'a: 'c, 'b: 'c, 'c>(&'a self, input: &'b I) -> Node<'c, I, A> {
        let func = self.0;
        func(input)
    }
}

/// A simple [`Decider`] that takes a predicate function and then two [`OwnedNode`]s; one to use if
/// the predicate returns true and one to use if the predicate returns false.
pub struct BinaryDecider<I, A: Clone> {
    predicate: Predicate<I>,
    on_true: OwnedNode<I, A>,
    on_false: OwnedNode<I, A>,
}
impl<I, A: Clone> BinaryDecider<I, A> {
    pub fn new(
        predicate: Predicate<I>,
        on_true: OwnedNode<I, A>,
        on_false: OwnedNode<I, A>,
    ) -> Self {
        Self {
            predicate,
            on_true,
            on_false,
        }
    }
}

impl<I, A: Clone> Decider<I, A> for BinaryDecider<I, A> {
    fn decide<'a: 'c, 'b: 'c, 'c>(&'a self, input: &'b I) -> Node<'c, I, A> {
        let predicate = self.predicate;
        if predicate(input) {
            (&self.on_true).into()
        } else {
            (&self.on_false).into()
        }
    }
}

type PredicatedNode<I, A> = (Predicate<I>, OwnedNode<I, A>);
/// A Decider that will return a given node depending on a given predicate function. The decider
/// will go through the predicates in the order that they were added and return the first node that
/// matches its predicate. If no predicates match, then a default node is returned.
pub struct PredicatedNodesDecider<I, A: Clone>(Vec<PredicatedNode<I, A>>, OwnedNode<I, A>);

impl<I, A: Clone> PredicatedNodesDecider<I, A> {
    pub fn new(default: OwnedNode<I, A>) -> Self {
        Self(Vec::new(), default)
    }

    /// Adds a new node that will be returned if the given predicate function returns `true`.
    pub fn add_node(&mut self, predicate: Predicate<I>, node: OwnedNode<I, A>) {
        self.0.push((predicate, node));
    }

    /// A convenience method that will wrap the provided answer in a [`OwnedNode::Answer`] and then
    /// add it to the decider.
    pub fn add_answer(&mut self, predicate: Predicate<I>, answer: A) {
        self.add_node(predicate, OwnedNode::Answer(answer));
    }

    /// A convenience method that will use the provided [`Decider`] to create a new [`Decision`]
    /// node and then add it to this decider.
    pub fn add_decision<D>(&mut self, predicate: Predicate<I>, decider: D)
    where
        D: Decider<I, A> + 'static,
    {
        self.add_node(predicate, OwnedNode::Decision(Decision::new(decider)));
    }
}

impl<I, A: Clone> Decider<I, A> for PredicatedNodesDecider<I, A> {
    fn decide<'a: 'c, 'b: 'c, 'c>(&'a self, input: &'b I) -> Node<'c, I, A> {
        for (predicate, node) in &self.0 {
            if predicate(input) {
                return node.into();
            }
        }
        (&self.1).into()
    }
}

type ContainsNode<I, A> = (Box<dyn Container<I>>, OwnedNode<I, A>);

/// A decider that returns a node based off of if the provided value has membership in one of the
/// provided [`Container`]s. This works in a similar fashion as a [`PredicatedNodesDecider`],
/// essentially using the container's `contains` method as its predicate.
pub struct ContainsDecider<I, A: Clone>(Vec<ContainsNode<I, A>>, OwnedNode<I, A>);

impl<I, A: Clone> ContainsDecider<I, A> {
    ///
    pub fn new(default: OwnedNode<I, A>) -> Self {
        Self(Vec::new(), default)
    }

    ///
    pub fn add_container<C>(&mut self, container: C, node: OwnedNode<I, A>)
    where
        C: Container<I> + 'static,
    {
        self.0.push((Box::new(container), node));
    }

    /// A convenience method that will wrap the provided answer in a [`OwnedNode::Answer`] and then
    /// add it to the decider.
    pub fn add_answer<C>(&mut self, container: C, answer: A)
    where
        C: Container<I> + 'static,
    {
        self.add_container(container, OwnedNode::Answer(answer));
    }

    /// A convenience method that will use the provided [`Decider`] to create a new [`Decision`]
    /// node and then add it to this decider.
    pub fn add_decision<C, D>(&mut self, container: C, decider: D)
    where
        C: Container<I> + 'static,
        D: Decider<I, A> + 'static,
    {
        self.add_container(container, OwnedNode::Decision(Decision::new(decider)));
    }
}

impl<I, A: Clone> Decider<I, A> for ContainsDecider<I, A> {
    fn decide<'a: 'c, 'b: 'c, 'c>(&'a self, input: &'b I) -> Node<'c, I, A> {
        for (container, node) in &self.0 {
            if container.contains(input) {
                return node.into();
            }
        }
        (&self.1).into()
    }
}

/// A simple trait that indicates a type can test for membership of a value. Used for
/// [`ContainsDecider`]s
pub trait Container<U> {
    /// Tests for membership.
    fn contains(&self, value: &U) -> bool;
}

impl<U, Idx> Container<U> for Range<Idx>
where
    U: PartialOrd<Idx>,
    Idx: PartialOrd<U> + PartialOrd<Idx>,
{
    fn contains(&self, value: &U) -> bool {
        Range::contains(self, value)
    }
}

impl<U, Idx> Container<U> for RangeTo<Idx>
where
    U: PartialOrd<Idx>,
    Idx: PartialOrd<U> + PartialOrd<Idx>,
{
    fn contains(&self, value: &U) -> bool {
        RangeTo::contains(self, value)
    }
}

impl<U, Idx> Container<U> for RangeFrom<Idx>
where
    U: PartialOrd<Idx>,
    Idx: PartialOrd<U> + PartialOrd<Idx>,
{
    fn contains(&self, value: &U) -> bool {
        RangeFrom::contains(self, value)
    }
}

impl<U, Idx> Container<U> for RangeInclusive<Idx>
where
    U: PartialOrd<Idx>,
    Idx: PartialOrd<U> + PartialOrd<Idx>,
{
    fn contains(&self, value: &U) -> bool {
        RangeInclusive::contains(self, value)
    }
}

impl<U, Idx> Container<U> for RangeToInclusive<Idx>
where
    U: PartialOrd<Idx>,
    Idx: PartialOrd<U> + PartialOrd<Idx>,
{
    fn contains(&self, value: &U) -> bool {
        RangeToInclusive::contains(self, value)
    }
}

impl<U, K, V> Container<U> for BTreeMap<K, V>
where
    K: Borrow<U> + Ord,
    U: Ord,
{
    fn contains(&self, value: &U) -> bool {
        self.contains_key(value)
    }
}

impl<U, V> Container<U> for BTreeSet<V>
where
    V: Borrow<U> + Ord,
    U: Ord,
{
    fn contains(&self, value: &U) -> bool {
        BTreeSet::contains(self, value)
    }
}

impl<U, K, V> Container<U> for HashMap<K, V>
where
    K: Borrow<U> + Hash + Eq,
    U: Hash + Eq,
{
    fn contains(&self, value: &U) -> bool {
        self.contains_key(value)
    }
}

impl<U, V> Container<U> for HashSet<V>
where
    V: Borrow<U> + Hash + Eq,
    U: Hash + Eq,
{
    fn contains(&self, value: &U) -> bool {
        HashSet::contains(self, value)
    }
}

impl<U> Container<U> for LinkedList<U>
where
    U: PartialEq<U>,
{
    fn contains(&self, value: &U) -> bool {
        LinkedList::contains(self, value)
    }
}

impl<U> Container<U> for VecDeque<U>
where
    U: PartialEq<U>,
{
    fn contains(&self, value: &U) -> bool {
        VecDeque::contains(self, value)
    }
}

impl<U> Container<U> for [U]
where
    U: PartialEq<U>,
{
    fn contains(&self, value: &U) -> bool {
        <[U]>::contains(self, value)
    }
}

#[cfg(test)]
mod tests {
    #![allow(dead_code)]
    use super::super::Decision;
    use super::*;
    #[derive(Copy, Clone)]
    enum TestAnswer {
        LessThanNegativeTen,
        LessThanZero,
        Zero,
        GreaterThanZero,
        GreaterThanTen,
    }
    impl TestAnswer {
        fn get_condition(action: TestAnswer) -> fn(&isize) -> bool {
            match action {
                Self::LessThanNegativeTen => |i: &isize| *i < -10,
                Self::LessThanZero => |i: &isize| *i < 0,
                Self::Zero => |i: &isize| *i == 0,
                Self::GreaterThanZero => |i: &isize| *i > 0,
                Self::GreaterThanTen => |i: &isize| *i > 10,
            }
        }

        fn get_from_number(number: isize) -> Self {
            match &number {
                -10..=-1 => Self::LessThanZero,
                0 => Self::Zero,
                1..=10 => Self::GreaterThanZero,
                _ if number < -10 => Self::LessThanNegativeTen,
                _ => Self::GreaterThanTen,
            }
        }
    }

    #[test]
    fn test_simple_decider() {
        let decider = SimpleDecider::new(|i: &isize| Node::Answer(TestAnswer::get_from_number(*i)));
        let decision = Decision::new(decider);
        assert!(matches!(
            decision.decide(&-11),
            TestAnswer::LessThanNegativeTen
        ));
        assert!(matches!(decision.decide(&-10), TestAnswer::LessThanZero));
        assert!(matches!(decision.decide(&-1), TestAnswer::LessThanZero));
        assert!(matches!(decision.decide(&0), TestAnswer::Zero));
        assert!(matches!(decision.decide(&1), TestAnswer::GreaterThanZero));
        assert!(matches!(decision.decide(&10), TestAnswer::GreaterThanZero));
        assert!(matches!(decision.decide(&11), TestAnswer::GreaterThanTen));
    }

    #[test]
    fn test_binary_decider() {
        let decision = Decision::new(BinaryDecider::new(
            |i: &isize| *i < 0,
            OwnedNode::Answer(String::from("negative")),
            OwnedNode::Answer(String::from("positive")),
        ));
        assert_eq!(decision.decide(&-1), "negative");
        assert_eq!(decision.decide(&0), "positive");
        assert_eq!(decision.decide(&1), "positive");
    }

    #[test]
    fn test_predicated_nodes_decider() {
        let mut decider = PredicatedNodesDecider::new(OwnedNode::Answer(TestAnswer::Zero));
        decider.add_answer(
            TestAnswer::get_condition(TestAnswer::LessThanNegativeTen),
            TestAnswer::LessThanNegativeTen,
        );
        decider.add_answer(
            TestAnswer::get_condition(TestAnswer::GreaterThanTen),
            TestAnswer::GreaterThanTen,
        );
        decider.add_answer(
            TestAnswer::get_condition(TestAnswer::LessThanZero),
            TestAnswer::LessThanZero,
        );
        decider.add_answer(
            TestAnswer::get_condition(TestAnswer::GreaterThanZero),
            TestAnswer::GreaterThanZero,
        );
        tree_asserts(Decision::new(decider));
    }

    #[test]
    fn test_container_decider_with_ranges() {
        let mut decider = ContainsDecider::new(OwnedNode::Answer(TestAnswer::Zero));
        decider.add_answer(-10..0, TestAnswer::LessThanZero);
        decider.add_answer(1..=10, TestAnswer::GreaterThanZero);
        decider.add_answer(11.., TestAnswer::GreaterThanTen);
        decider.add_answer(..-10, TestAnswer::LessThanNegativeTen);
        tree_asserts(Decision::new(decider));
    }

    #[test]
    fn test_binary_tree() {
        let gt_10_decision = Decision::new(BinaryDecider::new(
            |i: &isize| *i > 10,
            OwnedNode::Answer(TestAnswer::GreaterThanTen),
            OwnedNode::Answer(TestAnswer::GreaterThanZero),
        ));
        let gt_zero_decision = Decision::new(BinaryDecider::new(
            |i: &isize| *i > 0,
            OwnedNode::Decision(gt_10_decision),
            OwnedNode::Answer(TestAnswer::Zero),
        ));
        let lt_negative_10_decision = Decision::new(BinaryDecider::new(
            |i: &isize| *i < -10,
            OwnedNode::Answer(TestAnswer::LessThanNegativeTen),
            OwnedNode::Answer(TestAnswer::LessThanZero),
        ));
        let tree = Decision::new(BinaryDecider::new(
            |i: &isize| *i >= 0,
            OwnedNode::Decision(gt_zero_decision),
            OwnedNode::Decision(lt_negative_10_decision),
        ));
        tree_asserts(tree);
    }

    fn tree_asserts(tree: Decision<isize, TestAnswer>) {
        assert!(matches!(tree.decide(&-50), TestAnswer::LessThanNegativeTen));
        assert!(matches!(tree.decide(&-10), TestAnswer::LessThanZero));
        assert!(matches!(tree.decide(&-1), TestAnswer::LessThanZero));
        assert!(matches!(tree.decide(&0), TestAnswer::Zero));
        assert!(matches!(tree.decide(&1), TestAnswer::GreaterThanZero));
        assert!(matches!(tree.decide(&10), TestAnswer::GreaterThanZero));
        assert!(matches!(tree.decide(&11), TestAnswer::GreaterThanTen));
    }
}
