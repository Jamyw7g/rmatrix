#![feature(test)]
extern crate test;

#[cfg(test)]
mod tests {
    use test::Bencher;

    #[bench]
    fn nano_bench(b: &mut Bencher) {
        use nanorand::Rng;
        b.iter(|| {
            let mut rng = nanorand::WyRand::new();
            for _ in 0..u16::MAX {
                let _a = rng.generate_range(0..6usize);
                let _b = rng.generate_range(0..128i32);
            }
        })
    }

    #[bench]
    fn rand_bench(b: &mut Bencher) {
        use rand::Rng;
        b.iter(|| {
            let mut rng = rand::thread_rng();
            for _ in 0..u16::MAX {
                let _a = rng.gen_range(0..6usize);
                let _b = rng.gen_range(0..128i32);
            }
        })
    }
}
