use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

fn main() {
    App::new()
    .add_plugins(DefaultPlugins)
    .add_systems(Startup, setup)
    .add_systems(Update, orbit_camera_system)
        .run();
}

#[derive(Resource)]
struct OrbitCamera {
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub target: Vec3,
    pub rotating: bool,
}

// (removed unused helper make_image; we generate/supply high-res images via make_and_save)

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Load the provided PNGs from assets
    let grass_handle = asset_server.load("grass.png");
    let tronco_handle = asset_server.load("tronco.png");
    let madera_handle = asset_server.load("madera.png");

    let mat_grass = materials.add(StandardMaterial {
        base_color_texture: Some(grass_handle.clone()),
        perceptual_roughness: 1.0,
        ..Default::default()
    });

    let mat_tronco = materials.add(StandardMaterial {
        base_color_texture: Some(tronco_handle.clone()),
        perceptual_roughness: 0.9,
        ..Default::default()
    });

    let mat_madera = materials.add(StandardMaterial {
        base_color_texture: Some(madera_handle.clone()),
        perceptual_roughness: 0.8,
        ..Default::default()
    });

    // Shared meshes to reduce GPU uploads
    let cube = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));

    // Helper: create a plane mesh with tiled UVs (width along X, depth along Z)
    fn create_tiled_plane(width: f32, depth: f32, tiles_u: f32, tiles_v: f32) -> Mesh {
        let hw = width / 2.0;
        let hd = depth / 2.0;
        let positions = vec![
            [-hw, 0.0, -hd],
            [ hw, 0.0, -hd],
            [ hw, 0.0,  hd],
            [-hw, 0.0,  hd],
        ];
        let normals = vec![[0.0, 1.0, 0.0]; 4];
        let uvs = vec![
            [0.0, 0.0],
            [tiles_u, 0.0],
            [tiles_u, tiles_v],
            [0.0, tiles_v],
        ];
        let indices: Vec<u32> = vec![0, 2, 1, 0, 3, 2];
        let mut mesh = Mesh::new(bevy::render::mesh::PrimitiveTopology::TriangleList);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));
        mesh
    }

    // Create a tiled ground plane
    let ground_size = 200.0f32;
    let ground_tiles = 128.0f32; // increase repeats for better detail
    let ground_mesh = meshes.add(create_tiled_plane(ground_size, ground_size, ground_tiles, ground_tiles));

    // Floor - large tiled plane using the provided grass texture
    commands.spawn(PbrBundle {
        mesh: ground_mesh.clone(),
        material: mat_grass.clone(),
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        ..Default::default()
    });

    // Read individual layer files (layer_0.txt .. layer_4.txt) and spawn cubes stacked on Y
    let layer_files = [
        "assets/layer_0.txt",
        "assets/layer_1.txt",
        "assets/layer_2.txt",
        "assets/layer_3.txt",
        "assets/layer_4.txt",
        "assets/layer_5.txt",
        "assets/layer_6.txt",
    ];

    for (layer_idx, path) in layer_files.iter().enumerate() {
        let text = std::fs::read_to_string(path).unwrap_or_else(|_| {
            warn!("Could not read {}, skipping", path);
            String::new()
        });

        if text.trim().is_empty() {
            continue;
        }

        let rows: Vec<&str> = text.lines().collect();
        let rows_count = rows.len();
        if rows_count == 0 {
            continue;
        }

        // assume all rows have the same width
        let cols = rows[0].chars().count();
        let cols_f = cols as f32;
        let rows_f = rows_count as f32;

        for (row_idx, row) in rows.iter().enumerate() {
            for (col_idx, ch) in row.chars().enumerate() {
                // center blocks so the house is centered at origin
                let x = col_idx as f32 - (cols_f - 1.0) / 2.0;
                let z = - (row_idx as f32 - (rows_f - 1.0) / 2.0);
                let y = layer_idx as f32; // stack by file index
                let pos = Vec3::new(x, y, z);

                let bundle = match ch {
                    'g' | 'G' => Some(PbrBundle { mesh: cube.clone(), material: mat_grass.clone(), transform: Transform::from_translation(pos), ..Default::default() }),
                    't' | 'T' => Some(PbrBundle { mesh: cube.clone(), material: mat_tronco.clone(), transform: Transform::from_translation(pos), ..Default::default() }),
                    'm' | 'M' => Some(PbrBundle { mesh: cube.clone(), material: mat_madera.clone(), transform: Transform::from_translation(pos), ..Default::default() }),
                    'r' | 'R' => Some(PbrBundle { mesh: cube.clone(), material: mat_tronco.clone(), transform: Transform::from_translation(pos), ..Default::default() }),
                    '.' | ' ' => None,
                    _ => None,
                };

                if let Some(b) = bundle {
                    commands.spawn(b);
                }
            }
        }
    }

    // Directional light coming from the east (+X) angled downwards
    let mut light_transform = Transform::from_xyz(4.0, 8.0, 0.0);
    light_transform.look_at(Vec3::ZERO, Vec3::Y);
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: light_transform,
        ..Default::default()
    });

    // Camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(3.0, 3.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    // Orbit camera resource
    commands.insert_resource(OrbitCamera {
        yaw: -45f32.to_radians(),
        pitch: -25f32.to_radians(),
        // start farther back so we can see the whole house
        distance: 20.0,
        target: Vec3::new(0.0, 0.5, 0.0),
        rotating: false,
    });
}

fn orbit_camera_system(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    buttons: Res<Input<MouseButton>>,
    keys: Res<Input<KeyCode>>,
    mut cams: Query<&mut Transform, With<Camera3d>>,
    mut orbit: ResMut<OrbitCamera>,
) {
    // Determine if left mouse is down (rotate)
    let rotating = buttons.pressed(MouseButton::Left);
    orbit.rotating = rotating;

    // apply mouse motion to yaw/pitch
    for ev in mouse_motion_events.iter() {
        if orbit.rotating {
            let sens = 0.005;
            orbit.yaw -= ev.delta.x * sens;
            orbit.pitch -= ev.delta.y * sens;
            orbit.pitch = orbit.pitch.clamp(-1.5, 1.4);
        }
    }

    // zoom with wheel (smooth exponential scaling)
    for ev in mouse_wheel_events.iter() {
        // smaller factor for finer control
        orbit.distance *= (1.0 - ev.y * 0.06).clamp(0.5, 1.5);
    }

    // keyboard zoom: PageUp to zoom out, PageDown to zoom in
    if keys.pressed(KeyCode::PageUp) {
        orbit.distance += 0.5;
    }
    if keys.pressed(KeyCode::PageDown) {
        orbit.distance -= 0.5;
    }

    // clamp the distance to a larger maximum so user can zoom way out
    orbit.distance = orbit.distance.clamp(1.0, 500.0);

    // update camera transform(s)
    for mut t in cams.iter_mut() {
        let x = orbit.distance * orbit.pitch.cos() * orbit.yaw.cos();
        let z = orbit.distance * orbit.pitch.cos() * orbit.yaw.sin();
        let y = orbit.distance * orbit.pitch.sin();
        let pos = Vec3::new(orbit.target.x + x, orbit.target.y + y, orbit.target.z + z);
        *t = Transform::from_translation(pos).looking_at(orbit.target, Vec3::Y);
    }
}

fn grass_follow_system(
) { }

