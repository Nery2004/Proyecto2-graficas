use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::render::texture::Image as BevyImage;
use image::{RgbaImage, Rgba};
use std::fs::create_dir_all;

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

fn make_image<F>(width: u32, height: u32, mut f: F) -> BevyImage
where
    F: FnMut(u32, u32) -> [u8; 4],
{
    let mut data = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            let px = f(x, y);
            data.push(px[0]);
            data.push(px[1]);
            data.push(px[2]);
            data.push(px[3]);
        }
    }

    BevyImage::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
    )
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<BevyImage>>,
) {
    // Higher-quality textures: 1024x1024 with procedural detail
    let size = 1024u32;

    // Ensure assets dir exists so saved PNGs are available for inspection
    let _ = create_dir_all("assets");

    // Helper to create both a Bevy image and save a PNG to assets
    fn make_and_save<F>(name: &str, width: u32, height: u32, mut f: F) -> (BevyImage, String)
    where
        F: FnMut(u32, u32) -> [u8; 4],
    {
        // build RgbaImage for saving
        let mut img = RgbaImage::new(width, height);
        let mut data = Vec::with_capacity((width * height * 4) as usize);
        for y in 0..height {
            for x in 0..width {
                let px = f(x, y);
                img.put_pixel(x, y, Rgba(px));
                data.push(px[0]);
                data.push(px[1]);
                data.push(px[2]);
                data.push(px[3]);
            }
        }
        // save PNG
        let path = format!("assets/{}", name);
        let _ = img.save(&path);
        // create Bevy image
        let bevy_img = BevyImage::new(
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            data,
            TextureFormat::Rgba8UnormSrgb,
        );
        (bevy_img, path)
    }

    // grass: layered noise + subtle blades
    let (grass, grass_path) = make_and_save("high_grass.png", size, size, |x, y| {
        let fx = x as f32 / size as f32;
        let fy = y as f32 / size as f32;
        // layered hash noise
        let n1 = ((x.wrapping_mul(73856093) ^ y.wrapping_mul(19349663)) & 255) as u8;
        let n2 = (((x / 4).wrapping_mul(73856093) ^ (y / 4).wrapping_mul(19349663)) & 255) as u8;
        let variation = ((n1 as u16 + (n2 as u16 / 2)) / 2) as u8;
        // blade pattern
        let blade = ((fx * 40.0).sin() * (fy * 60.0).cos() * 0.5 + 0.5) * 40.0;
        let base_g = 120u8.saturating_add((variation as f32 * 0.7) as u8);
        let g = (base_g as f32 + blade) as u8;
        [30u8, g, 18u8, 255u8]
    });

    // tronco: vertical grain with sine perturbation and noise
    let (tronco, tronco_path) = make_and_save("high_tronco.png", size, size, |x, y| {
        let fx = x as f32 / 16.0;
        let grain = (fx + (y as f32 / 128.0).sin() * 2.0).sin();
        let base = 90.0 + grain * 40.0;
        let noise = ((x.wrapping_mul(73856093) ^ y.wrapping_mul(19349663)) & 127) as f32 / 127.0 * 20.0;
        let r = (base + noise) as u8;
        let g = (base * 0.7 + noise * 0.6) as u8;
        let b = (base * 0.4) as u8;
        [r, g, b, 255u8]
    });

    // madera: horizontal grain with occasional knots
    let (madera, madera_path) = make_and_save("high_madera.png", size, size, |x, y| {
        let fy = y as f32 / 12.0;
        let grain = (fy + (x as f32 / 200.0).sin() * 1.5).sin();
        let base = 150.0 + grain * 40.0;
        // knots
        let cx = (size as f32 * 0.45) as i32;
        let cy = (size as f32 * 0.5) as i32;
        let dx = x as i32 - cx;
        let dy = y as i32 - cy;
        let dist = ((dx * dx + dy * dy) as f32).sqrt();
        let knot = if dist < (size as f32 * 0.08) { (1.0 - dist / (size as f32 * 0.08)) * -60.0 } else { 0.0 };
        let noise = ((x.wrapping_mul(73856093) ^ y.wrapping_mul(19349663)) & 127) as f32 / 127.0 * 12.0;
        let r = (base + knot + noise) as u8;
        let g = (base * 0.8 + knot * 0.6 + noise * 0.6) as u8;
        let b = (base * 0.5) as u8;
        [r, g, b, 255u8]
    });

    // Add images to asset storage and create material handles
    let grass_handle = images.add(grass);
    let tronco_handle = images.add(tronco);
    let madera_handle = images.add(madera);

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
    let plane = meshes.add(Mesh::from(shape::Plane { size: 1.0, subdivisions: 1 }));

    // Floor (large plane) - tiling simulated by scaling the plane and using small texture
    commands.spawn(PbrBundle {
        mesh: plane.clone(),
        material: mat_grass.clone(),
        transform: Transform::from_scale(Vec3::new(10.0, 1.0, 10.0)).with_translation(Vec3::new(0.0, 0.0, 0.0)),
        ..Default::default()
    });

    // Create a rectangular platform (raised) where the house stands
    commands.spawn(PbrBundle {
        mesh: cube.clone(),
        material: mat_madera.clone(),
        transform: Transform::from_xyz(0.0, 0.25, 0.0).with_scale(Vec3::new(6.0, 0.5, 6.0)),
        ..Default::default()
    });

    // Walls - darker wood (tronco) for posts and lighter for planks
    // Four outer walls using cubes scaled
    let wall_thickness = 0.3;
    let wall_height = 2.0;
    let wall_length = 5.0;

    // Back wall
    commands.spawn(PbrBundle {
        mesh: cube.clone(),
        material: mat_madera.clone(),
        transform: Transform::from_xyz(0.0, wall_height / 2.0 + 0.5, -wall_length / 2.0)
            .with_scale(Vec3::new(wall_length, wall_height, wall_thickness)),
        ..Default::default()
    });

    // Front wall (with gap for door)
    commands.spawn(PbrBundle {
        mesh: cube.clone(),
        material: mat_madera.clone(),
        transform: Transform::from_xyz(0.0, wall_height / 2.0 + 0.5, wall_length / 2.0)
            .with_scale(Vec3::new(wall_length, wall_height, wall_thickness)),
        ..Default::default()
    });

    // Left wall
    commands.spawn(PbrBundle {
        mesh: cube.clone(),
        material: mat_madera.clone(),
        transform: Transform::from_xyz(-wall_length / 2.0, wall_height / 2.0 + 0.5, 0.0)
            .with_scale(Vec3::new(wall_thickness, wall_height, wall_length)),
        ..Default::default()
    });

    // Right wall
    commands.spawn(PbrBundle {
        mesh: cube.clone(),
        material: mat_madera.clone(),
        transform: Transform::from_xyz(wall_length / 2.0, wall_height / 2.0 + 0.5, 0.0)
            .with_scale(Vec3::new(wall_thickness, wall_height, wall_length)),
        ..Default::default()
    });

    // Posts (tronco) at corners
    let post_scale = Vec3::new(0.3, wall_height + 0.5, 0.3);
    let corners = [
        Vec3::new(-wall_length / 2.0, 0.0, -wall_length / 2.0),
        Vec3::new(wall_length / 2.0, 0.0, -wall_length / 2.0),
        Vec3::new(-wall_length / 2.0, 0.0, wall_length / 2.0),
        Vec3::new(wall_length / 2.0, 0.0, wall_length / 2.0),
    ];
    for c in corners {
        commands.spawn(PbrBundle {
            mesh: cube.clone(),
            material: mat_tronco.clone(),
            transform: Transform::from_translation(Vec3::new(c.x, (wall_height + 0.5) / 2.0 + 0.5, c.z)).with_scale(post_scale),
            ..Default::default()
        });
    }

    // Simple roof: two sloped slabs made from scaled cubes rotated
    let roof_thickness = 0.2;
    let roof_length = wall_length + 0.6;
    let roof_height = 1.2;

    // Left roof half
    commands.spawn(PbrBundle {
        mesh: cube.clone(),
        material: mat_tronco.clone(),
        transform: Transform::from_xyz(0.0, wall_height + roof_height / 2.0 + 0.5, 0.0)
            .with_rotation(Quat::from_rotation_z(0.35))
            .with_scale(Vec3::new(roof_length, roof_thickness, 3.5)),
        ..Default::default()
    });

    // Right roof half
    commands.spawn(PbrBundle {
        mesh: cube.clone(),
        material: mat_tronco.clone(),
        transform: Transform::from_xyz(0.0, wall_height + roof_height / 2.0 + 0.5, 0.0)
            .with_rotation(Quat::from_rotation_z(-0.35))
            .with_scale(Vec3::new(roof_length, roof_thickness, 3.5)),
        ..Default::default()
    });

    // Simple stairs at front
    for i in 0..4 {
        let h = 0.12 + i as f32 * 0.12;
        let depth = 0.6;
        commands.spawn(PbrBundle {
            mesh: cube.clone(),
            material: mat_madera.clone(),
            transform: Transform::from_xyz(0.0, h + 0.01, wall_length / 2.0 + 0.3 + i as f32 * depth).with_scale(Vec3::new(1.2, 0.12, depth)),
            ..Default::default()
        });
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

