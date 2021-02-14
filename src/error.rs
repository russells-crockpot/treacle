#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("The decision is not a branch.")]
    DecisionIsNotABranch,
    #[error("The decision is not an answer.")]
    DecisionIsNotAAnswer,
}

pub type Result<R> = std::result::Result<R, Error>;
