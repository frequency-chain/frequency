//! Benchmarking setup for handles pallet
use super::*;

#[allow(unused)]
use crate::Pallet as Template;
use frame_benchmarking::benchmarks;

benchmarks! {

impl_benchmark_test_suite!(Handles,
	crate::tests::mock::new_test_ext(),
	crate::tests::mock::Test);
}
