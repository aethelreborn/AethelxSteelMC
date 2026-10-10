//! Skull and head block implementations.
//!
//! Mirrors vanilla `SkullBlock` / `WallSkullBlock` plus the variant classes
//! `WitherSkullBlock`, `WitherWallSkullBlock`, `PlayerHeadBlock`,
//! `PlayerWallHeadBlock`, and `PiglinWallSkullBlock`. All variants share the
//! `skull` block entity, which stores the owner profile.
//!
//! Known gap vs vanilla: placing a wither skeleton skull does not yet run the
//! wither-summon block pattern. That behavior needs Vanilla's `BlockPattern`
//! infrastructure and a Wither entity, neither of which Steel has yet. Every
//! other skull behavior (placement, survival, drops, profile copy) is
//! implemented.

use std::sync::{Arc, Weak};

use steel_macros::block_behavior;
use steel_registry::REGISTRY;
use steel_registry::blocks::BlockRef;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::blocks::properties::{
    BlockStateProperties, Direction, EnumProperty, IntProperty,
};
use steel_registry::blocks::shapes::SupportType;
use steel_registry::data_components::vanilla_components::PROFILE;
use steel_registry::item_stack::ItemStack;
use steel_registry::vanilla_blocks;
use steel_utils::{BlockPos, BlockStateId, Downcast as _};

use crate::behavior::block::{BlockBehavior, BlockEntityCreation};
use crate::behavior::context::{BlockPlaceContext, PlacementSource};
use crate::block_entity::entities::SkullBlockEntity;
use crate::world::{LevelReader, ScheduledTickAccess, World};

const ROTATION_16: &IntProperty = &BlockStateProperties::ROTATION_16;
const HORIZONTAL_FACING: &EnumProperty<Direction> = &BlockStateProperties::HORIZONTAL_FACING;

/// Computes the standing skull `rotation` property from the placer's yaw.
///
/// Vanilla `SkullBlock.getStateForPlacement`:
/// `Mth.floor((context.getRotation() * 16F / 360F) + 0.5F) & 15`.
fn rotation_segment_from_yaw(yaw: f32) -> u8 {
    ((((yaw.rem_euclid(360.0)) / (360.0 / 16.0)) + 0.5) as u8) & 15
}

/// Returns whether a standing skull has support below.
///
/// Vanilla checks rigid center support from the block below
/// (`canSupportRigidBlock`).
fn can_survive_floor(world: &dyn LevelReader, pos: BlockPos) -> bool {
    let below_pos = pos.below();
    let below_state = world.get_block_state(below_pos);
    world.is_face_sturdy_for(below_state, below_pos, Direction::Up, SupportType::Rigid)
}

/// Returns the state for a standing skull after a neighbor update.
fn floor_update_shape(
    state: BlockStateId,
    world: &dyn ScheduledTickAccess,
    pos: BlockPos,
    direction: Direction,
) -> BlockStateId {
    if direction == Direction::Down && !can_survive_floor(world, pos) {
        return REGISTRY.blocks.get_default_state_id(&vanilla_blocks::AIR);
    }
    state
}

/// Returns whether a wall skull has a sturdy face to attach to behind it.
fn can_survive_wall(state: BlockStateId, world: &dyn LevelReader, pos: BlockPos) -> bool {
    let facing: Direction = state.get_value(HORIZONTAL_FACING);
    let attach_pos = facing.opposite().relative(pos);
    let attach_state = world.get_block_state(attach_pos);
    world.is_face_sturdy(attach_state, attach_pos, facing)
}

/// Returns the state for a wall skull after a neighbor update.
fn wall_update_shape(
    state: BlockStateId,
    world: &dyn ScheduledTickAccess,
    pos: BlockPos,
    direction: Direction,
) -> BlockStateId {
    let facing: Direction = state.get_value(HORIZONTAL_FACING);
    if direction == facing.opposite() && !can_survive_wall(state, world, pos) {
        return REGISTRY.blocks.get_default_state_id(&vanilla_blocks::AIR);
    }
    state
}

/// Copies the placed stack's profile component into the skull block entity.
fn copy_profile_from_placed_stack(
    world: &Arc<World>,
    pos: BlockPos,
    source: &PlacementSource<'_>,
) {
    let Some(entity) = world.get_block_entity(pos) else {
        return;
    };
    let Some(skull) = entity.downcast_ref::<SkullBlockEntity>() else {
        return;
    };
    let profile = source.with_item(|stack| stack.get(PROFILE).cloned());
    skull.set_owner(profile);
}

/// Returns the pick-block stack for a skull, copying the stored profile.
fn clone_item_stack_with_profile(
    world: &Arc<World>,
    pos: BlockPos,
    block: BlockRef,
) -> Option<ItemStack> {
    let mut stack = ItemStack::new(REGISTRY.items.by_block(block));
    if let Some(entity) = world.get_block_entity(pos)
        && let Some(skull) = entity.downcast_ref::<SkullBlockEntity>()
        && let Some(profile) = skull.owner()
    {
        stack.set(PROFILE, profile);
    }
    Some(stack)
}

/// Behavior for standing skull blocks (`skeleton_skull`, `zombie_head`,
/// `creeper_head`, `dragon_head`, `piglin_head`).
#[block_behavior]
pub struct SkullBlock {
    block: BlockRef,
}

impl SkullBlock {
    /// Creates a new standing skull block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for SkullBlock {
    fn can_survive(&self, _state: BlockStateId, world: &dyn LevelReader, pos: BlockPos) -> bool {
        can_survive_floor(world, pos)
    }

    fn update_shape(
        &self,
        state: BlockStateId,
        world: &dyn ScheduledTickAccess,
        pos: BlockPos,
        direction: Direction,
        _neighbor_pos: BlockPos,
        _neighbor_state: BlockStateId,
    ) -> BlockStateId {
        floor_update_shape(state, world, pos, direction)
    }

    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        let rotation = rotation_segment_from_yaw(context.rotation());
        Some(self.block.default_state().set_value(ROTATION_16, rotation))
    }

    fn new_block_entity(
        &self,
        level: Weak<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) -> BlockEntityCreation {
        BlockEntityCreation::Created(Arc::new(SkullBlockEntity::new(level, pos, state)))
    }

    fn set_placed_by(
        &self,
        _state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        source: &PlacementSource<'_>,
    ) {
        copy_profile_from_placed_stack(world, pos, source);
    }

    fn get_clone_item_stack_with_data(
        &self,
        world: &Arc<World>,
        pos: BlockPos,
        block: BlockRef,
        _state: BlockStateId,
        _include_data: bool,
    ) -> Option<ItemStack> {
        clone_item_stack_with_profile(world, pos, block)
    }
}

/// Behavior for wall skull blocks (`skeleton_wall_skull`, `zombie_wall_head`,
/// `creeper_wall_head`, `dragon_wall_head`).
#[block_behavior]
pub struct WallSkullBlock {
    block: BlockRef,
}

impl WallSkullBlock {
    /// Creates a new wall skull block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for WallSkullBlock {
    fn can_survive(&self, state: BlockStateId, world: &dyn LevelReader, pos: BlockPos) -> bool {
        can_survive_wall(state, world, pos)
    }

    fn update_shape(
        &self,
        state: BlockStateId,
        world: &dyn ScheduledTickAccess,
        pos: BlockPos,
        direction: Direction,
        _neighbor_pos: BlockPos,
        _neighbor_state: BlockStateId,
    ) -> BlockStateId {
        wall_update_shape(state, world, pos, direction)
    }

    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        let clicked_face = context.clicked_face();
        if !clicked_face.is_horizontal() {
            return None;
        }
        let state = self
            .block
            .default_state()
            .set_value(HORIZONTAL_FACING, clicked_face.opposite());
        Some(state)
    }

    fn new_block_entity(
        &self,
        level: Weak<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) -> BlockEntityCreation {
        BlockEntityCreation::Created(Arc::new(SkullBlockEntity::new(level, pos, state)))
    }

    fn set_placed_by(
        &self,
        _state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        source: &PlacementSource<'_>,
    ) {
        copy_profile_from_placed_stack(world, pos, source);
    }

    fn get_clone_item_stack_with_data(
        &self,
        world: &Arc<World>,
        pos: BlockPos,
        block: BlockRef,
        _state: BlockStateId,
        _include_data: bool,
    ) -> Option<ItemStack> {
        clone_item_stack_with_profile(world, pos, block)
    }
}

/// Behavior for standing wither skeleton skulls.
///
/// Placement does not yet attempt the wither-summon block pattern; see the
/// module documentation for the missing foundations.
#[block_behavior]
pub struct WitherSkullBlock {
    block: BlockRef,
}

impl WitherSkullBlock {
    /// Creates a new standing wither skeleton skull behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for WitherSkullBlock {
    fn can_survive(&self, _state: BlockStateId, world: &dyn LevelReader, pos: BlockPos) -> bool {
        can_survive_floor(world, pos)
    }

    fn update_shape(
        &self,
        state: BlockStateId,
        world: &dyn ScheduledTickAccess,
        pos: BlockPos,
        direction: Direction,
        _neighbor_pos: BlockPos,
        _neighbor_state: BlockStateId,
    ) -> BlockStateId {
        floor_update_shape(state, world, pos, direction)
    }

    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        let rotation = rotation_segment_from_yaw(context.rotation());
        Some(self.block.default_state().set_value(ROTATION_16, rotation))
    }

    fn new_block_entity(
        &self,
        level: Weak<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) -> BlockEntityCreation {
        BlockEntityCreation::Created(Arc::new(SkullBlockEntity::new(level, pos, state)))
    }

    fn set_placed_by(
        &self,
        _state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        source: &PlacementSource<'_>,
    ) {
        copy_profile_from_placed_stack(world, pos, source);
    }

    fn get_clone_item_stack_with_data(
        &self,
        world: &Arc<World>,
        pos: BlockPos,
        block: BlockRef,
        _state: BlockStateId,
        _include_data: bool,
    ) -> Option<ItemStack> {
        clone_item_stack_with_profile(world, pos, block)
    }
}

/// Behavior for wall wither skeleton skulls.
///
/// Placement does not yet attempt the wither-summon block pattern; see the
/// module documentation for the missing foundations.
#[block_behavior]
pub struct WitherWallSkullBlock {
    block: BlockRef,
}

impl WitherWallSkullBlock {
    /// Creates a new wall wither skeleton skull behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for WitherWallSkullBlock {
    fn can_survive(&self, state: BlockStateId, world: &dyn LevelReader, pos: BlockPos) -> bool {
        can_survive_wall(state, world, pos)
    }

    fn update_shape(
        &self,
        state: BlockStateId,
        world: &dyn ScheduledTickAccess,
        pos: BlockPos,
        direction: Direction,
        _neighbor_pos: BlockPos,
        _neighbor_state: BlockStateId,
    ) -> BlockStateId {
        wall_update_shape(state, world, pos, direction)
    }

    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        let clicked_face = context.clicked_face();
        if !clicked_face.is_horizontal() {
            return None;
        }
        let state = self
            .block
            .default_state()
            .set_value(HORIZONTAL_FACING, clicked_face.opposite());
        Some(state)
    }

    fn new_block_entity(
        &self,
        level: Weak<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) -> BlockEntityCreation {
        BlockEntityCreation::Created(Arc::new(SkullBlockEntity::new(level, pos, state)))
    }

    fn set_placed_by(
        &self,
        _state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        source: &PlacementSource<'_>,
    ) {
        copy_profile_from_placed_stack(world, pos, source);
    }

    fn get_clone_item_stack_with_data(
        &self,
        world: &Arc<World>,
        pos: BlockPos,
        block: BlockRef,
        _state: BlockStateId,
        _include_data: bool,
    ) -> Option<ItemStack> {
        clone_item_stack_with_profile(world, pos, block)
    }
}

/// Behavior for standing player heads.
#[block_behavior]
pub struct PlayerHeadBlock {
    block: BlockRef,
}

impl PlayerHeadBlock {
    /// Creates a new standing player head behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for PlayerHeadBlock {
    fn can_survive(&self, _state: BlockStateId, world: &dyn LevelReader, pos: BlockPos) -> bool {
        can_survive_floor(world, pos)
    }

    fn update_shape(
        &self,
        state: BlockStateId,
        world: &dyn ScheduledTickAccess,
        pos: BlockPos,
        direction: Direction,
        _neighbor_pos: BlockPos,
        _neighbor_state: BlockStateId,
    ) -> BlockStateId {
        floor_update_shape(state, world, pos, direction)
    }

    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        let rotation = rotation_segment_from_yaw(context.rotation());
        Some(self.block.default_state().set_value(ROTATION_16, rotation))
    }

    fn new_block_entity(
        &self,
        level: Weak<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) -> BlockEntityCreation {
        BlockEntityCreation::Created(Arc::new(SkullBlockEntity::new(level, pos, state)))
    }

    fn set_placed_by(
        &self,
        _state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        source: &PlacementSource<'_>,
    ) {
        copy_profile_from_placed_stack(world, pos, source);
    }

    fn get_clone_item_stack_with_data(
        &self,
        world: &Arc<World>,
        pos: BlockPos,
        block: BlockRef,
        _state: BlockStateId,
        _include_data: bool,
    ) -> Option<ItemStack> {
        clone_item_stack_with_profile(world, pos, block)
    }
}

/// Behavior for wall player heads.
#[block_behavior]
pub struct PlayerWallHeadBlock {
    block: BlockRef,
}

impl PlayerWallHeadBlock {
    /// Creates a new wall player head behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for PlayerWallHeadBlock {
    fn can_survive(&self, state: BlockStateId, world: &dyn LevelReader, pos: BlockPos) -> bool {
        can_survive_wall(state, world, pos)
    }

    fn update_shape(
        &self,
        state: BlockStateId,
        world: &dyn ScheduledTickAccess,
        pos: BlockPos,
        direction: Direction,
        _neighbor_pos: BlockPos,
        _neighbor_state: BlockStateId,
    ) -> BlockStateId {
        wall_update_shape(state, world, pos, direction)
    }

    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        let clicked_face = context.clicked_face();
        if !clicked_face.is_horizontal() {
            return None;
        }
        let state = self
            .block
            .default_state()
            .set_value(HORIZONTAL_FACING, clicked_face.opposite());
        Some(state)
    }

    fn new_block_entity(
        &self,
        level: Weak<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) -> BlockEntityCreation {
        BlockEntityCreation::Created(Arc::new(SkullBlockEntity::new(level, pos, state)))
    }

    fn set_placed_by(
        &self,
        _state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        source: &PlacementSource<'_>,
    ) {
        copy_profile_from_placed_stack(world, pos, source);
    }

    fn get_clone_item_stack_with_data(
        &self,
        world: &Arc<World>,
        pos: BlockPos,
        block: BlockRef,
        _state: BlockStateId,
        _include_data: bool,
    ) -> Option<ItemStack> {
        clone_item_stack_with_profile(world, pos, block)
    }
}

/// Behavior for wall piglin heads.
///
/// Piglin avoidance of piglin heads is entity AI (`PiglinAi`) rather than
/// block behavior; only the block mechanics live here.
#[block_behavior]
pub struct PiglinWallSkullBlock {
    block: BlockRef,
}

impl PiglinWallSkullBlock {
    /// Creates a new wall piglin head behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for PiglinWallSkullBlock {
    fn can_survive(&self, state: BlockStateId, world: &dyn LevelReader, pos: BlockPos) -> bool {
        can_survive_wall(state, world, pos)
    }

    fn update_shape(
        &self,
        state: BlockStateId,
        world: &dyn ScheduledTickAccess,
        pos: BlockPos,
        direction: Direction,
        _neighbor_pos: BlockPos,
        _neighbor_state: BlockStateId,
    ) -> BlockStateId {
        wall_update_shape(state, world, pos, direction)
    }

    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        let clicked_face = context.clicked_face();
        if !clicked_face.is_horizontal() {
            return None;
        }
        let state = self
            .block
            .default_state()
            .set_value(HORIZONTAL_FACING, clicked_face.opposite());
        Some(state)
    }

    fn new_block_entity(
        &self,
        level: Weak<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) -> BlockEntityCreation {
        BlockEntityCreation::Created(Arc::new(SkullBlockEntity::new(level, pos, state)))
    }

    fn set_placed_by(
        &self,
        _state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        source: &PlacementSource<'_>,
    ) {
        copy_profile_from_placed_stack(world, pos, source);
    }

    fn get_clone_item_stack_with_data(
        &self,
        world: &Arc<World>,
        pos: BlockPos,
        block: BlockRef,
        _state: BlockStateId,
        _include_data: bool,
    ) -> Option<ItemStack> {
        clone_item_stack_with_profile(world, pos, block)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use glam::DVec3;
    use simdnbt::borrow::read_compound;
    use simdnbt::owned::NbtCompound;
    use steel_registry::init_vanilla_registry;
    use steel_registry::resolvable_profile::{PlayerSkinPatch, ResolvableProfile};
    use steel_registry::vanilla_items;
    use steel_utils::ChunkPos;
    use steel_utils::types::{InteractionHand, UpdateFlags};

    use super::*;
    use crate::behavior::{
        BlockHitResult, BlockItem, InteractionResult, PlacementOrientation, PlacementSource,
    };
    use crate::block_entity::BlockEntity as _;
    use crate::test_support::{TestLevel, fresh_test_world, insert_ready_full_chunk};

    fn floor_skull() -> SkullBlock {
        SkullBlock::new(&vanilla_blocks::SKELETON_SKULL)
    }

    fn wall_skull() -> WallSkullBlock {
        WallSkullBlock::new(&vanilla_blocks::SKELETON_WALL_SKULL)
    }

    #[test]
    fn floor_skull_pops_when_the_block_below_is_removed() {
        init_vanilla_registry();
        let pos = BlockPos::new(0, 64, 0);
        let state = vanilla_blocks::SKELETON_SKULL.default_state();
        let skull = floor_skull();

        let supported =
            TestLevel::default().with_block(pos.below(), vanilla_blocks::STONE.default_state());
        assert_eq!(
            skull.update_shape(
                state,
                &supported,
                pos,
                Direction::Down,
                pos.below(),
                vanilla_blocks::STONE.default_state(),
            ),
            state
        );

        let unsupported = TestLevel::default();
        assert!(
            skull
                .update_shape(
                    state,
                    &unsupported,
                    pos,
                    Direction::Down,
                    pos.below(),
                    vanilla_blocks::AIR.default_state(),
                )
                .is_air()
        );
    }

    #[test]
    fn wall_skull_pops_only_when_its_attached_face_is_removed() {
        init_vanilla_registry();
        let pos = BlockPos::new(0, 64, 0);
        let state = vanilla_blocks::SKELETON_WALL_SKULL
            .default_state()
            .set_value(HORIZONTAL_FACING, Direction::North);
        let skull = wall_skull();
        let attach_pos = pos.south();

        let supported =
            TestLevel::default().with_block(attach_pos, vanilla_blocks::STONE.default_state());
        assert_eq!(
            skull.update_shape(
                state,
                &supported,
                pos,
                Direction::South,
                attach_pos,
                vanilla_blocks::STONE.default_state(),
            ),
            state
        );

        let unsupported = TestLevel::default();
        assert!(
            skull
                .update_shape(
                    state,
                    &unsupported,
                    pos,
                    Direction::South,
                    attach_pos,
                    vanilla_blocks::AIR.default_state(),
                )
                .is_air()
        );
        assert_eq!(
            skull.update_shape(
                state,
                &unsupported,
                pos,
                Direction::Up,
                pos.above(),
                vanilla_blocks::AIR.default_state(),
            ),
            state
        );
    }

    #[test]
    fn skulls_require_support_to_survive() {
        init_vanilla_registry();
        let pos = BlockPos::new(0, 64, 0);
        let floor = vanilla_blocks::SKELETON_SKULL.default_state();
        let wall_state = vanilla_blocks::SKELETON_WALL_SKULL
            .default_state()
            .set_value(HORIZONTAL_FACING, Direction::North);

        let supported_floor =
            TestLevel::default().with_block(pos.below(), vanilla_blocks::STONE.default_state());
        assert!(floor_skull().can_survive(floor, &supported_floor, pos));
        assert!(!floor_skull().can_survive(floor, &TestLevel::default(), pos));

        let supported_wall =
            TestLevel::default().with_block(pos.south(), vanilla_blocks::STONE.default_state());
        assert!(wall_skull().can_survive(wall_state, &supported_wall, pos));
        assert!(!wall_skull().can_survive(wall_state, &TestLevel::default(), pos));
    }

    #[test]
    fn floor_skull_placement_rotation_matches_vanilla_yaw_segments() {
        init_vanilla_registry();
        let world_fixture = fresh_test_world("skull_rotation");
        let world = &world_fixture.world;
        let support = BlockPos::new(0, 64, 0);
        let _holder = insert_ready_full_chunk(world, ChunkPos::from_block_pos(support));

        let skull = floor_skull();
        for (yaw, expected) in [(0.0, 0), (90.0, 4), (-90.0, 12), (180.0, 8)] {
            let mut stack = ItemStack::new(&vanilla_items::SKELETON_SKULL);
            let source = PlacementSource::direct(
                None,
                InteractionHand::MainHand,
                &mut stack,
                PlacementOrientation::Player {
                    rotation: yaw,
                    pitch: 0.0,
                },
                false,
            );
            let context = BlockPlaceContext::new(
                world,
                source,
                &BlockHitResult {
                    location: DVec3::new(0.5, 64.5, 0.5),
                    direction: Direction::Up,
                    block_pos: support,
                    miss: false,
                    inside: false,
                    world_border_hit: false,
                },
            );
            let state = skull
                .get_state_for_placement(&context)
                .expect("floor skull placement should produce a state");
            assert_eq!(state.get_value(ROTATION_16), expected, "yaw {yaw}");
        }
    }

    #[test]
    fn wall_skull_placement_faces_the_clicked_wall_side() {
        init_vanilla_registry();
        let world_fixture = fresh_test_world("skull_wall_facing");
        let world = &world_fixture.world;
        let support = BlockPos::new(0, 64, 0);
        let _holder = insert_ready_full_chunk(world, ChunkPos::from_block_pos(support));

        let skull = wall_skull();
        for clicked_face in [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ] {
            let mut stack = ItemStack::new(&vanilla_items::SKELETON_WALL_SKULL);
            let source = PlacementSource::direct(
                None,
                InteractionHand::MainHand,
                &mut stack,
                PlacementOrientation::Directional {
                    direction: clicked_face,
                },
                false,
            );
            let context = BlockPlaceContext::new(
                world,
                source,
                &BlockHitResult {
                    location: DVec3::new(0.5, 64.5, 0.5),
                    direction: clicked_face,
                    block_pos: support,
                    miss: false,
                    inside: false,
                    world_border_hit: false,
                },
            );
            let state = skull
                .get_state_for_placement(&context)
                .expect("wall skull placement on a horizontal face should produce a state");
            assert_eq!(
                state.get_value(HORIZONTAL_FACING),
                clicked_face.opposite(),
                "clicked face {clicked_face:?}"
            );
        }

        let mut stack = ItemStack::new(&vanilla_items::SKELETON_WALL_SKULL);
        let source = PlacementSource::direct(
            None,
            InteractionHand::MainHand,
            &mut stack,
            PlacementOrientation::Directional {
                direction: Direction::Up,
            },
            false,
        );
        let context = BlockPlaceContext::new(
            world,
            source,
            &BlockHitResult {
                location: DVec3::new(0.5, 64.5, 0.5),
                direction: Direction::Up,
                block_pos: support,
                miss: false,
                inside: false,
                world_border_hit: false,
            },
        );
        assert!(skull.get_state_for_placement(&context).is_none());
    }

    #[test]
    fn placed_head_stores_the_profile_and_round_trips_nbt() {
        init_vanilla_registry();
        let world_fixture = fresh_test_world("skull_profile");
        let world = &world_fixture.world;
        let support = BlockPos::new(0, 64, 0);
        let pos = support.above();
        let _holder = insert_ready_full_chunk(world, ChunkPos::from_block_pos(pos));
        assert!(world.set_block(
            support,
            vanilla_blocks::STONE.default_state(),
            UpdateFlags::UPDATE_NONE,
        ));

        let profile =
            ResolvableProfile::dynamic_name("Notch".to_owned(), PlayerSkinPatch::default())
                .expect("Notch is a valid profile name");
        let mut stack = ItemStack::new(&vanilla_items::PLAYER_HEAD);
        stack.set(PROFILE, profile.clone());

        let source = PlacementSource::direct(
            None,
            InteractionHand::MainHand,
            &mut stack,
            PlacementOrientation::Player {
                rotation: 0.0,
                pitch: 0.0,
            },
            false,
        );
        let context = BlockPlaceContext::new(
            world,
            source,
            &BlockHitResult {
                location: DVec3::new(0.5, 64.5, 0.5),
                direction: Direction::Up,
                block_pos: support,
                miss: false,
                inside: false,
                world_border_hit: false,
            },
        );
        assert_eq!(
            BlockItem::new(&vanilla_blocks::PLAYER_HEAD).place(context),
            InteractionResult::Success,
        );
        assert!(stack.is_empty());

        let entity = world
            .get_block_entity(pos)
            .expect("placed head creates its block entity");
        let skull = entity
            .downcast_ref::<SkullBlockEntity>()
            .expect("placed head block entity is a skull entity");
        assert_eq!(skull.owner(), Some(profile.clone()));

        let picked = PlayerHeadBlock::new(&vanilla_blocks::PLAYER_HEAD)
            .get_clone_item_stack_with_data(
                world,
                pos,
                &vanilla_blocks::PLAYER_HEAD,
                vanilla_blocks::PLAYER_HEAD.default_state(),
                false,
            )
            .expect("player head always has a clone stack");
        assert_eq!(picked.get(PROFILE), Some(&profile));

        let mut saved = NbtCompound::new();
        skull.save_additional(&mut saved);
        let mut bytes = Vec::new();
        saved.write(&mut bytes);
        let borrowed = read_compound(&mut Cursor::new(bytes.as_slice()))
            .expect("saved skull NBT should reborrow");
        let restored = SkullBlockEntity::new(
            Weak::new(),
            pos,
            vanilla_blocks::PLAYER_HEAD.default_state(),
        );
        restored.load_additional(&borrowed);
        assert_eq!(restored.owner(), Some(profile));
    }
}
