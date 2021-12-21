pub struct Palette<T: Ord> {
    lookup: Vec<T>,
    index_bits: usize,
    index_mask: usize,
    indexes_per_slice: usize,
    elements: Vec<usize>,
    len: usize,
}

impl<T: Ord> Palette<T> {
    const MAX_INDEX_BITS: usize = usize::MAX.count_ones() as usize;

    // In a very verbose fashion, converts the total length of the palette array into
    //  a bit length (the bit length of each entry added together), and then
    //  determines how many `usize`s are required to contain that.
    const fn compute_slices(index_bits: usize, array_len: usize) -> usize {
        ((index_bits * array_len) + (Self::MAX_INDEX_BITS - 1)) / Self::MAX_INDEX_BITS
    }

    fn set_value(&mut self, index: usize, lookup_index: usize) {
        let palette_index = (index * self.index_bits) / Self::MAX_INDEX_BITS;
        let slice_offset = (index - (palette_index * self.indexes_per_slice)) * self.index_bits;
        self.elements[palette_index] = (self.elements[palette_index]
            & !(self.index_mask << slice_offset))
            | (lookup_index << slice_offset);

        debug_assert_eq!(
            self.calculate_lookup_from_index(index),
            lookup_index,
            "palette has failed to correctly set value"
        );
    }

    fn calculate_lookup_from_index(&self, index: usize) -> usize {
        let palette_index = (index * self.index_bits) / Self::MAX_INDEX_BITS;
        let slice_offset = (index - (palette_index * self.indexes_per_slice)) * self.index_bits;

        (self.elements[palette_index] & (self.index_mask << slice_offset)) >> slice_offset
    }

    fn allocate_lookup_entry(&mut self, entry: T) -> usize {
        assert!(
            !self.lookup.contains(&entry),
            "lookup entry already present"
        );

        let entry_index = self.lookup.len();
        self.lookup.push(entry);

        // Ensure we can fit the new index bits.
        if self.index_bits == Self::MAX_INDEX_BITS {
            panic!(
                "palette cannot contain more than {:?} entries",
                Self::MAX_INDEX_BITS
            );
        // Check if the index mask needs to be recalculated.
        } else if self.lookup.len() > self.index_mask {
            let new_index_bits = self.index_bits << 1;
            let new_indexes_per_slice = Self::MAX_INDEX_BITS / new_index_bits;
            let mut palette = vec![0usize; Self::compute_slices(new_index_bits, self.len())];

            // Iterate the new palette, and copy the old palette values.
            let mut index: usize = 0;
            for slice in palette.iter_mut() {
                // Temporary slice value.
                let mut new_slice = 0;

                // Iterate each lookup index in the slice.
                for slice_index in 0..new_indexes_per_slice {
                    // Find the current index's lookup index, and if it isn't the
                    //  default value, copy it into the new slice.
                    let lookup_index = self.calculate_lookup_from_index(index);
                    if lookup_index > 0 {
                        new_slice |= lookup_index << (slice_index * new_index_bits);
                    }

                    // Ensure we increment the index, to keep current.
                    index += 1;
                }

                // If our new slice isn't all default values, copy it to the palette.
                if new_slice > 0 {
                    *slice = new_slice;
                }
            }

            // Repalce our palette data with the newly expanded palette.
            self.index_bits = new_index_bits;
            self.index_mask = Self::compute_mask(new_index_bits);
            self.indexes_per_slice = new_indexes_per_slice;
            self.elements = palette;
        }

        entry_index
    }

    fn compute_mask(index_bits: usize) -> usize {
        let mut new_mask = 0;

        for bit in 0..index_bits {
            new_mask |= 1 << bit;
        }

        new_mask
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub fn lookup_len(&self) -> usize {
        self.lookup.len()
    }

    pub fn get(&self, index: usize) -> &T {
        &self.lookup[self.calculate_lookup_from_index(index)]
    }

    pub fn set(&mut self, index: usize, value: T) {
        let lookup_index = self
            .lookup
            .binary_search(&value)
            .unwrap_or_else(|_| self.allocate_lookup_entry(value));

        self.set_value(index, lookup_index);
    }

    pub fn get_lookup_value(&self, index: usize) -> &T {
        &self.lookup[index]
    }
}

impl<T: Ord + Copy> Palette<T> {
    pub fn copy_to_slice(&self, slice: &mut [T]) {
        assert!(
            slice.len() >= self.len(),
            "Slice must be large enought to accomodate palette contents."
        );

        let index = 0;
        'elements: for element_index in 0..self.len() {
            let element = self.elements[element_index];

            for bit_shift in (0..(usize::MAX.count_ones())).step_by(self.index_bits) {
                if index >= self.len() {
                    break 'elements;
                } else {
                    slice[index] = self
                        .get_lookup_value((element >> bit_shift) & self.index_mask)
                        .clone();
                }
            }
        }
    }
}
