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

    // Delete the range at `index` from the set
    fn delete(&mut self, index: usize) -> Option<()> {
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

    /// Consume a `RangeInclusive` from the existing available ranges in the set. This operation
    /// trimms any range that overlaps with the one we want to consume, deletes any range that is
    /// equal to the one we want to consume and splits any range that has a middle overlap with
    /// the one we want to consume.
    pub fn consume(&mut self, range: &RangeInclusive<u64>) -> Option<()> {
        for idx in 0..self.size {
            // It is safe to clone here because this is just a local copy we want to use
            let entry = self.elements[idx].clone();
            // If the 2 ranges equal, we just delete the entry
            if &entry == range {
                return self.delete(idx);
            }
            // Based on how we implemented insert, there cannot exist 2 ranges that overlap or
            // touch already in the set. As such, we have to find the range in the set that fully
            // contains the input range.
            if !contains(&entry, range) { continue; }

            // At this point, we know our desired range is contained in the entry. We now have to
            // find how much of the entry remains, after we remove the desired range.

            // If the 2 starts are equal, we just update the start of the current entry
            if entry.start() == range.start() {
                self.elements[idx] =
                    RangeInclusive::new(range.end().saturating_add(1), *entry.end());
                return Some(());
            // If the 2 ends are equal, we just update the end of the current entry
            } else if entry.end() == range.end() {
                self.elements[idx] =
                    RangeInclusive::new(*entry.start(), range.start().saturating_sub(1));
                return Some(());
            // At this point the range we want to extract actually splits our current range in 2.
            } else {
                // Check we have enough room to further split this range. Which means that if the
                // set is too fragmented, we either need to increase the capacity of the set, or
                // create a new set.
                if self.size == self.elements.len() {
                    return None;
                }
                // We do have room, so we keep the low end in this current entry and insert the
                // high end into a new entry
                self.elements[idx] =
                    RangeInclusive::new(*entry.start(), range.start().saturating_sub(1));
                *self.elements.get_mut(self.size)? =
                    RangeInclusive::new(range.end().saturating_add(1), *entry.end());
                self.size = self.size.saturating_add(1);
                return Some(());
            }
        }
        // If we reached this point, it means there are no ranges that satify our call, so we
        // return `None`
        None
    }

    /// Discards the `range` from the available set. This does not care if the range already exists
    /// or not. If a certain portion of the range is already discarded, we move past that and
    /// continue discarding.
    pub fn discard(&mut self, range: &RangeInclusive<u64>) -> Option<()> {
        Some(())
    }
    pub fn len(&self) -> usize {
        self.size
    }

    pub fn ranges(&self) -> &[RangeInclusive<u64>] {
        &self.elements[..self.size]
    }

    pub fn sum(&self) -> u64 {
        let iter = RangeSetIter::new(&self);
        let sum = iter.fold(0u64, |acc, range| {
            // We compute the size of the range. We add 1 because we use `RangeInclusive`
            let size = range.end().saturating_add(1).saturating_sub(*(range.start()));
            acc.saturating_add(size)
        });
        sum
    }
}

pub struct RangeSetIter<'a> {
    set: &'a RangeSet,
    idx: usize,
}

impl<'a> RangeSetIter<'a> {
    pub fn new(set: &'a RangeSet) -> Self {
        Self { set, idx: 0 }
    }
}

impl<'a> Iterator for RangeSetIter<'a> {
    type Item = &'a RangeInclusive<u64>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx == self.set.len() {
            None
        } else {
            let range = self.set.ranges().get(self.idx)?;
            self.idx = self.idx.saturating_add(1);
            Some(range)
        }
    }
}

// Checks if ther start of the range is smaller or equal than the end
fn check_range(range: &RangeInclusive<u64>) -> bool {
    range.start() <= range.end()
}

fn contains(container: &RangeInclusive<u64>, contained: &RangeInclusive<u64>) -> bool {
    // We do not support descending ranges
    if !check_range(&container) || !check_range(&contained) {
        return false;
    }
    container.start() <= contained.start() && contained.end() <= container.end()
}

fn overlap_or_touch(range1: &RangeInclusive<u64>, range2: &RangeInclusive<u64>) -> bool {
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
    fn range_set_no_overlap() {
        let mut set = RangeSet::new();
        set.insert(RangeInclusive::new(0, 10)).expect("Could not insert range");
        set.insert(RangeInclusive::new(15, 20)).expect("Could not insert range");
        set.insert(RangeInclusive::new(30, 40)).expect("Could not insert range");

        assert!(
            set.ranges() ==
            &[
                RangeInclusive::new(0, 10),
                RangeInclusive::new(15, 20),
                RangeInclusive::new(30, 40),
            ]
        );
        assert!(set.len() == 3);
    }

    #[test]
    fn range_set_simple_overlap() {
        let mut set = RangeSet::new();
        set.insert(RangeInclusive::new(0, 10)).expect("Could not insert range");
        set.insert(RangeInclusive::new(15, 20)).expect("Could not insert range");
        set.insert(RangeInclusive::new(30, 40)).expect("Could not insert range");
        set.insert(RangeInclusive::new(19, 25)).expect("Could not insert range");
        set.insert(RangeInclusive::new(27, 35)).expect("Could not insert range");

        assert!(
            set.ranges() ==
            &[
                RangeInclusive::new(0, 10),
                RangeInclusive::new(15, 25),
                RangeInclusive::new(27, 40),
            ]
        );
        assert!(set.len() == 3);
    }

    #[test]
    fn range_set_recursive_overlap() {
        let mut set = RangeSet::new();
        set.insert(RangeInclusive::new(0, 10)).expect("Could not insert range");
        set.insert(RangeInclusive::new(15, 20)).expect("Could not insert range");
        set.insert(RangeInclusive::new(30, 40)).expect("Could not insert range");
        set.insert(RangeInclusive::new(19, 25)).expect("Could not insert range");
        set.insert(RangeInclusive::new(24, 35)).expect("Could not insert range");

        assert!(
            set.ranges() ==
            &[
                RangeInclusive::new(0, 10),
                RangeInclusive::new(15, 40),
            ]
        );
        assert!(set.len() == 2);
    }

    #[test]
    fn range_set_recursive_touching_overlap() {
        let mut set = RangeSet::new();
        set.insert(RangeInclusive::new(0, 10)).expect("Could not insert range");
        set.insert(RangeInclusive::new(15, 20)).expect("Could not insert range");
        set.insert(RangeInclusive::new(30, 40)).expect("Could not insert range");
        set.insert(RangeInclusive::new(21, 29)).expect("Could not insert range");
        set.insert(RangeInclusive::new(11, 14)).expect("Could not insert range");

        assert!(
            set.ranges() ==
            &[
                RangeInclusive::new(0, 40),
            ]
        );
        assert!(set.len() == 1);
    }

    #[test]
    fn range_set_basic_consume() {
        let mut set = RangeSet::new();
        set.insert(RangeInclusive::new(0, 10)).expect("Could not insert range");
        set.insert(RangeInclusive::new(15, 20)).expect("Could not insert range");
        set.insert(RangeInclusive::new(30, 40)).expect("Could not insert range");
        set.insert(RangeInclusive::new(50, 100)).expect("Could not insert range");

        set.consume(&RangeInclusive::new(0, 5)).expect("Could not consume");
        set.consume(&RangeInclusive::new(6, 10)).expect("Could not consume");
        set.consume(&RangeInclusive::new(15, 20)).expect("Could not consume");
        set.consume(&RangeInclusive::new(33, 39)).expect("Could not consume");
        set.consume(&RangeInclusive::new(55, 100)).expect("Could not consume");

        assert!(set.consume(&RangeInclusive::new(49, 50)).is_none());

        assert!(
            set.ranges() ==
            &[
                RangeInclusive::new(30, 32),
                RangeInclusive::new(50, 54),
                RangeInclusive::new(40, 40),
            ]
        );
        assert!(set.len() == 3);
    }

    #[test]
    fn range_set_sum() {
        let mut set = RangeSet::new();
        set.insert(RangeInclusive::new(0, 10)).expect("Could not insert range");
        set.insert(RangeInclusive::new(15, 20)).expect("Could not insert range");
        set.insert(RangeInclusive::new(30, 40)).expect("Could not insert range");

        let sum = set.sum();

        assert!(sum == 28);
    }
}
