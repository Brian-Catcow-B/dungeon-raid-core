pub struct CollectionMultipliers {
    pub shield_collection_multiplier: usize,
    pub coin_collection_multiplier: usize,
    pub weapon_collection_multiplier: usize,
}

impl Default for CollectionMultipliers {
    fn default() -> Self {
        Self {
            shield_collection_multiplier: 1,
            coin_collection_multiplier: 1,
            weapon_collection_multiplier: 1,
        }
    }
}
