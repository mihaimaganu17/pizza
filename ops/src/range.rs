//! Module defining and implementing `RangeSet` and orbiting utilities for it
use core::ops::RangeInclusive;

/// Defines a set of exclusive `RangeInclusive`s, [start..=end]. The set is limited to 32 elements
/// because we do not have an allocator.
pub struct RangeSet {
    // Elements in this set
    elements: [RangeInclusive<u64>; 32],
    // Size of the set -> how many elements we have
    size: usize,
}

impl RangeSet {
    pub const fn new() -> Self {
        const EMPTY_RANGE: RangeInclusive<u64> = RangeInclusive::new(0, 0);
        Self {
            elements: [EMPTY_RANGE; 32],
            size: 0,
        }
    }

    /// Insert a new range into the set. The range will be merged with another already existing
    /// range in the set if either:
    /// 1. The lower range end is contained in the higher range [start..end]
    /// 2. The lower range end l_end and the higher range range start h_start are consecutive
    /// integers: l_end + 1 = h_start
    pub fn insert(&mut self, range: RangeInclusive<u64>) -> Option<()> {
        // If the range does not pass of sanity checks, return None
        if !check_range(&range) { return None; }

        let mut tmp_range = range;

        'merge: loop {
            for idx in 0..self.size {
                // If the ranges do not overlap or touch, then we go to the next range
                if !overlap_or_touch(self.elements.get(idx)?, &tmp_range) {
                    continue;
                }

                // Create the new range from the 2 overlapping ones
                let start = core::cmp::min(*tmp_range.start(), *self.elements.get(idx)?.start());
                let end = core::cmp::max(*tmp_range.end(), *self.elements.get(idx)?.end());

                // We delete the range we found to be overlapping
                self.delete(idx);

                // We construct a new range which will be used for overlap test
                tmp_range = RangeInclusive::new(start, end);

                // We now test these new range for merging
                continue 'merge;
            }
            // If we reached this point, there is no more overlap. We can insert the range and
            // break
            if self.size == self.elements.len() {
                // No more room
                return None;
            } else {
                *self.elements.get_mut(self.size)? = tmp_range;
                self.size = self.size.saturating_add(1);
                break 'merge;
            }
        }
        Some(())
    }

    /// Delete the range at `index` from the set
    pub fn delete(&mut self, index: usize) -> Option<()> {
        // Index is bigger than the last position an element occupies
        if index >= self.size || self.size == 0 {
            return None;
        }

        // Move the desired to delete element to the last position
        for idx in index..self.size - 1 {
            let idx_range = self.elements[idx].clone();
            self.elements[idx] = self.elements[idx+1].clone();
            self.elements[idx+1] = idx_range;
        }

        // Decrease the size
        self.size -= 1;

        // Remove the last element, by replacing it with zero.
        self.elements[self.size] = RangeInclusive::new(0, 0);

        Some(())
    }

    pub const fn len(&self) -> usize {
        self.size
    }

}

// Checks if ther start of the range is smaller or equal than the end
fn check_range(range: &RangeInclusive<u64>) -> bool {
    range.start() <= range.end()
}

pub fn overlap_or_touch(range1: &RangeInclusive<u64>, range2: &RangeInclusive<u64>) -> bool {
    // We do not support descending ranges
    if !check_range(&range1) || !check_range(&range2) {
        return false;
    }
    // Make sure start1 is smaller than start2
    let (start1, start2) = if range1.start() <= range2.start() {
        (range1.start(), range2.start())
    } else {
        (range2.start(), range1.start())
    };
    // Make sure end1 is smaller than end2
    let (end1, end2) = if range1.end() <= range2.end() {
        // We add 1 to the lower range to make sure we check for touching
        (range1.end().saturating_add(1), range2.end())
    } else {
        // We add 1 to the lower range to make sure we check for touching
        (range2.end().saturating_add(1), range1.end())
    };

    start1 <= end2 && *start2 <= end1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let set = RangeSet::new();
        assert!(set.len() == 0);
    }

    #[test]
    fn overlap() {
        let range1 = RangeInclusive::new(10, 20);
        let range2 = RangeInclusive::new(10, 15);
        assert!(overlap_or_touch(&range1, &range2) == true);

        let range2 = RangeInclusive::new(10, 30);
        assert!(overlap_or_touch(&range1, &range2) == true);

        let range2 = RangeInclusive::new(10, 20);
        assert!(overlap_or_touch(&range1, &range2) == true);

        let range2 = RangeInclusive::new(5, 11);
        assert!(overlap_or_touch(&range1, &range2) == true);

        let range2 = RangeInclusive::new(5, 15);
        assert!(overlap_or_touch(&range1, &range2) == true);

        let range2 = RangeInclusive::new(5, 10);
        assert!(overlap_or_touch(&range1, &range2) == true);

        let range2 = RangeInclusive::new(15, 25);
        assert!(overlap_or_touch(&range1, &range2) == true);

        let range2 = RangeInclusive::new(19, 25);
        assert!(overlap_or_touch(&range1, &range2) == true);
    }

    #[test]
    fn touch() {
        let range1 = RangeInclusive::new(10, 20);
        let range2 = RangeInclusive::new(5, 9);
        assert!(overlap_or_touch(&range1, &range2) == true);
        let range2 = RangeInclusive::new(21, 30);
        assert!(overlap_or_touch(&range1, &range2) == true);
    }

    #[test]
    fn no_overlap() {
        let range1 = RangeInclusive::new(10, 20);
        let range2 = RangeInclusive::new(5, 8);
        assert!(overlap_or_touch(&range1, &range2) == false);

        let range2 = RangeInclusive::new(22, 30);
        assert!(overlap_or_touch(&range1, &range2) == false);

        let range2 = RangeInclusive::new(15, 5);
        assert!(overlap_or_touch(&range1, &range2) == false);

        let range2 = RangeInclusive::new(15, 12);
        assert!(overlap_or_touch(&range1, &range2) == false);

        let range2 = RangeInclusive::new(25, 12);
        assert!(overlap_or_touch(&range1, &range2) == false);
    }

    #[test]
    fn range_overlap() {
    }
}
