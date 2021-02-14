//!

pub mod deciders;
pub use deciders::SimpleDecider;

/// The engine of a [`Decision`] node that decides what to do next by returning a new [`Node`]
pub trait Decider<I, A> {
    /// Makes a decsion based off of the provided input.
    fn decide<'a: 'c, 'b: 'c, 'c>(&'a self, input: &'b I) -> Node<'c, I, A>;
}

/// A decsion to be made. At it's core, this is essentially just a wrapper for a [`Decider`].
pub struct Decision<I, A>(Box<dyn Decider<I, A>>);

impl<I, A> Decision<I, A> {
    /// Creates a new [`Decision`] using the given decider.
    pub fn new<D>(decider: D) -> Self
    where
        D: Decider<I, A> + 'static,
    {
        Self(Box::new(decider))
    }

    /// Runs the decision, going through any child nodes if needed.
    pub fn decide(&self, input: &I) -> A {
        let mut decision = self;
        loop {
            match decision.0.decide(input) {
                Node::Answer(a) => return a,
                Node::Decision(d) => decision = d,
            }
        }
    }
}

/// A node in a decision tree. This can either be a reference to a [`Decision`] or it can be an
/// answer.
pub enum Node<'a, I, A> {
    Decision(&'a Decision<I, A>),
    Answer(A),
}

/// A convenience enum. While similar to the [`Node`] enum, this one owns its data and references
/// to it can be converted into a [`Node`].
pub enum OwnedNode<I, A: Clone> {
    Decision(Decision<I, A>),
    Answer(A),
}

impl<'a, I, A: Clone> From<&'a OwnedNode<I, A>> for Node<'a, I, A> {
    fn from(value: &'a OwnedNode<I, A>) -> Node<'a, I, A> {
        match value {
            OwnedNode::Decision(d) => Self::Decision(&d),
            OwnedNode::Answer(a) => Self::Answer(a.clone()),
        }
    }
}
