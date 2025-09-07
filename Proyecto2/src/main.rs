use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseWheel};

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
) {
    // Red cube (shadow caster)
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.7, 0.1, 0.1),
            perceptual_roughness: 0.9,
            ..Default::default()
        }),
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

