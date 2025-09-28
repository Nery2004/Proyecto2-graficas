use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::pbr::AlphaMode;
use bevy::ecs::component::Component;

#[derive(Component)]
struct BaseLightIntensity(f32);

fn main() {
    App::new()
    .add_plugins(DefaultPlugins)
    .add_systems(Startup, setup)
    .add_systems(Update, (
        orbit_camera_system,
        button_system,
        update_lighting_system,
    ))
        .run();
}

#[derive(Resource, Deref, DerefMut)]
struct NightMode(bool);

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
    let leaves_handle = asset_server.load("leaves.png");
    let agua_handle = asset_server.load("agua.png");
    let tierra_handle = asset_server.load("tierra.png");
    let vidrio_handle = asset_server.load("vidrio.png");
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
    // emissive material for lantern body
    let mat_lantern = materials.add(StandardMaterial {
        base_color: Color::rgb(1.0, 0.85, 0.6),
        emissive: Color::rgb(1.0, 0.6, 0.2),
        perceptual_roughness: 0.3,
        ..Default::default()
    });
    let mat_leaves = materials.add(StandardMaterial {
        base_color_texture: Some(leaves_handle.clone()),
        perceptual_roughness: 0.7,
        ..Default::default()
    });
    let mat_agua = materials.add(StandardMaterial {
        base_color_texture: Some(agua_handle.clone()),
        // make water glossy/reflective so it shows specular highlights from lamps
        perceptual_roughness: 0.06,
        reflectance: 0.6,
        metallic: 0.05,
        ..Default::default()
    });
    // transparent glass material (use 'w' in layers for windows)
    let mat_glass = materials.add(StandardMaterial {
        base_color: Color::rgba(0.8, 0.9, 1.0, 0.28),
        perceptual_roughness: 0.05,
        reflectance: 0.0,
        alpha_mode: AlphaMode::Blend,
        ..Default::default()
    });
        let mat_tierra = materials.add(StandardMaterial {
        base_color_texture: Some(tierra_handle.clone()),
        perceptual_roughness: 0.4,
        ..Default::default()
    });
    let mat_vidrio = materials.add(StandardMaterial {
        base_color_texture: Some(vidrio_handle.clone()),
        perceptual_roughness: 0.4,
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
        "assets/layer_7.txt",
        "assets/layer_8.txt",
        "assets/layer_9.txt",
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

                match ch {
                    'g' | 'G' => {
                        commands.spawn(PbrBundle { mesh: cube.clone(), material: mat_grass.clone(), transform: Transform::from_translation(pos), ..Default::default() });
                    }
                    't' | 'T' => {
                        commands.spawn(PbrBundle { mesh: cube.clone(), material: mat_tronco.clone(), transform: Transform::from_translation(pos), ..Default::default() });
                    }
                    'm' | 'M' => {
                        commands.spawn(PbrBundle { mesh: cube.clone(), material: mat_madera.clone(), transform: Transform::from_translation(pos), ..Default::default() });
                    }
                    'r' | 'R' => {
                        commands.spawn(PbrBundle { mesh: cube.clone(), material: mat_tronco.clone(), transform: Transform::from_translation(pos), ..Default::default() });
                    }
                    'l' | 'L' => {
                        // leaves (original behavior)
                        commands.spawn(PbrBundle { mesh: cube.clone(), material: mat_leaves.clone(), transform: Transform::from_translation(pos), ..Default::default() });
                    }
                    'f' | 'F' => {
                        // spawn a lantern composed entity: pole + lantern body (emissive) + point light
                        commands.spawn(SpatialBundle { transform: Transform::from_translation(pos), ..Default::default() })
                            .with_children(|parent| {
                                // lantern body (emissive cube)
                                parent.spawn(PbrBundle {
                                    mesh: cube.clone(),
                                    material: mat_lantern.clone(),
                                    transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)).with_scale(Vec3::splat(0.5)),
                                    ..Default::default()
                                });

                                // point light slightly above the cube center
                                parent.spawn((PointLightBundle {
                                    transform: Transform::from_translation(Vec3::new(0.0, 0.15, 0.0)),
                                    point_light: PointLight {
                                        intensity: 1200.0,
                                        range: 8.0,
                                        color: Color::rgb(1.0, 0.85, 0.6),
                                        shadows_enabled: true,
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }, BaseLightIntensity(1200.0)));
                            });
                    }
                    'a' | 'A' => {
                        commands.spawn(PbrBundle { mesh: cube.clone(), material: mat_agua.clone(), transform: Transform::from_translation(pos), ..Default::default() });
                    }
                    'w' | 'W' => {
                        // glass / window - transparent
                        commands.spawn(PbrBundle { mesh: cube.clone(), material: mat_glass.clone(), transform: Transform::from_translation(pos), ..Default::default() });
                    }
                    'd' | 'D' => {
                        commands.spawn(PbrBundle { mesh: cube.clone(), material: mat_tierra.clone(), transform: Transform::from_translation(pos), ..Default::default() });
                    }
                    'v' | 'V' => {
                        commands.spawn(PbrBundle { mesh: cube.clone(), material: mat_vidrio.clone(), transform: Transform::from_translation(pos), ..Default::default() });
                    }
                    '.' | ' ' => { /* empty */ }
                    _ => { /* unknown char */ }
                }
            }
        }
    }

    // Directional light coming from the east (+X) angled downwards
    let mut light_transform = Transform::from_xyz(4.0, 8.0, 0.0);
    light_transform.look_at(Vec3::ZERO, Vec3::Y);
    // spawn directional light and tag it so we can toggle
    commands.spawn((DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: light_transform,
        ..Default::default()
    }, Name::new("MainDirectionalLight")));

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

    // UI: top-right button to toggle night/day (absolute positioned)
    commands.spawn(NodeBundle {
        style: Style {
            position_type: PositionType::Absolute,
            top: Val::Px(8.0),
            right: Val::Px(8.0),
            ..Default::default()
        },
        background_color: BackgroundColor(Color::NONE),
        ..Default::default()
    }).with_children(|parent| {
        parent.spawn(ButtonBundle {
            style: Style {
                padding: UiRect::all(Val::Px(8.0)),
                margin: UiRect::all(Val::Px(4.0)),
                ..Default::default()
            },
            background_color: BackgroundColor(Color::rgb(0.15, 0.15, 0.2)),
            ..Default::default()
        }).with_children(|b| {
            b.spawn(TextBundle::from_section("Toggle Night", TextStyle {
                font_size: 16.0,
                color: Color::WHITE,
                ..Default::default()
            }));
        });
    });

    // start in day mode
    commands.insert_resource(NightMode(false));
}

// System: handle button interaction
fn button_system(
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<Button>)>,
    mut night: ResMut<NightMode>,
) {
    for (interaction, mut bg) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                // toggle night mode
                **night = !**night;
                *bg = BackgroundColor(Color::rgb(0.2, 0.2, 0.25));
            }
            Interaction::Hovered => {
                *bg = BackgroundColor(Color::rgb(0.2, 0.2, 0.3));
            }
            Interaction::None => {
                *bg = BackgroundColor(Color::rgb(0.15, 0.15, 0.2));
            }
        }
    }
}

// System: update lighting and background color based on NightMode
fn update_lighting_system(
    night: Res<NightMode>,
    mut dir_query: Query<&mut DirectionalLight>,
    mut clear_color: ResMut<ClearColor>,
    mut point_lights: Query<(&mut PointLight, &BaseLightIntensity)>,
) {
    if night.is_changed() {
        if **night {
            // night: dark sky, dim directional light
            clear_color.0 = Color::rgb(0.02, 0.03, 0.06);
            for mut dl in &mut dir_query {
                dl.illuminance = 800.0;
            }
            // boost lantern point lights
            for (mut pl, base) in &mut point_lights {
                pl.intensity = base.0 * 1.6; // 60% brighter at night
                pl.range = (pl.range).max(8.0);
            }
        } else {
            // day
            clear_color.0 = Color::rgb(0.5, 0.7, 1.0);
            for mut dl in &mut dir_query {
                dl.illuminance = 10000.0;
            }
            // restore lantern intensities
            for (mut pl, base) in &mut point_lights {
                pl.intensity = base.0;
            }
        }
    }
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

    // apply mouse motion to yaw/pitch or panning depending on mouse button
    for ev in mouse_motion_events.iter() {
        if orbit.rotating {
            let sens = 0.005;
            orbit.yaw -= ev.delta.x * sens;
            orbit.pitch -= ev.delta.y * sens;
            orbit.pitch = orbit.pitch.clamp(-1.5, 1.4);
        } else if buttons.pressed(MouseButton::Right) {
            // pan the camera target horizontally in XZ plane using right-drag
            // compute right and forward vectors on XZ plane from yaw
            let pan_sens = 0.002 * orbit.distance.max(1.0);
            let right_dir = Vec3::new(-orbit.yaw.sin(), 0.0, orbit.yaw.cos());
            let forward_dir = Vec3::new(orbit.yaw.cos(), 0.0, orbit.yaw.sin());
            // move target opposite to mouse X (dragging right moves view right)
            orbit.target -= right_dir * ev.delta.x * pan_sens;
            // move target along forward for vertical mouse movement (dragging up moves forward)
            orbit.target += forward_dir * ev.delta.y * pan_sens;
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

