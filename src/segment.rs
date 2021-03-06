use crate::tsdef::{IdType, Position};

/// A segment is a half-open
/// interval of [``crate::Position``]s
/// associated with a [``crate::Node``].
///
/// This type is public primarily because
/// it is the value element of a
/// [``crate::EdgeBuffer``].
#[derive(Clone, Copy)]
pub struct Segment {
    /// Left edge of interval
    pub left: Position,
    /// Right edge of interval
    pub right: Position,
    /// The node
    pub node: IdType,
}

impl Segment {
    /// Create a new instance.
    pub fn new(left: Position, right: Position, node: IdType) -> Self {
        Segment { left, right, node }
    }
}
