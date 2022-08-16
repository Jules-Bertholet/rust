#[derive(PartialEq, Eq)]
#[non_exhaustive]
pub enum NonExhaustiveZstEnum {
    Variant(),
}

pub static NON_EXHAUSTIVE_ZST_ENUM: NonExhaustiveZstEnum = NonExhaustiveZstEnum::Variant();

#[derive(PartialEq, Eq)]
#[non_exhaustive]
pub struct NonExhaustiveZstStruct();

pub static NON_EXHAUSTIVE_ZST_STRUCT: NonExhaustiveZstStruct = NonExhaustiveZstStruct();

#[derive(PartialEq, Eq)]
pub enum NonExhaustiveVariantZstEnum {
    #[non_exhaustive]
    Variant(),
}

pub static NON_EXHAUSTIVE_VARIANT_ZST_ENUM: NonExhaustiveVariantZstEnum =
    NonExhaustiveVariantZstEnum::Variant();
