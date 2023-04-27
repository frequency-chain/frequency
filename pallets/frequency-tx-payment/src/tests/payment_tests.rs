use super::mock::{ExtBuilder, Test};

use crate::InitialPayment;

#[test]
fn test_initial_payment_is_capacity() {
	ExtBuilder::default().build().execute_with(|| {
		assert!(InitialPayment::Capacity::<Test>.is_capacity());
		assert!(!InitialPayment::Capacity::<Test>.is_free());
		assert!(!InitialPayment::Capacity::<Test>.is_token());
	});
}

#[test]
fn test_initial_payment_is_token() {
	ExtBuilder::default().build().execute_with(|| {
		assert!(!InitialPayment::Token::<Test>(Default::default()).is_capacity());
		assert!(!InitialPayment::Token::<Test>(Default::default()).is_free());
		assert!(InitialPayment::Token::<Test>(Default::default()).is_token());
	});
}

#[test]
fn test_initial_payment_is_free() {
	ExtBuilder::default().build().execute_with(|| {
		assert!(!InitialPayment::Free::<Test>.is_capacity());
		assert!(InitialPayment::Free::<Test>.is_free());
		assert!(!InitialPayment::Free::<Test>.is_token());
	});
}
