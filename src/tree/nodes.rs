//! This modules contains various kinds of prebuild [`Node`]s.
use super::{Decision, Node};
use std::collections::HashMap;

/// A convenience type for predicate functions.
pub type Predicate<I> = Box<dyn Fn(&I) -> bool>;

///
pub struct BinaryNode<I, A>
where
    Self: Node<I, A>,
{
    condition: Predicate<I>,
    on_true: Decision<Self, I, A>,
    on_false: Decision<Self, I, A>,
}

impl<I, A> BinaryNode<I, A>
where
    Self: Node<I, A>,
{
    pub fn new(
        condition: Predicate<I>,
        on_true: Decision<Self, I, A>,
        on_false: Decision<Self, I, A>,
    ) -> Self {
        Self {
            condition,
            on_true,
            on_false,
        }
    }
}

impl<I, A> Node<I, A> for BinaryNode<I, A> {
    fn decide(&self, input: &I) -> &Decision<Self, I, A> {
        let condition = &self.condition;
        if condition(input) {
            &self.on_true
        } else {
            &self.on_false
        }
    }
}

///
pub trait MappingNode<I, A>: Node<I, A> + Sized {
    fn add_decision(&mut self, value: I, decision: Decision<Self, I, A>);

    fn default_decision(&self) -> &Decision<Self, I, A>;

    fn get_decision(&self, input: &I) -> Option<&Decision<Self, I, A>>;

    fn add_action(&mut self, value: I, action: A) {
        self.add_decision(value, Decision::Action(action));
    }

    fn add_node(&mut self, value: I, node: Self) {
        self.add_decision(value, Decision::node(node));
    }

    fn decide(&self, input: &I) -> &Decision<Self, I, A> {
        match self.get_decision(input) {
            Some(d) => d,
            None => self.default_decision(),
        }
    }
}

///
pub struct HashMapNode<I, A>(HashMap<I, Decision<Self, I, A>>, Decision<Self, I, A>)
where
    I: std::hash::Hash + Eq;

impl<I, A> HashMapNode<I, A>
where
    I: std::hash::Hash + Eq,
{
    pub fn new(default: Decision<Self, I, A>) -> Self {
        Self(HashMap::new(), default)
    }
}

impl<I, A> MappingNode<I, A> for HashMapNode<I, A>
where
    I: std::hash::Hash + Eq,
{
    fn add_decision(&mut self, value: I, decision: Decision<Self, I, A>) {
        self.0.insert(value, decision);
    }

    fn get_decision(&self, input: &I) -> Option<&Decision<Self, I, A>> {
        self.0.get(input)
    }

    fn default_decision(&self) -> &Decision<Self, I, A> {
        &self.1
    }
}

impl<I, A> Node<I, A> for HashMapNode<I, A>
where
    I: std::hash::Hash + Eq,
{
    fn decide(&self, input: &I) -> &Decision<Self, I, A> {
        MappingNode::decide(self, input)
    }
}

///
pub struct PredicateListNode<I, A>(
    Vec<(Predicate<I>, Decision<Self, I, A>)>,
    Decision<Self, I, A>,
);

impl<I, A> PredicateListNode<I, A> {
    pub fn new(default: Decision<Self, I, A>) -> Self {
        Self(Vec::new(), default)
    }

    pub fn add_decision(&mut self, predicate: Predicate<I>, decision: Decision<Self, I, A>) {
        self.0.push((predicate, decision));
    }

    pub fn add_action(&mut self, predicate: Predicate<I>, action: A) {
        self.add_decision(predicate, Decision::action(action));
    }

    pub fn add_node(&mut self, predicate: Predicate<I>, node: Self) {
        self.add_decision(predicate, Decision::node(node));
    }
}

impl<I, A> Node<I, A> for PredicateListNode<I, A> {
    fn decide(&self, input: &I) -> &Decision<Self, I, A> {
        for (predicate, decision) in self.0.iter() {
            if predicate(input) {
                return decision;
            }
        }
        &self.1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::Decision;

    #[test]
    fn test_binary_node() {
        let on_true = Decision::action(1);
        let on_false = Decision::action(2);
        let node = BinaryNode::new(Box::new(|n: &isize| *n >= 0), on_true, on_false);
        let gt0 = node.decide(&1);
        assert!(matches!(gt0, Decision::Action(1)));
        let lt0 = node.decide(&-1);
        assert!(matches!(lt0, Decision::Action(2)));
    }
}
