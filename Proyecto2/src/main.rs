use bevy::prelude::*;
use bevy::render::render_resource::Extent3d;
use bevy::render::texture::Image;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use std::fs::read;
use image::GenericImageView;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(orbit_camera_system)
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

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {

    // Prefer to load PNG via AssetServer. If only a JPG exists we convert it at runtime to a temporary PNG
    // so AssetServer can load it (avoids depending on a jpg AssetLoader at compile-time).
    let jpg_path = "assets/cube_texture.jpg";
    let png_tmp_path = "assets/_cube_texture_tmp.png";
    let texture_handle = if std::path::Path::new(jpg_path).exists() {
        // try to convert JPG -> PNG and save temporary PNG
        match read(jpg_path) {
            Ok(bytes) => match image::load_from_memory(&bytes) {
                Ok(img) => {
                    if let Err(e) = img.save(png_tmp_path) {
                        warn!("Failed to write temporary PNG: {}", e);
                        // fallback: let asset_server try the jpg directly
                        asset_server.load("cube_texture.jpg")
                    } else {
                        // Load the temporary PNG via AssetServer (PNG loader is usually available)
                        let handle = asset_server.load("_cube_texture_tmp.png");
                        // DO NOT delete the temporary PNG immediately — AssetServer loads asynchronously and
                        // will need the file available. The temp file can be removed manually later if desired.
                        handle
                    }
                }
                Err(e) => {
                    warn!("Failed to decode cube_texture.jpg: {}", e);
                    asset_server.load("cube_texture.jpg")
                }
            },
            Err(e) => {
                warn!("Failed to read assets/cube_texture.jpg: {}", e);
                asset_server.load("cube_texture.jpg")
            }
        }
    } else {
        // No JPG present — try PNG or whatever the asset server supports directly
        asset_server.load("cube_texture.png")
    };

    // Create a material that uses the texture handle
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        perceptual_roughness: 0.9,
        ..Default::default()
    });

    // Textured cube (shadow caster)
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: material_handle,
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..Default::default()
    });

    // Large near-ground plane to receive shadows (slightly lowered to avoid z-fighting when camera is top-down)
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 50.0, subdivisions: 1 })),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.2, 0.8, 0.2),
            ..Default::default()
        }),
        transform: Transform::from_xyz(0.0, -0.001, 0.0),
        ..Default::default()
    });

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
        distance: 6.0,
        target: Vec3::new(0.0, 0.5, 0.0),
        rotating: false,
    });
}

fn orbit_camera_system(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    buttons: Res<Input<MouseButton>>,
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

    // zoom with wheel
    for ev in mouse_wheel_events.iter() {
        orbit.distance *= 1.0 - ev.y * 0.1;
        orbit.distance = orbit.distance.clamp(1.0, 100.0);
    }

    // update camera transform(s)
    for mut t in cams.iter_mut() {
        let x = orbit.distance * orbit.pitch.cos() * orbit.yaw.cos();
        let z = orbit.distance * orbit.pitch.cos() * orbit.yaw.sin();
        let y = orbit.distance * orbit.pitch.sin();
        let pos = Vec3::new(orbit.target.x + x, orbit.target.y + y, orbit.target.z + z);
        *t = Transform::from_translation(pos).looking_at(orbit.target, Vec3::Y);
    }
}

