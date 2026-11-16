use engine::bevy::prelude::*;
use engine::prelude::*;

pub struct Player6DoF {
    pub body: RigidBody3D,
    pub current_linvel: Vec3,
    pub current_angvel: Vec3,
    pub acceleration: f32,
    pub boost_multiplier: f32,
    pub linear_damping: f32,
    pub roll_speed: f32,
    pub mouse_sensitivity: f32,
    pub pending_mouse_delta: Vec2,
}

impl Player6DoF {
    pub fn new() -> Self {
        Self {
            body: RigidBody3D::new(ColliderShape::Ball { radius: 1.0 }, 5.0),
            current_linvel: Vec3::ZERO,
            current_angvel: Vec3::ZERO,
            acceleration: 40.0,
            boost_multiplier: 5.0,
            linear_damping: 0.0,
            roll_speed: 2.2,
            mouse_sensitivity: 0.005,
            pending_mouse_delta: Vec2::ZERO,
        }
    }
}

impl ObjectBehavior for Player6DoF {
    fn start(&mut self, id: ObjectId, ctx: &mut TreeContext) {
        self.body.start(id, ctx);
    }

    fn frame_tick(&mut self, _id: ObjectId, _delta: f32, ctx: &mut TreeContext) {
        self.pending_mouse_delta += ctx.input.mouse_delta;
    }

    fn fixed_tick(&mut self, id: ObjectId, delta: f32, ctx: &mut TreeContext) {
        // 0. Sync with the physics engine's actual velocity after any collisions
        // (Adjust this line to match your engine's getter, e.g., ctx.get_velocity(id) or self.body.linear_velocity())
        if let Some((linvel, _angvel)) = ctx.get_velocity(id) {
            self.current_linvel = linvel;
        }

        let rot = Quat::from_array(self.body.transform.rotation);
        let mouse_delta = std::mem::take(&mut self.pending_mouse_delta);

        // 1. Local Translation Input (WASD + Space/LeftCtrl)
        let mut local_wish = Vec3::ZERO;
        if ctx.input.is_key_pressed(KeyCode::KeyW) {
            local_wish.z -= 1.0;
        }
        if ctx.input.is_key_pressed(KeyCode::KeyS) {
            local_wish.z += 1.0;
        }
        if ctx.input.is_key_pressed(KeyCode::KeyA) {
            local_wish.x -= 1.0;
        }
        if ctx.input.is_key_pressed(KeyCode::KeyD) {
            local_wish.x += 1.0;
        }
        if ctx.input.is_key_pressed(KeyCode::Space) {
            local_wish.y += 1.0;
        }
        if ctx.input.is_key_pressed(KeyCode::ControlLeft) {
            local_wish.y -= 1.0;
        }

        let accel = if ctx.input.is_key_pressed(KeyCode::ShiftLeft) {
            self.acceleration * self.boost_multiplier
        } else {
            self.acceleration
        };

        // Add thruster acceleration onto the post-collision velocity
        if local_wish.length_squared() > 0.0 {
            local_wish = local_wish.normalize();
            let world_accel = (rot * local_wish) * accel;
            self.current_linvel += world_accel * delta;
        }

        // Apply custom damping only if > 0.0
        if self.linear_damping > 0.0 {
            self.current_linvel *= (1.0 - self.linear_damping * delta).max(0.0);
        }

        // 2. 6DoF Local Rotation (Mouse to Pitch/Yaw + Q/E to Roll)
        let mut local_torque = Vec3::ZERO;
        if delta > 0.0 {
            let mouse_angular_speed = self.mouse_sensitivity / delta;
            local_torque.y -= mouse_delta.x * mouse_angular_speed;
            local_torque.x -= mouse_delta.y * mouse_angular_speed;
        }
        if ctx.input.is_key_pressed(KeyCode::KeyQ) {
            local_torque.z += self.roll_speed;
        }
        if ctx.input.is_key_pressed(KeyCode::KeyE) {
            local_torque.z -= self.roll_speed;
        }

        let target_world_angvel = rot * local_torque;
        self.current_angvel = target_world_angvel;
        ctx.set_velocity(id, self.current_linvel, self.current_angvel);
    }


    fn transform(&self) -> Option<&Transform3D> {
        Some(&self.body.transform)
    }

    fn transform_mut(&mut self) -> Option<&mut Transform3D> {
        Some(&mut self.body.transform)
    }
}
