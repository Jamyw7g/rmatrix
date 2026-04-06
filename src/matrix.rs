use nanorand::{Rng, WyRand};

pub const EMPTY: u8 = 0;
pub const CH_BEGIN: u8 = 33;
pub const CH_END: u8 = 127;
pub const BLANK: u8 = b' ';
const FLAG_HEAD: u8 = 1;

#[derive(Clone, Debug)]
pub struct Item {
    pub val: u8,
    flags: u8,
}

impl Default for Item {
    fn default() -> Self {
        Item {
            val: EMPTY,
            flags: 0,
        }
    }
}

impl Item {
    #[inline]
    pub fn is_head(&self) -> bool {
        self.flags & FLAG_HEAD != 0
    }

    #[inline]
    fn clear_head(&mut self) {
        self.flags = 0;
    }

    #[inline]
    fn set_head(&mut self) {
        self.flags = FLAG_HEAD;
    }
}

pub struct Matrix {
    rng: WyRand,
    rand_pool: [u64; 64],
    rand_idx: usize,

    buffer: Vec<Item>,
    spaces: Vec<u16>,
    lengths: Vec<u16>,
}

impl Matrix {
    pub fn new(cols: usize, rows: usize) -> Self {
        let mut rng = WyRand::new();
        let mut buffer = vec![Item::default(); (rows + 1) * cols];
        let mut spaces = vec![0; cols];
        let mut lengths = vec![0; cols];

        let lines = rows as u16;
        for j in (0..cols).step_by(2) {
            buffer[cols + j].val = BLANK;
            lengths[j] = 3 + rng.generate_range(0..lines as u16 - 3);
            spaces[j] = 1 + rng.generate_range(0..lines as u16);
        }

        Self {
            rng,
            rand_pool: [0; 64],
            rand_idx: 64,
            buffer,
            spaces,
            lengths,
        }
    }

    pub fn next(&mut self) -> &[Item] {
        let Self {
            rng,
            rand_pool,
            rand_idx,
            buffer,
            spaces,
            lengths,
        } = self;

        let cols = lengths.len();
        let total = buffer.len();
        let lines = total / cols - 1;
        let lines_u16 = lines as u16;
        let mut next_u64 = || {
            if *rand_idx >= rand_pool.len() {
                for slot in rand_pool.iter_mut() {
                    *slot = rng.generate::<u64>();
                }
                *rand_idx = 0;
            }
            let value = rand_pool[*rand_idx];
            *rand_idx += 1;
            value
        };

        // Reproduce from https://github.com/abishekvashok/cmatrix/blob/eb2fd2a5fed63da49848ff5e9b2d1b1d5e2ecd81/cmatrix.c#L642
        for j in (0..cols).step_by(2) {
            let idx_0j = j;
            let idx_1j = cols + j;

            if buffer[idx_0j].val == EMPTY && buffer[idx_1j].val == BLANK && spaces[j] > 0 {
                spaces[j] -= 1;
            } else if buffer[idx_0j].val == EMPTY && buffer[idx_1j].val == BLANK {
                let length_upper = lines_u16.saturating_sub(3).max(1);
                let length_part = (next_u64() % length_upper as u64) as u16;
                let space_upper = lines_u16.max(1);
                let space_part = (next_u64() % space_upper as u64) as u16;
                let ch = CH_BEGIN + (next_u64() % (CH_END - CH_BEGIN) as u64) as u8;

                lengths[j] = 3 + length_part;
                buffer[idx_0j].val = ch;
                spaces[j] = 1 + space_part;
            }

            let mut i = 0usize;
            let mut first_done = false;
            while i <= lines {
                let mut idx_ij = i * cols + j;

                while i <= lines && (buffer[idx_ij].val == BLANK || buffer[idx_ij].val == EMPTY) {
                    i += 1;
                    if i <= lines {
                        idx_ij += cols;
                    }
                }
                if i > lines {
                    break;
                }

                let z_idx = idx_ij;
                let mut y = 0u16;
                while i <= lines && (buffer[idx_ij].val != BLANK && buffer[idx_ij].val != EMPTY) {
                    buffer[idx_ij].clear_head();
                    i += 1;
                    y += 1;
                    if i <= lines {
                        idx_ij += cols;
                    }
                }

                if i > lines {
                    buffer[z_idx].val = BLANK;
                    buffer[z_idx].clear_head();
                    continue;
                }

                buffer[idx_ij].val = CH_BEGIN + (next_u64() % (CH_END - CH_BEGIN) as u64) as u8;
                buffer[idx_ij].set_head();
                if y > lengths[j] || first_done {
                    buffer[z_idx].val = BLANK;
                    buffer[z_idx].clear_head();
                    buffer[idx_0j].val = EMPTY;
                    buffer[idx_0j].clear_head();
                }
                first_done = true;
                i += 1;
            }
        }
        &buffer[..total - cols]
    }
}

#[cfg(test)]
mod tests {
    use super::Item;
    use std::mem::size_of;

    #[test]
    fn item_size_is_compact() {
        assert!(size_of::<Item>() <= 2);
    }
}
