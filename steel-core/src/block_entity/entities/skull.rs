//! Skull block entity implementation.
//!
//! Stores the owner profile rendered on standing and wall skull/head blocks.
//! Vanilla parity: `SkullBlockEntity`.

use std::sync::Weak;

use simdnbt::borrow::{
    BaseNbtCompound as BorrowedNbtCompound, NbtCompound as BorrowedNbtCompoundView,
};
use simdnbt::owned::NbtCompound;
use simdnbt::{FromNbtTag as _, ToNbtTag as _};
use steel_registry::resolvable_profile::ResolvableProfile;
use steel_registry::vanilla_block_entity_types;
use steel_utils::{BlockPos, BlockStateId, DowncastType, DowncastTypeKey, locks::SyncMutex};

use crate::block_entity::{BlockEntity, BlockEntityBase};
use crate::world::World;

/// Block entity for every skull and head block.
///
/// Holds the optional owner profile from the `minecraft:profile` component so
/// skins render in-world and survive pick-block / loot copy-back.
pub struct SkullBlockEntity {
    base: BlockEntityBase,
    owner: SyncMutex<Option<ResolvableProfile>>,
}

// SAFETY: This key identifies Steel's shared skull implementation for every
// skull/head block entity registry entry.
unsafe impl DowncastType for SkullBlockEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:block_entity/skull");
}

impl SkullBlockEntity {
    /// Creates a new skull block entity.
    #[must_use]
    pub fn new(level: Weak<World>, pos: BlockPos, state: BlockStateId) -> Self {
        Self {
            base: BlockEntityBase::new(&vanilla_block_entity_types::SKULL, level, pos, state),
            owner: SyncMutex::new(None),
        }
    }

    /// Returns a copy of the stored owner profile.
    #[must_use]
    pub fn owner(&self) -> Option<ResolvableProfile> {
        self.owner.lock().clone()
    }

    /// Sets or clears the stored owner profile.
    pub fn set_owner(&self, owner: Option<ResolvableProfile>) {
        *self.owner.lock() = owner;
    }
}

impl BlockEntity for SkullBlockEntity {
    fn base(&self) -> &BlockEntityBase {
        &self.base
    }

    fn load_additional(&self, nbt: &BorrowedNbtCompound<'_>) {
        let nbt_view: BorrowedNbtCompoundView<'_, '_> = nbt.into();
        if let Some(tag) = nbt_view.get("profile") {
            *self.owner.lock() = ResolvableProfile::from_nbt_tag(tag.clone());
        }
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        if let Some(profile) = self.owner.lock().as_ref() {
            nbt.insert("profile", profile.clone().to_nbt_tag());
        }
    }

    fn get_update_tag(&self) -> Option<NbtCompound> {
        // Sync the owner profile so clients can render the skin.
        let mut nbt = NbtCompound::new();
        self.save_additional(&mut nbt);
        Some(nbt)
    }
}
