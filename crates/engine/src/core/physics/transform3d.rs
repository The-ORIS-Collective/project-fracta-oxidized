use bevy::prelude::*;

pub const CHUNK_SIZE: f32 = 8192.0;


#[derive(Component, Debug, Clone, Copy)]
pub struct Transform3D {
    pub chunk    :   [i64; 3],  // Grid cell coordinate (virtually infinite)
    pub position :   [f32; 3],  // Local x, y, z offset inside the chunk
    pub rotation :   [f32; 4],  // Quaternion x, y, z, w
    pub scale    :   [f32; 3],  // x, y, z
}



impl Default for Transform3D {
    fn default() -> Self {
        Self::identity()
    }
}


impl Transform3D {
    /// Creates a new transform at the origin with no rotation and unit scale.
    pub fn identity() -> Self {
        Self {
            chunk    : [0, 0, 0],
            position : [0.0, 0.0, 0.0],
            rotation : [0.0, 0.0, 0.0, 1.0],
            scale    : [1.0, 1.0, 1.0],
        }
    }



    /// Moves the position of the transform by adding an offset to its current position.
    pub fn translate(&mut self, dx: f32, dy: f32, dz: f32) {
        self.position[0] += dx;
        self.position[1] += dy;
        self.position[2] += dz;

        self.normalize_chunk();
    }

    /// Wraps local position into [-CHUNK_SIZE/2, +CHUNK_SIZE/2] and updates chunk index.
    pub fn normalize_chunk(&mut self) {
        let half = CHUNK_SIZE * 0.5;
        for i in 0..3 {
            while self.position[i] > half {
                self.position[i] -= CHUNK_SIZE;
                self.chunk[i] += 1;
            }
            while self.position[i] < -half {
                self.position[i] += CHUNK_SIZE;
                self.chunk[i] -= 1;
            }
        }
    }



    // * --- Local Translation ---

    /// Moves the transform relative to its current facing direction.
    pub fn translate_local(&mut self, local_offset: Vec3) {
        let world_offset = Quat::from_array(self.rotation) * local_offset;
        self.translate(world_offset.x, world_offset.y, world_offset.z);
    }


    // * --- World-Axis Rotations (Radians) ---

    /// Applies an arbitrary quaternion rotation in world space.
    pub fn rotate(&mut self, delta: Quat) {
        let current = Quat::from_array(self.rotation);
        self.rotation = (delta * current).normalize().to_array();
    }

    /// Rotates around the world X axis (pitch).
    pub fn rotate_x(&mut self, radians: f32) {
        self.rotate(Quat::from_rotation_x(radians));
    }

    /// Rotates around the world Y axis (yaw).
    pub fn rotate_y(&mut self, radians: f32) {
        self.rotate(Quat::from_rotation_y(radians));
    }

    /// Rotates around the world Z axis (roll).
    pub fn rotate_z(&mut self, radians: f32) {
        self.rotate(Quat::from_rotation_z(radians));
    }




    // * --- Local-Axis Rotations (Radians) ---

    /// Rotates around the transform's own local axes.
    pub fn rotate_local(&mut self, local_delta: Quat) {
        let current = Quat::from_array(self.rotation);
        self.rotation = (current * local_delta).normalize().to_array();
    }

    /// Pitches up/down around the transform's local X axis.
    pub fn rotate_local_x(&mut self, radians: f32) {
        self.rotate_local(Quat::from_rotation_x(radians));
    }

    /// Yaws left/right around the transform's local Y axis.
    pub fn rotate_local_y(&mut self, radians: f32) {
        self.rotate_local(Quat::from_rotation_y(radians));
    }

    /// Rolls around the transform's local Z axis.
    pub fn rotate_local_z(&mut self, radians: f32) {
        self.rotate_local(Quat::from_rotation_z(radians));
    }





    // * --- Helper Functions ---

    /// Returns the exact 3D vector from `self` to `other` in f64 world units.
    pub fn delta_to(&self, other: &Self) -> [f64; 3] {
        let chunk_size = CHUNK_SIZE as f64;
        [
            (other.chunk[0] as f64 - self.chunk[0] as f64) * chunk_size
                + (other.position[0] as f64 - self.position[0] as f64),
            (other.chunk[1] as f64 - self.chunk[1] as f64) * chunk_size
                + (other.position[1] as f64 - self.position[1] as f64),
            (other.chunk[2] as f64 - self.chunk[2] as f64) * chunk_size
                + (other.position[2] as f64 - self.position[2] as f64),
        ]
    }

    /// Returns the squared Euclidean distance between `self` and `other`.
    /// Faster than `distance_to` for radius checks (avoids a square root).
    pub fn distance_squared_to(&self, other: &Self) -> f64 {
        let [dx, dy, dz] = self.delta_to(other);
        dx * dx + dy * dy + dz * dz
    }


    /// Returns the exact Euclidean distance between `self` and `other`.
    pub fn distance_to(&self, other: &Self) -> f64 {
        self.distance_squared_to(other).sqrt()
    }

    /// Checks if `other` is within `radius` units without computing a square root.
    pub fn is_within_range(&self, other: &Self, radius: f64) -> bool {
        self.distance_squared_to(other) <= radius * radius
    }

    /// Returns the local forward direction (-Z).
    pub fn forward(&self) -> Vec3 {
        Quat::from_array(self.rotation) * Vec3::NEG_Z
    }

    /// Returns the local right direction (+X).
    pub fn right(&self) -> Vec3 {
        Quat::from_array(self.rotation) * Vec3::X
    }

    /// Returns the local up direction (+Y).
    pub fn up(&self) -> Vec3 {
        Quat::from_array(self.rotation) * Vec3::Y
    }

    /// Rotates the transform so its forward axis (-Z) points directly at `target`.
    pub fn look_at(&mut self, target: &Self, up: Vec3) {
        let [dx, dy, dz] = self.delta_to(target);
        let len_sq = dx * dx + dy * dy + dz * dz;

        if len_sq > f64::EPSILON {
            let len = len_sq.sqrt();
            let dir = Vec3::new((dx / len) as f32, (dy / len) as f32, (dz / len) as f32);
            let mut temp = Transform::IDENTITY;
            temp.look_to(dir, up);
            self.rotation = temp.rotation.to_array();
        }
    }

}
