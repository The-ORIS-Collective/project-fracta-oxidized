#[derive(Component)]
pub struct OriginAnchor;

pub fn sync_chunk_origin(
    anchor_query: Query<&Transform3D, With<OriginAnchor>>,
    mut entities: Query<&Transform3D, &mut Transform>
) {
    let Ok(anchor) = anchor_query.single() else {
        return;
    };

    for (world_tf, mut render_tf) in &mut entities {
        // 1. Subtract integer chunk coordinates first
        let dx_chunk = (world_tf.chunk[0] - anchor.chunk[0]) as f32 * CHUNK_SIZE;
        let dy_chunk = (world_tf.chunk[1] - anchor.chunk[1]) as f32 * CHUNK_SIZE;
        let dz_chunk = (world_tf.chunk[2] - anchor.chunk[2]) as f32 * CHUNK_SIZE;

        // 2. Add local f32 position
        render_tf.translation = Vec3::new(
            dx_chunk + world_tf.position[0],
            dy_chunk + world_tf.position[1],
            dz_chunk + world_tf.position[2],
        );

        render_tf.rotation = Quat::from_array(world_tf.rotation);
        render_tf.scale = Vec3::from_array(world_tf.scale);
    }
}