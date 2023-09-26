use crate::random::RandomGenerator;

#[test]
fn test_random_generator() {
    let mut rng = RandomGenerator::with_nonce(0);
    for n in [17, 1900000000] {
        let n_f64 = n as f64;
        let mean = (n_f64 - 1.0) / 2.0;
        let variance = (n_f64 * n_f64 - 1.0) / 12.0;
        let num_iters: u32 = 1000000;
        let max_error = 4.0 * (num_iters as f64 * variance).sqrt();
        let mut total: u64 = 0;
        for _ in 0..num_iters {
            let a = rng.uniform(n);
            assert!((0..n).contains(&a));
            total += u64::from(a);
        }
        assert!((total as f64 - num_iters as f64 * mean).abs() < max_error);
    }
}
