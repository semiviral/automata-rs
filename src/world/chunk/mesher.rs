use super::{CHUNK_SIZE, CHUNK_SIZE_CUBED, CHUNK_SIZE_MASK, CHUNK_SIZE_SHIFT, CHUNK_SIZE_SQUARED};
use crate::{
    collections::Palette,
    render::mesh::{PackedVertex, QuadIndexes, QuadVertexes},
    world::block::{self, Block, BlockRegistry},
    DIRECTION,
};

const PACKED_VERTEX_BY_NORMAL_INDEX: [[i32; 4]; 6] = [
    [
        // East      z      y      x
        0b01_01_10_000001_000001_000001,
        0b01_01_10_000001_000000_000001,
        0b01_01_10_000000_000000_000001,
        0b01_01_10_000000_000001_000001,
    ],
    [
        // Up        z      y      x
        0b01_10_01_000001_000001_000000,
        0b01_10_01_000001_000001_000001,
        0b01_10_01_000000_000001_000001,
        0b01_10_01_000000_000001_000000,
    ],
    [
        // North     z      y      x
        0b10_01_01_000001_000001_000000,
        0b10_01_01_000001_000000_000000,
        0b10_01_01_000001_000000_000001,
        0b10_01_01_000001_000001_000001,
    ],
    [
        // West      z      y      x
        0b01_01_00_000000_000001_000000,
        0b01_01_00_000000_000000_000000,
        0b01_01_00_000001_000000_000000,
        0b01_01_00_000001_000001_000000,
    ],
    [
        // Down      z      y      x
        0b01_00_01_000001_000000_000001,
        0b01_00_01_000001_000000_000000,
        0b01_00_01_000000_000000_000000,
        0b01_00_01_000000_000000_000001,
    ],
    [
        // South     z      y      x
        0b00_01_01_000000_000001_000001,
        0b00_01_01_000000_000000_000001,
        0b00_01_01_000000_000000_000000,
        0b00_01_01_000000_000001_000000,
    ],
];

const INDEX_STEP_BY_NORMAL_INDEX: [i32; 6] = [
    1,
    CHUNK_SIZE_SQUARED,
    CHUNK_SIZE,
    -1,
    -CHUNK_SIZE_SQUARED,
    -CHUNK_SIZE,
];

pub fn generate_packed_mesh(
    block_registry: &BlockRegistry,
    blocks_palette: &mut Palette<Block>,
    neighbors: [Option<&Palette<Block>>; 6],
) {
    if blocks_palette.lookup_len() == 1
        && blocks_palette.get_lookup_value(0).id() == BlockRegistry::AIR_ID
    {
        // return empty mesh data
    }

    // TODO create a pooled list for these
    let mut indexes = Vec::<QuadIndexes<u32>>::new();
    let mut vertexes = Vec::<QuadVertexes<PackedVertex>>::new();
    let mut blocks = [Block::AIR; CHUNK_SIZE_CUBED as usize];
    let mut faces = [DIRECTION::empty(); CHUNK_SIZE_CUBED as usize];

    blocks_palette.copy_to_slice(&mut blocks);

    let mut index = -1;
    for y in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                index += 1;

                let block = blocks[index as usize];

                if block.id() == 0 {
                    continue;
                } else {
                    let is_transparent = block_registry
                        .get_block_attributes(block.id())
                        .contains(block::Attributes::TRANSPARENT);
                    let local_position =
                        x | (y << CHUNK_SIZE_SHIFT) | (z << CHUNK_SIZE_SHIFT << CHUNK_SIZE_SHIFT);

                    // Iterate once over all 6 faces of the block.
                    for normal_index in 0..6 {
                        // Face direction always exists on a single bit, so we can iterate
                        // directions by shifting with the normal index.
                        let face_direction =
                            DIRECTION::from_bits_truncate((1 << normal_index) as u8);

                        // If the current normal already has a face.
                        if faces[index as usize].contains(face_direction) {
                            continue;
                        }

                        let is_negative_normal = ((normal_index as i32) - 3) >= 0;
                        // Normal index constrained to represent the xyz axes.
                        let component_index = normal_index % 3;
                        let component_shift = CHUNK_SIZE_SHIFT * component_index;

                        // Axis index of the current normal direction.
                        let faced_axis_value =
                            (local_position >> component_shift) & CHUNK_SIZE_MASK;
                        // Indicates whether or not the face check is within the current chunk bounds.
                        let facing_neighbor = (!is_negative_normal
                            && (faced_axis_value == (CHUNK_SIZE - 1)))
                            || (is_negative_normal && (faced_axis_value == 0));

                        // Counts our successful traversals.
                        let mut traversals = 0;
                        for perpendicular_normal_index in 1..3 {
                            let traversal_normal_index =
                                (component_index + perpendicular_normal_index) % 3;
                            let traversal_normal_shift = CHUNK_SIZE_SHIFT * traversal_normal_index;
                            let traversal_normal_axis_value =
                                (local_position >> traversal_normal_shift) & CHUNK_SIZE_MASK;
                            // Amount to add to current index to 'traverse' our 1D array by 1 block in our current normal direction.
                            let traversal_index_step =
                                INDEX_STEP_BY_NORMAL_INDEX[traversal_normal_index as usize];
                            let mut traversal_index = index + (traversals * traversal_index_step);
                            let mut total_traversal_len = traversal_normal_axis_value + traversals;

                            while total_traversal_len < CHUNK_SIZE
                                && !faces[traversal_index as usize].contains(face_direction)
                                && blocks[traversal_index as usize].id() == block.id()
                            {
                                if facing_neighbor {
                                    // This block of code translates the local position to a local position in the neighbor
                                    // in the direction of our perpendicular normal index.
                                    let sign = if is_negative_normal { -1 } else { 1 };
                                    let component_mask = CHUNK_SIZE_MASK << component_shift;
                                    let traversal_local_position =
                                        local_position + (traversals << traversal_normal_shift);

                                    let neighbor_local_position = (!component_mask
                                        & traversal_local_position)
                                        | (wrap(
                                            ((traversal_local_position & component_mask)
                                                >> component_shift)
                                                + sign,
                                            CHUNK_SIZE,
                                            0,
                                            CHUNK_SIZE_MASK,
                                        ) << component_shift);

                                    // Index into the neighbor blocks collections and call .GetPoint() with adjusted local position.
                                    //
                                    // Remark: If there's no neighbor at the index given, no chunk exists there (for instance, chunks)
                                    // at the edge of render distance).
                                    let neighbor_x = neighbor_local_position & CHUNK_SIZE_MASK;
                                    let neighbor_y = (neighbor_local_position >> CHUNK_SIZE_SHIFT)
                                        & CHUNK_SIZE_MASK;
                                    let neighbor_z = (neighbor_local_position
                                        >> (CHUNK_SIZE_SHIFT * 2))
                                        & CHUNK_SIZE_MASK;

                                    let neighbor_blocks_index = neighbor_x
                                        + (CHUNK_SIZE * (neighbor_z + (CHUNK_SIZE * neighbor_y)));

                                    if let Some(neighbor_palette) = neighbors[normal_index as usize]
                                    {
                                        let faced_block_id =
                                            neighbor_palette.get(neighbor_blocks_index as usize);

                                        if is_transparent {
                                            if block.id() == faced_block_id.id() {
                                                break;
                                            }
                                        } else if !block_registry
                                            .get_block_attributes(faced_block_id.id())
                                            .contains(block::Attributes::TRANSPARENT)
                                        {
                                            break;
                                        }
                                    }
                                } else {
                                    // Amount to add to current traversal index to get the block currently
                                    // being faced by our traverser.
                                    let faced_block_index = traversal_index
                                        + INDEX_STEP_BY_NORMAL_INDEX[normal_index as usize];
                                    let faced_block_id = blocks[faced_block_index as usize].id();

                                    if is_transparent {
                                        if block.id() == faced_block_id {
                                            break;
                                        }
                                    } else if block_registry
                                        .get_block_attributes(faced_block_id)
                                        .contains(block::Attributes::TRANSPARENT)
                                    {
                                        if !is_negative_normal {
                                            // The current face is culled, and the faced block is opaque, so
                                            // cull its face adjacent to the current block.
                                            faces[faced_block_index as usize] |=
                                                DIRECTION::from_bits_truncate(
                                                    (1 << ((normal_index + 3) % 6)) as u8,
                                                );
                                        }

                                        break;
                                    }
                                }

                                faces[traversal_index as usize] |= face_direction;
                                traversal_index += traversal_index_step;
                                total_traversal_len += 1;
                                traversals += 1;
                            }

                            // Face is occluded.
                            if traversals == 0 {
                                break;
                            }
                            // If it's the first traversal and we've only made a 1x1 face, continue and test the next axis
                            else if traversals == 1 && perpendicular_normal_index == 1 {
                                continue;
                            }

                            let compressed_vertexes =
                                PACKED_VERTEX_BY_NORMAL_INDEX[normal_index as usize];
                            let traversal_component_mask =
                                CHUNK_SIZE_MASK << traversal_normal_shift;
                            let unary_traversal_component_mask = !traversal_component_mask;

                            // This solution should probably be temporary, as it seems like it *should*
                            // be able to be optimized.
                            let uv_shift = (component_index
                                + traversal_index
                                + if component_index == 1 && traversal_normal_index == 2 {
                                    1
                                } else {
                                    0
                                })
                                % 2;

                            let indexes_start = (vertexes.len() * 4) as u32;
                            indexes.push(QuadIndexes::new([
                                indexes_start + 0,
                                indexes_start + 1,
                                indexes_start + 3,
                                indexes_start + 1,
                                indexes_start + 2,
                                indexes_start + 3,
                            ]));

                            // TODO uvz calculations
                            vertexes.push(QuadVertexes::new([
                                PackedVertex {
                                    xyz: local_position
                                        + (unary_traversal_component_mask & compressed_vertexes[0])
                                        | ((compressed_vertexes[0] * traversals)
                                            & traversal_component_mask),
                                    uvz: 0,
                                },
                                PackedVertex {
                                    xyz: local_position
                                        + (unary_traversal_component_mask & compressed_vertexes[1])
                                        | ((compressed_vertexes[1] * traversals)
                                            & traversal_component_mask),
                                    uvz: 0,
                                },
                                PackedVertex {
                                    xyz: local_position
                                        + (unary_traversal_component_mask & compressed_vertexes[2])
                                        | ((compressed_vertexes[2] * traversals)
                                            & traversal_component_mask),
                                    uvz: 0,
                                },
                                PackedVertex {
                                    xyz: local_position
                                        + (unary_traversal_component_mask & compressed_vertexes[3])
                                        | ((compressed_vertexes[3] * traversals)
                                            & traversal_component_mask),
                                    uvz: 0,
                                },
                            ]));

                            break;
                        }
                    }
                }
            }
        }
    }
}

#[inline(always)]
const fn wrap(mut value: i32, delta: i32, min_val: i32, max_val: i32) -> i32 {
    let mod_val = (max_val + 1) - min_val;
    value += delta - min_val;
    value += (1 - (value / mod_val)) * mod_val;
    (value % mod_val) + min_val
}
