pub mod apply;
pub mod diff;
pub mod error;
pub mod explain;

pub use apply::apply_patch;
pub use diff::{parse_diff, FileDiff, Hunk, HunkLine};
pub use error::PatchError;
pub use explain::explain_diff;

pub type Result<T> = std::result::Result<T, PatchError>;
