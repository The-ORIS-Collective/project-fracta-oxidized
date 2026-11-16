use crate::core::physics::Transform3D;
use crate::core::scene::{ObjectBehavior, ObjectId, RenderCommand, SceneObject, TreeCommand, TreeContext};
use crate::objects::Camera3D;
use crate::core::input::InputSnapshot;

pub struct ObjectTree {
    objects: Vec<Option<SceneObject>>,
    root: Option<ObjectId>,
    pub pending_render_commands: Vec<RenderCommand>,
}

impl ObjectTree {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            root: None,
            pending_render_commands: Vec::new(),
        }
    }

    pub fn root(&self) -> Option<ObjectId> {
        self.root
    }

    pub fn scene_tree_snapshot(&self) -> Vec<(ObjectId, usize, String)> {
        self.objects
            .iter()
            .enumerate()
            .filter_map(|(index, slot)| {
                let object = slot.as_ref()?;
                let mut depth = 0;
                let mut parent = object.parent;
                while let Some(parent_id) = parent {
                    let Some(parent_object) = self
                        .objects
                        .get(parent_id.0)
                        .and_then(Option::as_ref)
                    else {
                        break;
                    };
                    depth += 1;
                    parent = parent_object.parent;
                }
                Some((ObjectId(index), depth, object.name.clone()))
            })
            .collect()
    }

    pub fn add_object(
        &mut self,
        name: &str,
        parent: Option<ObjectId>,
        behavior: Box<dyn ObjectBehavior>,
    ) -> ObjectId {
        let id = ObjectId(self.objects.len());

        self.objects.push(Some(SceneObject {
            name: name.to_string(),
            parent,
            children: Vec::new(),
            behavior,
        }));

        if let Some(parent_id) = parent {
            if let Some(Some(parent_obj)) = self.objects.get_mut(parent_id.0) {
                parent_obj.children.push(id);
            }
        } else if self.root.is_none() {
            self.root = Some(id);
        }

        // Trigger `start()` immediately upon entering the tree
        let mut pending_commands = Vec::new();
        let empty_input = InputSnapshot::default();
        if let Some(mut obj) = self.objects[id.0].take() {
            let mut ctx = TreeContext::new(&self.objects, &empty_input);
            obj.behavior.start(id, &mut ctx);
            pending_commands = ctx.commands;
            self.pending_render_commands.extend(ctx.render_commands);
            self.objects[id.0] = Some(obj);
        }

        self.process_commands(pending_commands);
        id
    }

    pub fn update(&mut self, delta: f32, input: &InputSnapshot) {
        let mut all_commands = Vec::new();

        for i in 0..self.objects.len() {
            if let Some(mut obj) = self.objects[i].take() {
                let mut ctx = TreeContext::new(&self.objects, input);
                obj.behavior.frame_tick(ObjectId(i), delta, &mut ctx);
                all_commands.extend(ctx.commands);
                self.pending_render_commands.extend(ctx.render_commands);
                self.objects[i] = Some(obj);
            }
        }

        self.process_commands(all_commands);
    }

    pub fn fixed_update(&mut self, fixed_delta: f32, input: &InputSnapshot) {
        let mut all_commands = Vec::new();

        for i in 0..self.objects.len() {
            if let Some(mut obj) = self.objects[i].take() {
                let mut ctx = TreeContext::new(&self.objects, input);
                obj.behavior.fixed_tick(ObjectId(i), fixed_delta, &mut ctx);
                all_commands.extend(ctx.commands);
                self.pending_render_commands.extend(ctx.render_commands);
                self.objects[i] = Some(obj);
            }
        }

        self.process_commands(all_commands);
    }


    // * --- Spatial & Camera Helpers for Bevy Sync ---

    /// Computes an object's global Transform3D by walking up its parent chain.
    pub fn get_global_transform(&self, id: ObjectId) -> Option<Transform3D> {
        let obj = self.objects.get(id.0)?.as_ref()?;
        let local_tf = *obj.behavior.transform()?;

        match obj.parent {
            Some(parent_id) => {
                if let Some(parent_global) = self.get_global_transform(parent_id) {
                    Some(parent_global.combine(&local_tf))
                } else {
                    Some(local_tf)
                }
            }
            None => Some(local_tf),
        }
    }

    pub fn get_transform_mut(&mut self, id: ObjectId) -> Option<&mut Transform3D> {
        self.objects
            .get_mut(id.0)?
            .as_mut()?
            .behavior
            .transform_mut()
    }

    /// Finds the global Transform3D of the first active `Camera3D` in the tree.
    pub fn find_active_camera_transform(&self) -> Option<Transform3D> {
        for (i, slot) in self.objects.iter().enumerate() {
            if let Some(obj) = slot {
                if let Some(cam) = obj.behavior.as_any().downcast_ref::<Camera3D>() {
                    if cam.is_current {
                        return self.get_global_transform(ObjectId(i));
                    }
                }
            }
        }
        None
    }

    // * --- Internal Command Processing ---

    fn process_commands(&mut self, commands: Vec<TreeCommand>) {
        for cmd in commands {
            match cmd {
                TreeCommand::Remove(id) => self.remove_object_recursive(id),
                TreeCommand::AddChild {
                    parent,
                    name,
                    behavior,
                } => {
                    self.add_object(&name, parent, behavior);
                }
            }
        }
    }

    fn remove_object_recursive(&mut self, id: ObjectId) {
        let Some(mut obj) = self.objects.get_mut(id.0).and_then(|slot| slot.take()) else {
            return;
        };

        // 1. Recursively free all children first
        for child_id in obj.children.drain(..) {
            self.remove_object_recursive(child_id);
        }

        // 2. Detach from parent's children list
        if let Some(parent_id) = obj.parent {
            if let Some(Some(parent_obj)) = self.objects.get_mut(parent_id.0) {
                parent_obj.children.retain(|&c| c != id);
            }
        }

        if self.root == Some(id) {
            self.root = None;
        }

        // 3. Call `end()` cleanup hook and guarantee Bevy visual despawn
        let empty_input = InputSnapshot::default();
        let mut ctx = TreeContext::new(&self.objects, &empty_input);
        obj.behavior.end(id, &mut ctx);
        let cleanup_commands = ctx.commands;
        self.pending_render_commands.extend(ctx.render_commands);
        self.pending_render_commands.push(RenderCommand::DespawnVisual(id));

        // Drop `obj` before processing any commands spawned during `end()`
        drop(obj);
        self.process_commands(cleanup_commands);
    }
}
