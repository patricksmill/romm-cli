/// Cache key for ROMs: one entry per console or collection.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RomCacheKey {
    Platform(u64),
    Collection(u64),
}
