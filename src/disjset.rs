use crate::disjset::Roots::{DisJoint, Same};

/// DisjointSet according to my Data Structures and Algorithms textbook
pub struct DisjSet {
    nodes: Vec<Option<usize>>,
}

/// Status of 2 sets relative to each other.
#[derive(Debug)]
pub enum Roots {
    Same(usize),
    DisJoint(usize, usize),
}

impl DisjSet {
    pub fn new(size: usize) -> DisjSet {
        return DisjSet {
            nodes: vec![None; size],
        };
    }

    /// join the the 2 sets
    pub fn union(&mut self, r1: usize, r2: usize) {
        self.nodes[r2] = Some(r1);
    }

    /// Get the root set for each of the given nodes.
    pub fn find_roots(&mut self, a: usize, b: usize) -> Roots {
        let ra = self.find(a);
        let rb = self.find(b);
        return if ra == rb { Same(ra) } else { DisJoint(ra, rb) };
    }

    /// find the root of the given set.
    /// uses the path compression method to optimize subsequent lookups.
    pub fn find(&mut self, c: usize) -> usize {
        return match self.nodes[c] {
            None => c,
            Some(p) => {
                let np = self.find(p);
                self.nodes[c] = Some(np);
                np
            }
        };
    }

    /// lookup root of the given set.
    /// this method differs from find in that it doesn't compress the path.
    pub fn lookup(&self, c: usize) -> usize {
        return match self.nodes[c] {
            None => c,
            Some(s) => self.lookup(s),
        };
    }

    /// get the size of the nodes in the universe.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// get count of nodes not in a union set with other nodes
    pub fn distinct_sets(&self) -> usize {
        self.nodes
            .iter()
            .filter(|c| match c {
                None => true,
                _ => false,
            })
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_union() -> Result<(), String> {
        let mut m = DisjSet::new(5);
        assert_eq!(m.distinct_sets(), 5);
        assert_eq!(m.find(1), 1);
        assert_eq!(m.find(2), 2);
        m.union(1, 2);
        assert_eq!(m.distinct_sets(), 4);
        assert_eq!(m.find(1), 1);
        assert_eq!(m.find(2), 1);
        assert_eq!(m.lookup(2), 1);

        Ok(())
    }
}
