use macroquad::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const WORLD_RADIUS: i32 = 28;
const CHUNK_SIZE: i32 = 16;
const PLAYER_RADIUS: f32 = 0.30;
const PLAYER_HEIGHT: f32 = 1.80;
const REACH: f32 = 6.0;
const GRAVITY: f32 = 24.0;
const JUMP_SPEED: f32 = 8.5;
const HOTBAR_SIZE: usize = 7;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct BlockPos {
    x: i32,
    y: i32,
    z: i32,
}

impl BlockPos {
    fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    fn from_world(point: Vec3) -> Self {
        Self::new(
            point.x.floor() as i32,
            point.y.floor() as i32,
            point.z.floor() as i32,
        )
    }

    fn center(self) -> Vec3 {
        vec3(self.x as f32 + 0.5, self.y as f32 + 0.5, self.z as f32 + 0.5)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Block {
    Grass,
    Dirt,
    Stone,
    Wood,
    Leaves,
    Sand,
    CoalOre,
}

impl Block {
    const ALL: [Block; HOTBAR_SIZE] = [
        Block::Grass,
        Block::Dirt,
        Block::Stone,
        Block::Wood,
        Block::Leaves,
        Block::Sand,
        Block::CoalOre,
    ];

    fn index(self) -> usize {
        match self {
            Block::Grass => 0,
            Block::Dirt => 1,
            Block::Stone => 2,
            Block::Wood => 3,
            Block::Leaves => 4,
            Block::Sand => 5,
            Block::CoalOre => 6,
        }
    }

    fn from_index(index: usize) -> Option<Self> {
        Self::ALL.get(index).copied()
    }

    fn name(self) -> &'static str {
        match self {
            Block::Grass => "Grass",
            Block::Dirt => "Dirt",
            Block::Stone => "Stone",
            Block::Wood => "Wood",
            Block::Leaves => "Leaves",
            Block::Sand => "Sand",
            Block::CoalOre => "Coal ore",
        }
    }

    fn color(self) -> Color {
        match self {
            Block::Grass => Color::from_rgba(91, 157, 66, 255),
            Block::Dirt => Color::from_rgba(126, 83, 53, 255),
            Block::Stone => Color::from_rgba(120, 126, 133, 255),
            Block::Wood => Color::from_rgba(142, 101, 58, 255),
            Block::Leaves => Color::from_rgba(57, 132, 67, 255),
            Block::Sand => Color::from_rgba(218, 199, 137, 255),
            Block::CoalOre => Color::from_rgba(70, 76, 82, 255),
        }
    }
}

struct Player {
    position: Vec3, // feet position
    velocity: Vec3,
    yaw: f32,
    pitch: f32,
    on_ground: bool,
}

struct Game {
    world: HashMap<BlockPos, Block>,
    chunk_meshes: HashMap<(i32, i32), Mesh>,
    mesh_dirty: bool,
    player: Player,
    inventory: [u32; HOTBAR_SIZE],
    selected: usize,
    cursor_locked: bool,
    dirty: bool,
    save_clock: f32,
    autosave_clock: f32,
    message: String,
    message_clock: f32,
}

impl Game {
    fn new() -> Self {
        let mut world = generate_world();
        let mut player = Player {
            position: vec3(0.5, 12.0, -9.5),
            velocity: Vec3::ZERO,
            yaw: 0.0,
            pitch: -0.10,
            on_ground: false,
        };
        let mut inventory = [0_u32; HOTBAR_SIZE];
        inventory[Block::Dirt.index()] = 32;
        inventory[Block::Stone.index()] = 32;
        inventory[Block::Wood.index()] = 16;
        inventory[Block::Sand.index()] = 16;
        let mut selected = 0;

        if let Some(saved) = load_game() {
            world = saved.world;
            player.position = saved.position;
            player.yaw = saved.yaw;
            player.pitch = saved.pitch.clamp(-1.45, 1.45);
            inventory = saved.inventory;
            selected = saved.selected.min(HOTBAR_SIZE - 1);
        }

        let chunk_meshes = build_chunk_meshes(&world);

        Self {
            world,
            chunk_meshes,
            mesh_dirty: false,
            player,
            inventory,
            selected,
            cursor_locked: false,
            dirty: false,
            save_clock: 0.0,
            autosave_clock: 0.0,
            message: "Welcome to RustBlocks — click to play".to_string(),
            message_clock: 5.0,
        }
    }

    fn camera_forward(&self) -> Vec3 {
        let cp = self.player.pitch.cos();
        vec3(
            self.player.yaw.sin() * cp,
            self.player.pitch.sin(),
            self.player.yaw.cos() * cp,
        )
        .normalize_or_zero()
    }

    fn camera_position(&self) -> Vec3 {
        self.player.position + vec3(0.0, 1.62, 0.0)
    }

    fn show_message(&mut self, text: impl Into<String>) {
        self.message = text.into();
        self.message_clock = 2.6;
    }

    fn lock_cursor(&mut self) {
        self.cursor_locked = true;
        set_cursor_grab(true);
        show_mouse(false);
    }

    fn unlock_cursor(&mut self) {
        self.cursor_locked = false;
        set_cursor_grab(false);
        show_mouse(true);
    }

    fn update(&mut self, dt: f32) {
        self.message_clock = (self.message_clock - dt).max(0.0);
        self.save_clock += dt;
        self.autosave_clock += dt;

        if is_key_pressed(KeyCode::Escape) {
            if self.cursor_locked {
                self.unlock_cursor();
                self.show_message("Paused — click the window to continue");
            } else {
                self.lock_cursor();
            }
        }

        if !self.cursor_locked {
            if is_mouse_button_pressed(MouseButton::Left) {
                self.lock_cursor();
            }
            return;
        }

        self.update_look();
        self.update_hotbar();
        self.update_movement(dt);
        self.update_block_actions();
        if self.mesh_dirty {
            self.chunk_meshes = build_chunk_meshes(&self.world);
            self.mesh_dirty = false;
        }

        if self.dirty && self.save_clock >= 1.0 {
            self.save_now();
        } else if self.autosave_clock >= 12.0 {
            self.save_now();
        }
    }

    fn update_look(&mut self) {
        let delta = mouse_delta_position();
        // mouse_delta_position() is measured in normalized screen coordinates.
        let sensitivity = 2.4;
        self.player.yaw -= delta.x * sensitivity;
        self.player.pitch = (self.player.pitch - delta.y * sensitivity).clamp(-1.48, 1.48);
    }

    fn update_hotbar(&mut self) {
        for slot in 0..HOTBAR_SIZE {
            let key = match slot {
                0 => KeyCode::Key1,
                1 => KeyCode::Key2,
                2 => KeyCode::Key3,
                3 => KeyCode::Key4,
                4 => KeyCode::Key5,
                5 => KeyCode::Key6,
                _ => KeyCode::Key7,
            };
            if is_key_pressed(key) {
                self.selected = slot;
            }
        }

        let (_, wheel_y) = mouse_wheel();
        if wheel_y > 0.0 {
            self.selected = (self.selected + HOTBAR_SIZE - 1) % HOTBAR_SIZE;
        } else if wheel_y < 0.0 {
            self.selected = (self.selected + 1) % HOTBAR_SIZE;
        }
    }

    fn update_movement(&mut self, dt: f32) {
        let forward = self.camera_forward();
        let horizontal_forward = vec3(forward.x, 0.0, forward.z).normalize_or_zero();
        let right = vec3(horizontal_forward.z, 0.0, -horizontal_forward.x);
        let mut wish = Vec3::ZERO;

        if is_key_down(KeyCode::W) {
            wish += horizontal_forward;
        }
        if is_key_down(KeyCode::S) {
            wish -= horizontal_forward;
        }
        if is_key_down(KeyCode::D) {
            wish += right;
        }
        if is_key_down(KeyCode::A) {
            wish -= right;
        }

        let speed = if is_key_down(KeyCode::LeftShift) { 7.0 } else { 5.0 };
        if wish.length_squared() > 0.0 {
            wish = wish.normalize() * speed;
        }

        self.player.velocity.x = wish.x;
        self.player.velocity.z = wish.z;
        if is_key_pressed(KeyCode::Space) && self.player.on_ground {
            self.player.velocity.y = JUMP_SPEED;
            self.player.on_ground = false;
        }
        self.player.velocity.y -= GRAVITY * dt;

        let move_x = self.player.velocity.x * dt;
        let mut candidate = self.player.position;
        candidate.x += move_x;
        if !collides(&self.world, candidate) {
            self.player.position.x = candidate.x;
        }

        let move_z = self.player.velocity.z * dt;
        candidate = self.player.position;
        candidate.z += move_z;
        if !collides(&self.world, candidate) {
            self.player.position.z = candidate.z;
        }

        let move_y = self.player.velocity.y * dt;
        candidate = self.player.position;
        candidate.y += move_y;
        if !collides(&self.world, candidate) {
            self.player.position.y = candidate.y;
            self.player.on_ground = false;
        } else {
            if move_y < 0.0 {
                self.player.on_ground = true;
            }
            self.player.velocity.y = 0.0;
        }

        // Keep the player inside the generated play area.
        self.player.position.x = self.player.position.x.clamp(-WORLD_RADIUS as f32, WORLD_RADIUS as f32);
        self.player.position.z = self.player.position.z.clamp(-WORLD_RADIUS as f32, WORLD_RADIUS as f32);
    }

    fn update_block_actions(&mut self) {
        let origin = self.camera_position();
        let direction = self.camera_forward();
        let hit = raycast(&self.world, origin, direction, REACH);

        if is_mouse_button_pressed(MouseButton::Left) {
            if let Some((target, _)) = hit {
                if let Some(block) = self.world.remove(&target) {
                    let slot = block.index();
                    self.inventory[slot] = self.inventory[slot].saturating_add(1);
                    self.dirty = true;
                    self.mesh_dirty = true;
                    self.save_clock = 0.0;
                    self.show_message(format!("Mined {}", block.name()));
                }
            }
        }

        if is_mouse_button_pressed(MouseButton::Right) {
            if let Some((_, place_at)) = hit {
                let block = Block::ALL[self.selected];
                if self.inventory[self.selected] == 0 {
                    self.show_message(format!("No {} blocks in your hotbar", block.name()));
                    return;
                }
                let candidate_player = self.player.position;
                if overlaps_player(place_at, candidate_player) {
                    self.show_message("You cannot place a block inside yourself");
                    return;
                }
                if !self.world.contains_key(&place_at)
                    && place_at.y >= 0
                    && place_at.y <= 64
                    && place_at.x.abs() <= WORLD_RADIUS
                    && place_at.z.abs() <= WORLD_RADIUS
                {
                    self.world.insert(place_at, block);
                    self.inventory[self.selected] -= 1;
                    self.dirty = true;
                    self.mesh_dirty = true;
                    self.save_clock = 0.0;
                    self.show_message(format!("Placed {}", block.name()));
                }
            }
        }
    }

    fn save_now(&mut self) {
        match save_game(
            &self.world,
            self.player.position,
            self.player.yaw,
            self.player.pitch,
            self.inventory,
            self.selected,
        ) {
            Ok(()) => {
                self.dirty = false;
                self.save_clock = 0.0;
                self.autosave_clock = 0.0;
            }
            Err(error) => {
                self.save_clock = 0.0;
                self.autosave_clock = 0.0;
                self.show_message(format!("Save failed: {error}"));
            }
        }
    }

    fn draw(&self) {
        clear_background(Color::from_rgba(134, 190, 230, 255));
        let camera_position = self.camera_position();
        let camera = Camera3D {
            position: camera_position,
            target: camera_position + self.camera_forward(),
            up: vec3(0.0, 1.0, 0.0),
            fovy: 72.0_f32.to_radians(),
            projection: Projection::Perspective,
            ..Default::default()
        };
        set_camera(&camera);

        // A subtle far-distance ground/sky horizon.
        draw_plane(
            vec3(0.0, -0.02, 0.0),
            vec2((WORLD_RADIUS * 2) as f32, (WORLD_RADIUS * 2) as f32),
            None,
            Color::from_rgba(105, 154, 89, 255),
        );

        let max_chunk_distance_squared = 58.0 * 58.0;
        for ((chunk_x, chunk_z), mesh) in &self.chunk_meshes {
            let center_x = (*chunk_x * CHUNK_SIZE) as f32 + CHUNK_SIZE as f32 * 0.5;
            let center_z = (*chunk_z * CHUNK_SIZE) as f32 + CHUNK_SIZE as f32 * 0.5;
            let dx = center_x - self.player.position.x;
            let dz = center_z - self.player.position.z;
            if dx * dx + dz * dz <= max_chunk_distance_squared {
                draw_mesh(mesh);
            }
        }

        let hit = raycast(&self.world, camera_position, self.camera_forward(), REACH);
        if let Some((target, _)) = hit {
            draw_cube_wires(
                target.center(),
                vec3(1.015, 1.015, 1.015),
                Color::from_rgba(255, 255, 255, 220),
            );
        }

        set_default_camera();
        self.draw_hud();
    }

    fn draw_hud(&self) {
        let sw = screen_width();
        let sh = screen_height();
        let center_x = sw * 0.5;
        let center_y = sh * 0.5;

        // Crosshair.
        draw_line(center_x - 8.0, center_y, center_x + 8.0, center_y, 2.0, WHITE);
        draw_line(center_x, center_y - 8.0, center_x, center_y + 8.0, 2.0, WHITE);

        draw_text("RUSTBLOCKS  //  ALPHA", 18.0, 30.0, 22.0, WHITE);
        draw_text(
            &format!("XYZ  {:.1}  {:.1}  {:.1}", self.player.position.x, self.player.position.y, self.player.position.z),
            18.0,
            52.0,
            16.0,
            WHITE,
        );

        let slot_size = 54.0;
        let gap = 5.0;
        let total_width = HOTBAR_SIZE as f32 * slot_size + (HOTBAR_SIZE - 1) as f32 * gap;
        let start_x = (sw - total_width) * 0.5;
        let y = sh - 76.0;

        for slot in 0..HOTBAR_SIZE {
            let x = start_x + slot as f32 * (slot_size + gap);
            let selected = slot == self.selected;
            let bg = if selected {
                Color::from_rgba(245, 245, 245, 230)
            } else {
                Color::from_rgba(20, 24, 30, 190)
            };
            draw_rectangle(x, y, slot_size, slot_size, bg);
            draw_rectangle_lines(
                x,
                y,
                slot_size,
                slot_size,
                if selected { 3.0 } else { 1.5 },
                if selected { YELLOW } else { LIGHTGRAY },
            );
            let block = Block::ALL[slot];
            draw_rectangle(x + 12.0, y + 9.0, 30.0, 27.0, block.color());
            draw_rectangle_lines(x + 12.0, y + 9.0, 30.0, 27.0, 1.0, BLACK);
            draw_text(&format!("{}", slot + 1), x + 4.0, y + 13.0, 12.0, WHITE);
            draw_text(
                &format!("{}", self.inventory[slot]),
                x + 34.0,
                y + 50.0,
                14.0,
                if selected { BLACK } else { WHITE },
            );
        }

        let selected_block = Block::ALL[self.selected];
        let selected_text = format!("{}  |  {} blocks", selected_block.name(), self.inventory[self.selected]);
        let text_width = measure_text(&selected_text, None, 18, 1.0).width;
        draw_text(&selected_text, (sw - text_width) * 0.5, y - 12.0, 18.0, WHITE);

        if self.message_clock > 0.0 {
            let width = measure_text(&self.message, None, 20, 1.0).width;
            draw_rectangle(
                (sw - width) * 0.5 - 14.0,
                sh * 0.19 - 24.0,
                width + 28.0,
                36.0,
                Color::from_rgba(15, 18, 22, 155),
            );
            draw_text(&self.message, (sw - width) * 0.5, sh * 0.19, 20.0, WHITE);
        }

        if !self.cursor_locked {
            draw_rectangle(0.0, 0.0, sw, sh, Color::from_rgba(0, 0, 0, 105));
            let title = "RUSTBLOCKS";
            let title_width = measure_text(title, None, 48, 1.0).width;
            draw_text(title, (sw - title_width) * 0.5, sh * 0.36, 48.0, WHITE);
            let subtitle = "A voxel sandbox built with Rust";
            let sub_width = measure_text(subtitle, None, 22, 1.0).width;
            draw_text(subtitle, (sw - sub_width) * 0.5, sh * 0.36 + 38.0, 22.0, WHITE);
            let hint = "CLICK TO PLAY   •   WASD MOVE   •   SPACE JUMP   •   ESC PAUSE";
            let hint_width = measure_text(hint, None, 18, 1.0).width;
            draw_text(hint, (sw - hint_width) * 0.5, sh * 0.56, 18.0, WHITE);
            let hint2 = "LEFT CLICK MINE   •   RIGHT CLICK PLACE   •   1–7 / WHEEL SELECT";
            let hint2_width = measure_text(hint2, None, 17, 1.0).width;
            draw_text(hint2, (sw - hint2_width) * 0.5, sh * 0.56 + 30.0, 17.0, WHITE);
        }
    }
}

struct SavedGame {
    world: HashMap<BlockPos, Block>,
    position: Vec3,
    yaw: f32,
    pitch: f32,
    inventory: [u32; HOTBAR_SIZE],
    selected: usize,
}

fn terrain_height(x: i32, z: i32) -> i32 {
    let xf = x as f32;
    let zf = z as f32;
    let rolling = (xf * 0.21).sin() * 1.7
        + (zf * 0.17).cos() * 1.6
        + ((xf + zf) * 0.11).sin() * 1.1;
    (3.0 + rolling).round().clamp(0.0, 7.0) as i32
}

fn hash2(x: i32, z: i32) -> u32 {
    let mut value = (x as u32).wrapping_mul(0x8da6_b343)
        ^ (z as u32).wrapping_mul(0xd816_3841);
    value ^= value >> 13;
    value = value.wrapping_mul(0x85eb_ca6b);
    value ^= value >> 16;
    value
}

fn generate_world() -> HashMap<BlockPos, Block> {
    let mut world = HashMap::new();
    for x in -WORLD_RADIUS..=WORLD_RADIUS {
        for z in -WORLD_RADIUS..=WORLD_RADIUS {
            let height = terrain_height(x, z);
            for y in 0..=height {
                let block = if y == height {
                    if height <= 1 { Block::Sand } else { Block::Grass }
                } else if y >= height - 2 {
                    Block::Dirt
                } else {
                    let ore_roll = hash2(x + y * 17, z - y * 31) % 41;
                    if ore_roll == 0 { Block::CoalOre } else { Block::Stone }
                };
                world.insert(BlockPos::new(x, y, z), block);
            }

            let tree_roll = hash2(x, z) % 37;
            let near_spawn = x.abs() < 5 && (z + 9).abs() < 5;
            if tree_roll == 0 && !near_spawn && height >= 2 {
                let trunk_height = 3 + (hash2(z, x) % 2) as i32;
                for dy in 1..=trunk_height {
                    world.insert(BlockPos::new(x, height + dy, z), Block::Wood);
                }
                let crown_y = height + trunk_height;
                for dx in -2..=2 {
                    for dz in -2..=2 {
                        for dy in -1..=2 {
                            let distance = dx * dx + dz * dz + (dy - 1) * (dy - 1);
                            if distance <= 6 && !(dx == 0 && dz == 0 && dy <= 0) {
                                let p = BlockPos::new(x + dx, crown_y + dy, z + dz);
                                world.entry(p).or_insert(Block::Leaves);
                            }
                        }
                    }
                }
            }
        }
    }
    world
}

fn shaded(color: Color, factor: f32) -> Color {
    Color::new(color.r * factor, color.g * factor, color.b * factor, color.a)
}

fn append_quad(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u16>,
    corners: [Vec3; 4],
    color: Color,
) {
    // A chunk mesh uses u16 indices; keep every index within that range.
    if vertices.len() > (u16::MAX as usize).saturating_sub(4) {
        return;
    }
    let base = vertices.len() as u16;
    for corner in corners {
        vertices.push(Vertex::new2(corner, vec2(0.0, 0.0), color));
    }
    indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}

fn build_chunk_mesh(
    world: &HashMap<BlockPos, Block>,
    chunk_x: i32,
    chunk_z: i32,
) -> Mesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for (position, block) in world {
        if position.x.div_euclid(CHUNK_SIZE) != chunk_x
            || position.z.div_euclid(CHUNK_SIZE) != chunk_z
        {
            continue;
        }

        let x0 = position.x as f32;
        let y0 = position.y as f32;
        let z0 = position.z as f32;
        let x1 = x0 + 1.0;
        let y1 = y0 + 1.0;
        let z1 = z0 + 1.0;
        let base = block.color();

        if !world.contains_key(&BlockPos::new(position.x, position.y, position.z + 1)) {
            append_quad(&mut vertices, &mut indices,
                [vec3(x0,y0,z1), vec3(x1,y0,z1), vec3(x1,y1,z1), vec3(x0,y1,z1)],
                shaded(base, 0.78));
        }
        if !world.contains_key(&BlockPos::new(position.x, position.y, position.z - 1)) {
            append_quad(&mut vertices, &mut indices,
                [vec3(x1,y0,z0), vec3(x0,y0,z0), vec3(x0,y1,z0), vec3(x1,y1,z0)],
                shaded(base, 0.68));
        }
        if !world.contains_key(&BlockPos::new(position.x + 1, position.y, position.z)) {
            append_quad(&mut vertices, &mut indices,
                [vec3(x1,y0,z1), vec3(x1,y0,z0), vec3(x1,y1,z0), vec3(x1,y1,z1)],
                shaded(base, 0.72));
        }
        if !world.contains_key(&BlockPos::new(position.x - 1, position.y, position.z)) {
            append_quad(&mut vertices, &mut indices,
                [vec3(x0,y0,z0), vec3(x0,y0,z1), vec3(x0,y1,z1), vec3(x0,y1,z0)],
                shaded(base, 0.72));
        }
        if !world.contains_key(&BlockPos::new(position.x, position.y + 1, position.z)) {
            append_quad(&mut vertices, &mut indices,
                [vec3(x0,y1,z0), vec3(x0,y1,z1), vec3(x1,y1,z1), vec3(x1,y1,z0)],
                shaded(base, 1.0));
        }
        if !world.contains_key(&BlockPos::new(position.x, position.y - 1, position.z)) {
            append_quad(&mut vertices, &mut indices,
                [vec3(x0,y0,z1), vec3(x0,y0,z0), vec3(x1,y0,z0), vec3(x1,y0,z1)],
                shaded(base, 0.52));
        }
    }

    Mesh {
        vertices,
        indices,
        texture: None,
    }
}

fn build_chunk_meshes(world: &HashMap<BlockPos, Block>) -> HashMap<(i32, i32), Mesh> {
    let mut chunk_keys = std::collections::HashSet::new();
    for position in world.keys() {
        chunk_keys.insert((
            position.x.div_euclid(CHUNK_SIZE),
            position.z.div_euclid(CHUNK_SIZE),
        ));
    }

    let mut meshes = HashMap::with_capacity(chunk_keys.len());
    for (chunk_x, chunk_z) in chunk_keys {
        let mesh = build_chunk_mesh(world, chunk_x, chunk_z);
        if !mesh.indices.is_empty() {
            meshes.insert((chunk_x, chunk_z), mesh);
        }
    }
    meshes
}

fn collides(world: &HashMap<BlockPos, Block>, feet: Vec3) -> bool {
    let min_x = (feet.x - PLAYER_RADIUS).floor() as i32;
    let max_x = (feet.x + PLAYER_RADIUS).floor() as i32;
    let min_y = feet.y.floor() as i32;
    let max_y = (feet.y + PLAYER_HEIGHT - 0.001).floor() as i32;
    let min_z = (feet.z - PLAYER_RADIUS).floor() as i32;
    let max_z = (feet.z + PLAYER_RADIUS).floor() as i32;

    for x in min_x..=max_x {
        for y in min_y..=max_y {
            for z in min_z..=max_z {
                if world.contains_key(&BlockPos::new(x, y, z)) {
                    let overlaps_x = feet.x + PLAYER_RADIUS > x as f32
                        && feet.x - PLAYER_RADIUS < x as f32 + 1.0;
                    let overlaps_y = feet.y + PLAYER_HEIGHT > y as f32
                        && feet.y < y as f32 + 1.0;
                    let overlaps_z = feet.z + PLAYER_RADIUS > z as f32
                        && feet.z - PLAYER_RADIUS < z as f32 + 1.0;
                    if overlaps_x && overlaps_y && overlaps_z {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn overlaps_player(block: BlockPos, feet: Vec3) -> bool {
    let min_x = feet.x - PLAYER_RADIUS;
    let max_x = feet.x + PLAYER_RADIUS;
    let min_y = feet.y;
    let max_y = feet.y + PLAYER_HEIGHT;
    let min_z = feet.z - PLAYER_RADIUS;
    let max_z = feet.z + PLAYER_RADIUS;
    max_x > block.x as f32
        && min_x < block.x as f32 + 1.0
        && max_y > block.y as f32
        && min_y < block.y as f32 + 1.0
        && max_z > block.z as f32
        && min_z < block.z as f32 + 1.0
}

fn raycast(
    world: &HashMap<BlockPos, Block>,
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
) -> Option<(BlockPos, BlockPos)> {
    let mut previous_empty = BlockPos::from_world(origin);
    let mut last_position = previous_empty;
    let step_size = 0.035;
    let mut distance = 0.0;
    while distance <= max_distance {
        let point = origin + direction * distance;
        let position = BlockPos::from_world(point);
        if position != last_position {
            if world.contains_key(&position) {
                return Some((position, previous_empty));
            }
            previous_empty = position;
            last_position = position;
        } else if world.contains_key(&position) {
            return Some((position, previous_empty));
        }
        distance += step_size;
    }
    None
}

fn save_file_path() -> PathBuf {
    // Keep saves outside the installation folder so normal Windows users can
    // save worlds even when the game is installed under Program Files.
    let data_root = if let Some(path) = std::env::var_os("LOCALAPPDATA") {
        PathBuf::from(path)
    } else if let Some(path) = std::env::var_os("XDG_DATA_HOME") {
        PathBuf::from(path)
    } else if let Some(home) = std::env::var_os("HOME") {
        PathBuf::from(home).join(".local").join("share")
    } else {
        return PathBuf::from("rustblocks_world.txt");
    };

    let app_dir = data_root.join("RustBlocks");
    if fs::create_dir_all(&app_dir).is_ok() {
        app_dir.join("rustblocks_world.txt")
    } else {
        PathBuf::from("rustblocks_world.txt")
    }
}

fn save_game(
    world: &HashMap<BlockPos, Block>,
    position: Vec3,
    yaw: f32,
    pitch: f32,
    inventory: [u32; HOTBAR_SIZE],
    selected: usize,
) -> std::io::Result<()> {
    let mut output = String::with_capacity(world.len() * 18 + 128);
    output.push_str("RUSTBLOCKS_SAVE 1\n");
    output.push_str(&format!(
        "P {:.4} {:.4} {:.4} {:.5} {:.5}\n",
        position.x, position.y, position.z, yaw, pitch
    ));
    output.push_str(&format!("I {}", selected));
    for count in inventory {
        output.push_str(&format!(" {count}"));
    }
    output.push('\n');

    let mut blocks: Vec<_> = world.iter().collect();
    blocks.sort_by_key(|(position, _)| (position.x, position.y, position.z));
    for (position, block) in blocks {
        output.push_str(&format!(
            "B {} {} {} {}\n",
            position.x,
            position.y,
            position.z,
            block.index()
        ));
    }
    fs::write(save_file_path(), output)
}

fn load_game() -> Option<SavedGame> {
    let path = save_file_path();
    if !path.exists() {
        return None;
    }
    let contents = fs::read_to_string(path).ok()?;
    let mut lines = contents.lines();
    if lines.next()? != "RUSTBLOCKS_SAVE 1" {
        return None;
    }

    let player_line: Vec<&str> = lines.next()?.split_whitespace().collect();
    if player_line.len() != 6 || player_line[0] != "P" {
        return None;
    }
    let px = player_line[1].parse::<f32>().ok()?;
    let py = player_line[2].parse::<f32>().ok()?;
    let pz = player_line[3].parse::<f32>().ok()?;
    let yaw = player_line[4].parse::<f32>().ok()?;
    let pitch = player_line[5].parse::<f32>().ok()?;
    if !px.is_finite() || !py.is_finite() || !pz.is_finite()
        || !yaw.is_finite() || !pitch.is_finite()
        || px.abs() > 300.0 || pz.abs() > 300.0 || py.abs() > 300.0
    {
        return None;
    }

    let inventory_line: Vec<&str> = lines.next()?.split_whitespace().collect();
    if inventory_line.len() != HOTBAR_SIZE + 2 || inventory_line[0] != "I" {
        return None;
    }
    let selected = inventory_line[1].parse::<usize>().ok()?.min(HOTBAR_SIZE - 1);
    let mut inventory = [0_u32; HOTBAR_SIZE];
    for slot in 0..HOTBAR_SIZE {
        inventory[slot] = inventory_line[slot + 2].parse::<u32>().ok()?;
    }

    let mut world = HashMap::new();
    for line in lines {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() != 5 || fields[0] != "B" {
            continue;
        }
        let x = match fields[1].parse::<i32>() { Ok(value) => value, Err(_) => continue };
        let y = match fields[2].parse::<i32>() { Ok(value) => value, Err(_) => continue };
        let z = match fields[3].parse::<i32>() { Ok(value) => value, Err(_) => continue };
        let block_index = match fields[4].parse::<usize>() { Ok(value) => value, Err(_) => continue };
        if let Some(block) = Block::from_index(block_index) {
            // Ignore unreasonable coordinates in a malformed save file.
            if x.abs() <= 256 && z.abs() <= 256 && (0..=64).contains(&y) {
                world.insert(BlockPos::new(x, y, z), block);
            }
        }
    }

    // An empty world is valid: the player may have mined every block.
    Some(SavedGame {
        world,
        position: vec3(px, py, pz),
        yaw,
        pitch,
        inventory,
        selected,
    })
}

#[macroquad::main("RustBlocks")]
async fn main() {
    let mut game = Game::new();
    loop {
        let dt = get_frame_time().clamp(0.0, 0.05);
        game.update(dt);
        game.draw();
        next_frame().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_indices_round_trip() {
        for (index, block) in Block::ALL.iter().enumerate() {
            assert_eq!(Block::from_index(index), Some(*block));
            assert_eq!(block.index(), index);
        }
    }

    #[test]
    fn generated_world_has_ground_and_surface() {
        let world = generate_world();
        assert!(world.contains_key(&BlockPos::new(0, 0, 0)));
        let surface = terrain_height(0, 0);
        assert_eq!(world.get(&BlockPos::new(0, surface, 0)), Some(&Block::Grass));
    }

    #[test]
    fn player_overlap_detects_block_intersection() {
        let feet = vec3(0.5, 1.0, 0.5);
        assert!(overlaps_player(BlockPos::new(0, 1, 0), feet));
        assert!(!overlaps_player(BlockPos::new(4, 1, 0), feet));
    }
}
