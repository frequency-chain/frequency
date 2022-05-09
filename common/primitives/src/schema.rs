use frame_support::BoundedVec;

pub type SchemaId = u16;

pub type Schema<T> = BoundedVec<u8, T>;
