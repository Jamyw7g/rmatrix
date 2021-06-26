use rand::{prelude::ThreadRng, Rng};

pub const CH_BEGIN: i32 = 33;
pub const CH_END: i32 = 127;
pub const BLANK: i32 = b' ' as i32;

#[derive(Clone, Debug)]
pub struct Item {
    pub val: i32,
    pub is_head: bool,
}

impl Default for Item {
    fn default() -> Self {
        Item {
            val: -1,
            is_head: false,
        }
    }
}

pub struct Matrix {
    rng: ThreadRng,

    buffer: Vec<Item>,
    spaces: Vec<u16>,
    lengths: Vec<u16>,
}

impl Matrix {
    pub fn new(cols: usize, rows: usize) -> Self {
        let mut rng = rand::thread_rng();
        let mut buffer = vec![Item::default(); (rows + 1) * cols];
        let mut spaces = vec![0; cols];
        let mut lengths = vec![0; cols];

        let lines = rows as u16;
        for j in (0..cols).step_by(2) {
            buffer[to_idx(1, j, cols)].val = BLANK;
            lengths[j] = 3 + rng.gen_range(0..lines as u16 - 3);
            spaces[j] = 1 + rng.gen_range(0..lines as u16);
        }

        Self {
            rng,
            buffer,
            spaces,
            lengths,
        }
    }

    pub fn next(&mut self) -> &[Item] {
        // Pattern match for convenience
        let Self {
            rng,
            buffer,
            spaces,
            lengths,
        } = self;

        let cols = lengths.len();
        let total = buffer.len();
        let lines = total / cols - 1;

        let (mut idx_0j, mut idx_1j, mut idx_ij, mut i, mut y, mut z, mut first_done);

        // Reproduce from https://github.com/abishekvashok/cmatrix/blob/eb2fd2a5fed63da49848ff5e9b2d1b1d5e2ecd81/cmatrix.c#L642
        for j in (0..cols).step_by(2) {
            idx_0j = to_idx(0, j, cols);
            idx_1j = to_idx(1, j, cols);
            if buffer[idx_0j].val == -1 && buffer[idx_1j].val == BLANK && spaces[j] > 0 {
                spaces[j] -= 1;
            } else if buffer[idx_0j].val == -1 && buffer[idx_1j].val == BLANK {
                lengths[j] = 3 + rng.gen_range(0..lines as u16 - 3);
                buffer[idx_0j].val = rng.gen_range(CH_BEGIN..CH_END);
                spaces[j] = 1 + rng.gen_range(0..lines as u16);
            }
            i = 0;
            first_done = false;
            while i <= lines {
                idx_ij = to_idx(i, j, cols);
                while i <= lines && (buffer[idx_ij].val == BLANK || buffer[idx_ij].val == -1) {
                    i += 1;
                    idx_ij = to_idx(i, j, cols);
                }
                if i > lines {
                    break;
                }
                z = i;
                y = 0;
                idx_ij = to_idx(i, j, cols);
                while i <= lines && (buffer[idx_ij].val != BLANK && buffer[idx_ij].val != -1) {
                    buffer[idx_ij].is_head = false;
                    i += 1;
                    y += 1;
                    idx_ij = to_idx(i, j, cols);
                }
                if i > lines {
                    buffer[to_idx(z, j, cols)].val = BLANK;
                    continue;
                }
                buffer[to_idx(i, j, cols)] = Item {
                    val: rng.gen_range(CH_BEGIN..CH_END),
                    is_head: true,
                };
                if y > lengths[j] || first_done {
                    buffer[to_idx(z, j, cols)].val = BLANK;
                    buffer[to_idx(0, j, cols)].val = -1;
                }
                first_done = true;
                i += 1;
            }
        }
        &buffer[..total - cols]
    }
}

#[inline]
fn to_idx(row: usize, col: usize, len: usize) -> usize {
    row * len + col
}
