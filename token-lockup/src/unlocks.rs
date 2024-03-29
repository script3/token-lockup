use soroban_sdk::{Map, Vec};

/// Binary search for the index with the closest sequence number
/// less than or equal to the given sequence
///
/// Returns the index of the closest sequence number less than or equal to the given sequence
/// or None if no such sequence exists
///
/// # Arguments
/// * `sequence` - The sequence number to search for
/// * `unlocks` - The list of sequence numbers
pub fn find_unlock_with_sequence(sequence: u32, unlocks: &Vec<u32>) -> Option<u32> {
    let index = unlocks.binary_search(sequence);
    match index {
        Ok(i) => Some(i as u32),
        Err(i) => {
            if i == 0 {
                None
            } else {
                Some((i - 1) as u32)
            }
        }
    }
}

/// Remove past unlocks up to the given index
///
/// # Arguments
/// * `index` - The index to remove unlocks up to (inclusive)
/// * `unlocks` - The map of sequence numbers to unlock percentages
pub fn remove_past_unlocks(index: u32, unlocks: &mut Map<u32, u64>) {
    for (i, sequence) in unlocks.keys().iter().enumerate() {
        if i as u32 <= index {
            unlocks.remove(sequence);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{Env, Vec};

    #[test]
    fn test_find_unlock_with_sequence() {
        let e = Env::default();
        let sequences = Vec::from_array(&e, [1, 3, 5, 7, 9]);

        let index = find_unlock_with_sequence(0, &sequences);
        assert_eq!(index, None);

        let index = find_unlock_with_sequence(1, &sequences);
        assert_eq!(index, Some(0));

        let index = find_unlock_with_sequence(8, &sequences);
        assert_eq!(index, Some(3));

        let index = find_unlock_with_sequence(9, &sequences);
        assert_eq!(index, Some(4));

        let index = find_unlock_with_sequence(20, &sequences);
        assert_eq!(index, Some(4));
    }
}
