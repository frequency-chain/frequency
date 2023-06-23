// Kilt
impl pallet_did_lookup::Config for Runtime {
	type Currency = Balances;
	type Deposit = ConstU128<UNIT>;
	type DidIdentifier = DidIdentifier;
	type EnsureOrigin = EnsureDipOrigin<
		DidIdentifier,
		AccountId,
		VerificationResult<KeyIdOf<Runtime>, BlockNumber, Web3Name, LinkableAccountId, 10, 10>,
	>;
	type OriginSuccess = DipOrigin<
		DidIdentifier,
		AccountId,
		VerificationResult<KeyIdOf<Runtime>, BlockNumber, Web3Name, LinkableAccountId, 10, 10>,
	>;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
}