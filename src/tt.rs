use cozy_chess::Move;

pub const DEFAULT_TT_SIZE_MB: usize = 16;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Bound {
    Exact,
    LowerBound, // Fail-high (beta cutoff)
    UpperBound, // Fail-low (all-nodes)
}

#[derive(Copy, Clone, Debug)]
pub struct TTEntry {
    pub key: u64,
    pub score: i16,
    pub depth: u8,
    pub bound: Bound,
    pub best_move: Option<Move>,
    pub age: u8,
}

pub struct TranspositionTable {
    entries: Vec<Option<TTEntry>>,
    size: usize,
    age: u8,
}

impl TranspositionTable {
    pub fn new(size_mb: usize) -> Self {
        let entry_size = std::mem::size_of::<Option<TTEntry>>();
        let total_bytes = size_mb * 1024 * 1024;
        let count = (total_bytes / entry_size).next_power_of_two();
        Self {
            entries: vec![None; count],
            size: count,
            age: 0,
        }
    }

    pub fn resize(&mut self, size_mb: usize) {
        let entry_size = std::mem::size_of::<Option<TTEntry>>();
        let total_bytes = size_mb * 1024 * 1024;
        let count = (total_bytes / entry_size).next_power_of_two();
        self.entries = vec![None; count];
        self.size = count;
        self.age = 0;
    }

    pub fn clear(&mut self) {
        self.entries.fill(None);
        self.age = 0;
    }

    pub fn new_search(&mut self) {
        self.age = self.age.wrapping_add(1);
    }

    #[inline(always)]
    fn index(&self, hash: u64) -> usize {
        (hash as usize) & (self.size - 1)
    }

    pub fn probe(&self, hash: u64, ply: usize) -> Option<(TTEntry, Option<i32>)> {
        let idx = self.index(hash);
        let entry = self.entries[idx]?;
        if entry.key == hash {
            let adjusted_score = self.score_from_tt(entry.score as i32, ply);
            Some((entry, Some(adjusted_score)))
        } else {
            None
        }
    }

    pub fn store(
        &mut self,
        hash: u64,
        depth: u8,
        score: i32,
        bound: Bound,
        best_move: Option<Move>,
        ply: usize,
    ) {
        let idx = self.index(hash);
        let tt_score = self.score_to_tt(score, ply) as i16;

        if let Some(existing) = &self.entries[idx] {
            // Replacement scheme:
            // Always replace if different hash, or if depth is >= existing, or if older search
            if existing.key == hash && existing.depth > depth && existing.age == self.age {
                // Keep existing deeper entry, but update best_move if we found one
                if best_move.is_some() && existing.best_move.is_none() {
                    let mut updated = *existing;
                    updated.best_move = best_move;
                    self.entries[idx] = Some(updated);
                }
                return;
            }
        }

        self.entries[idx] = Some(TTEntry {
            key: hash,
            score: tt_score,
            depth,
            bound,
            best_move,
            age: self.age,
        });
    }

    // Adjust mate score when storing to make it distance from root, not search node
    fn score_to_tt(&self, score: i32, ply: usize) -> i32 {
        if score > crate::search::MATE_BOUND {
            score + (ply as i32)
        } else if score < -crate::search::MATE_BOUND {
            score - (ply as i32)
        } else {
            score
        }
    }

    // Adjust mate score when probing to convert back to distance from current node
    fn score_from_tt(&self, score: i32, ply: usize) -> i32 {
        if score > crate::search::MATE_BOUND {
            score - (ply as i32)
        } else if score < -crate::search::MATE_BOUND {
            score + (ply as i32)
        } else {
            score
        }
    }
}
