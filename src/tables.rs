use crate::tsdef::TsInt;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum TablesError {
    #[error("Invalid genome length")]
    InvalidGenomeLength,
    #[error("Invalid node: {found:?}")]
    InvalidNodeValue { found: TsInt },
    #[error("Invalid value for position: {found:?}")]
    InvalidPosition { found: i64 },
    #[error("Invalid position range: {found:?}")]
    InvalidLeftRight { found: (i64, i64) },
    #[error("Invalid value for time: {found:?}")]
    InvalidTime { found: i64 },
    #[error("Invalid value for deme: {found:?}")]
    InvalidDeme { found: i32 },
}

/// Result type for operations on tables
pub type TablesResult<T> = std::result::Result<T, TablesError>;

/// A Node of a tree sequence
pub struct Node {
    /// Birth time
    pub time: i64,
    /// Population (deme) of node
    pub deme: TsInt,
}

/// An Edge is a transmission event
pub struct Edge {
    pub left: i64,
    pub right: i64,
    /// Index of parent in a [NodeTable](type.NodeTable.html)
    pub parent: TsInt,
    /// Index of child in a [NodeTable](type.NodeTable.html)
    pub child: TsInt,
}

// TODO: It would be nice to use generics here
// to allow arbitrary types for ancestral_state
// and derived_state.

/// A Site is the location and
/// ancestral state of a tables::Mutation
pub struct Site {
    pub position: i64,
    pub ancestral_state: i8,
}

/// A Mutation is the minimal information
/// needed about a mutation to track it
/// on a tree sequence.
pub struct Mutation {
    pub node: TsInt,
    pub key: usize,
    pub site: usize,
    pub derived_state: i8,
    pub neutral: bool,
}

// TODO: do these need to be pub
pub type NodeTable = Vec<Node>;
pub type EdgeTable = Vec<Edge>;
pub type SiteTable = Vec<Site>;
pub type MutationTable = Vec<Mutation>;

fn position_non_negative(x: i64) -> TablesResult<()> {
    if x < 0 {
        return Err(TablesError::InvalidPosition { found: x });
    }
    return Ok(());
}

fn node_non_negative(x: TsInt) -> TablesResult<()> {
    if x < 0 {
        return Err(TablesError::InvalidNodeValue { found: x });
    }
    return Ok(());
}

fn time_non_negative(x: i64) -> TablesResult<()> {
    if x < 0 {
        return Err(TablesError::InvalidTime { found: x });
    }
    return Ok(());
}

fn deme_non_negative(x: i32) -> TablesResult<()> {
    if x < 0 {
        return Err(TablesError::InvalidDeme { found: x });
    }
    return Ok(());
}

pub fn edge_table_add_row(
    edges: &mut EdgeTable,
    left: i64,
    right: i64,
    parent: TsInt,
    child: TsInt,
) -> TablesResult<usize> {
    if right <= left {
        return Err(TablesError::InvalidLeftRight {
            found: (left, right),
        });
    }
    position_non_negative(left)?;
    position_non_negative(right)?;
    node_non_negative(parent)?;
    node_non_negative(child)?;

    edges.push(Edge {
        left: left,
        right: right,
        parent: parent,
        child: child,
    });

    return Ok(edges.len());
}

// FIXME: need to validate all input params and raise errors
// if invalid.
pub fn node_table_add_row(nodes: &mut NodeTable, time: i64, deme: i32) -> TablesResult<TsInt> {
    time_non_negative(time)?;
    deme_non_negative(deme)?;
    nodes.push(Node {
        time: time,
        deme: deme,
    });

    // TODO: learn if there is a way to raise error
    // automagically if overlow.
    return Ok(nodes.len() as TsInt);
}

pub fn site_table_add_row(
    sites: &mut SiteTable,
    position: i64,
    ancestral_state: i8,
) -> TablesResult<usize> {
    position_non_negative(position)?;
    sites.push(Site {
        position: position,
        ancestral_state: ancestral_state,
    });
    Ok(sites.len())
}

pub fn mutation_table_add_row(
    mutations: &mut MutationTable,
    node: TsInt,
    key: usize,
    site: usize,
    derived_state: i8,
    neutral: bool,
) -> TablesResult<usize> {
    node_non_negative(node)?;
    mutations.push(Mutation {
        node: node,
        key: key,
        site: site,
        derived_state: derived_state,
        neutral: neutral,
    });
    return Ok(mutations.len());
}

// Wow, this Ord stuff takes
// some getting used to!
// NOTE: presumably panics if NaN/Inf show up?
fn sort_edge_table(nodes: &NodeTable, edges: &mut EdgeTable) -> () {
    // NOTE: it may by more idiomatic to
    // not use a slice here, and instead allow
    // the range-checking?
    let nslice = &nodes.as_slice();
    edges.sort_by(|a, b| {
        // NOTE: rust will simply NOT ALLOW
        // i32 to be an index!
        let pindex = a.parent as usize;
        let cindex = a.parent as usize;
        let ta = nslice[pindex].time;
        let tb = nslice[cindex].time;
        if ta == tb {
            if a.parent == b.parent {
                if a.child == b.child {
                    return a.left.partial_cmp(&b.left).unwrap();
                }
                return a.parent.cmp(&b.parent);
            }
        }
        return ta.partial_cmp(&tb).unwrap().reverse();
    });
}

fn sort_mutation_table(sites: &SiteTable, mutations: &mut MutationTable) -> () {
    let sslice = &sites.as_slice();
    mutations.sort_by(|a, b| {
        let pa = sslice[a.site].position;
        let pb = sslice[b.site].position;
        return pa.partial_cmp(&pb).unwrap().reverse();
    });
}

/// A collection of node, edge, site, and mutation tables.
pub struct TableCollection {
    length_: i64, // Not visible outside of this module

    pub(crate) nodes_: NodeTable,
    pub(crate) edges_: EdgeTable,
    pub(crate) sites_: SiteTable,
    pub(crate) mutations_: MutationTable,
}

impl TableCollection {
    pub const fn new(genome_length: i64) -> TablesResult<TableCollection> {
        if genome_length < 1 {
            return Err(TablesError::InvalidGenomeLength);
        }

        return Ok(TableCollection {
            length_: genome_length,
            nodes_: NodeTable::new(),
            edges_: EdgeTable::new(),
            sites_: SiteTable::new(),
            mutations_: MutationTable::new(),
        });
    }

    pub fn add_node(&mut self, time: i64, deme: i32) -> TablesResult<TsInt> {
        return node_table_add_row(&mut self.nodes_, time, deme);
    }

    /// Add an Edge
    pub fn add_edge(
        &mut self,
        left: i64,
        right: i64,
        parent: TsInt,
        child: TsInt,
    ) -> TablesResult<usize> {
        return edge_table_add_row(&mut self.edges_, left, right, parent, child);
    }

    pub fn add_site(&mut self, position: i64, ancestral_state: i8) -> TablesResult<usize> {
        if position >= self.length_ {
            return Err(TablesError::InvalidPosition { found: position });
        }
        return site_table_add_row(&mut self.sites_, position, ancestral_state);
    }

    pub fn add_mutation(
        &mut self,
        node: TsInt,
        key: usize,
        site: usize,
        derived_state: i8,
        neutral: bool,
    ) -> TablesResult<usize> {
        return mutation_table_add_row(
            &mut self.mutations_,
            node,
            key,
            site,
            derived_state,
            neutral,
        );
    }

    pub fn get_length(&self) -> i64 {
        return self.length_;
    }

    /// Return immutable reference to the [mutation table](type.MutationTable.html)
    pub fn mutations(&self) -> &MutationTable {
        return &self.mutations_;
    }

    /// Return immutable reference to the [edge table](type.EdgeTable.html)
    pub fn edges(&self) -> &EdgeTable {
        return &self.edges_;
    }

    /// Return number of edges
    pub fn num_edges(&self) -> usize {
        return self.edges_.len();
    }

    /// Return immutable reference to [node table](type.NodeTable.html)
    pub fn nodes(&self) -> &NodeTable {
        return &self.nodes_;
    }

    /// Return immutable reference to [site table](type.SiteTable.html)
    pub fn sites(&self) -> &SiteTable {
        return &self.sites_;
    }

    pub fn sort_tables_for_simplification(&mut self) -> () {
        sort_edge_table(&self.nodes_, &mut self.edges_);
        sort_mutation_table(&self.sites_, &mut self.mutations_);
    }
}

#[cfg(test)]
mod test_tables {

    use super::*;

    #[test]
    fn test_bad_genome_length() {
        let _ = TableCollection::new(0).map_or_else(
            |x: TablesError| assert_eq!(x, TablesError::InvalidGenomeLength),
            |_| assert!(false),
        );
    }

    #[test]
    fn test_add_edge() {
        let mut tables = TableCollection::new(10).unwrap();

        let result = tables.add_edge(0, 1, 2, 3).unwrap();

        assert_eq!(1, tables.edges().len());
        assert_eq!(1, tables.num_edges());
    }

    #[test]
    fn test_add_edge_bad_positions() {
        let mut tables = TableCollection::new(10).unwrap();

        let _ = tables.add_edge(-1, 1, 1, 2).map_or_else(
            |x: TablesError| assert_eq!(x, TablesError::InvalidPosition { found: -1 }),
            |_| assert!(false),
        );

        let _ = tables.add_edge(1, -1, 1, 2).map_or_else(
            |x: TablesError| assert_eq!(x, TablesError::InvalidLeftRight { found: (1, -1) }),
            |_| assert!(false),
        );
    }

    #[test]
    fn test_add_edge_bad_nodes() {
        let mut tables = TableCollection::new(10).unwrap();

        let _ = tables.add_edge(0, 1, -1, 2).map_or_else(
            |x: TablesError| assert_eq!(x, TablesError::InvalidNodeValue { found: -1 }),
            |_| assert!(false),
        );

        let _ = tables.add_edge(0, 1, 1, -2).map_or_else(
            |x: TablesError| assert_eq!(x, TablesError::InvalidNodeValue { found: -2 }),
            |_| assert!(false),
        );
    }
}
