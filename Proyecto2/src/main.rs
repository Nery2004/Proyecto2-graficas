use raylib::prelude::*;

fn main() {
    // Init window
    let (mut rl, thread) = raylib::init()
        .size(1280, 720)
        .title("Cubo rojo con sombra - Rust + raylib")
        .build();

    rl.set_target_fps(60);


    // Orbit camera state
    let mut cam_target = Vector3::new(0.0, 0.5, 0.0);
    let mut cam_yaw: f32 = -45.0_f32.to_radians();
    let mut cam_pitch: f32 = -20.0_f32.to_radians();
    let mut cam_distance: f32 = 8.0;
    let mut rotating = false;
    let mut panning = false;
    let mut last_mouse = rl.get_mouse_position();

    // Cube state
    let cube_pos = Vector3::new(0.0, 0.5, 0.0);
    let cube_size = 1.0f32;

    // Light direction (world space) - pointing downwards
    let light_dir = normalize(Vector3::new(-1.0, -2.0, -0.5));

    while !rl.window_should_close() {
        // Mouse input
        let mouse = rl.get_mouse_position();
        let dx = mouse.x - last_mouse.x;
        let dy = mouse.y - last_mouse.y;

        // Left button: rotate
        if rl.is_mouse_button_pressed(raylib::consts::MouseButton::MOUSE_BUTTON_LEFT) {
            rotating = true;
        }
        if rl.is_mouse_button_released(raylib::consts::MouseButton::MOUSE_BUTTON_LEFT) {
            rotating = false;
        }

        // Right button: pan
        if rl.is_mouse_button_pressed(raylib::consts::MouseButton::MOUSE_BUTTON_RIGHT) {
            panning = true;
        }
        if rl.is_mouse_button_released(raylib::consts::MouseButton::MOUSE_BUTTON_RIGHT) {
            panning = false;
        }

        if rotating && rl.is_mouse_button_down(raylib::consts::MouseButton::MOUSE_BUTTON_LEFT) {
            let sens = 0.01;
            cam_yaw -= dx * sens;
            cam_pitch -= dy * sens;
            // clamp pitch to avoid flipping
            cam_pitch = cam_pitch.clamp(-1.5, 1.4);
        }

        if panning && rl.is_mouse_button_down(raylib::consts::MouseButton::MOUSE_BUTTON_RIGHT) {
            // Pan target in camera plane
            let cam_pos = spherical_to_cartesian(cam_yaw, cam_pitch, cam_distance, cam_target);
            let forward = normalize(sub(cam_target, cam_pos));
            let right = normalize(cross(forward, Vector3::new(0.0, 1.0, 0.0)));
            let up = cross(right, forward);
            let pan_speed = 0.01 * cam_distance;
            cam_target = add3(cam_target, add3(mult(right, -dx * pan_speed), mult(up, dy * pan_speed)));
        }

        // Zoom with scroll
        let wheel = rl.get_mouse_wheel_move();
        if wheel != 0.0 {
            cam_distance *= 1.0 - wheel * 0.1;
            cam_distance = cam_distance.clamp(1.0, 100.0);
        }

        last_mouse = mouse;

        // Compute camera position from spherical coords
        let cam_pos = spherical_to_cartesian(cam_yaw, cam_pitch, cam_distance, cam_target);
        let camera = Camera3D::perspective(cam_pos, cam_target, Vector3::new(0.0, 1.0, 0.0), 45.0);

        // Drawing
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::RAYWHITE);

        let mut d3 = d.begin_mode3D(camera);

        // Ground
        d3.draw_plane(Vector3::new(0.0, 0.0, 0.0), Vector2::new(50.0, 50.0), Color::LIGHTGRAY);

        // Compute projected shadow of cube onto ground plane (y = 0) using light direction
        // Project the 8 cube vertices along light_dir to y = 0, then compute convex hull in XZ and draw triangles
        let half = cube_size / 2.0;
        let verts = [
            Vector3::new(cube_pos.x - half, cube_pos.y - half, cube_pos.z - half),
            Vector3::new(cube_pos.x - half, cube_pos.y - half, cube_pos.z + half),
            Vector3::new(cube_pos.x + half, cube_pos.y - half, cube_pos.z - half),
            Vector3::new(cube_pos.x + half, cube_pos.y - half, cube_pos.z + half),
            Vector3::new(cube_pos.x - half, cube_pos.y + half, cube_pos.z - half),
            Vector3::new(cube_pos.x - half, cube_pos.y + half, cube_pos.z + half),
            Vector3::new(cube_pos.x + half, cube_pos.y + half, cube_pos.z - half),
            Vector3::new(cube_pos.x + half, cube_pos.y + half, cube_pos.z + half),
        ];

        let mut proj_points: Vec<(f32, f32, f32)> = Vec::new();
        for v in verts.iter() {
            // if light_dir.y == 0, skip (avoid div by 0)
            if light_dir.y.abs() < 1e-6 {
                continue;
            }
            let s = v.y / light_dir.y;
            // projected point: v - light_dir * s
            let px = v.x - light_dir.x * s;
            let pz = v.z - light_dir.z * s;
            proj_points.push((px, 0.01, pz));
        }

        // Convex hull in XZ
        let hull = convex_hull_xz(&proj_points);
        if hull.len() >= 3 {
            // compute centroid
            let mut cx = 0.0f32;
            let mut cz = 0.0f32;
            for (x, _y, z) in hull.iter() {
                cx += *x;
                cz += *z;
            }
            cx /= hull.len() as f32;
            cz /= hull.len() as f32;
            let center = Vector3::new(cx, 0.02, cz);
            // draw fan triangles
            for i in 0..hull.len() {
                let (x1, _y1, z1) = hull[i];
                let (x2, _y2, z2) = hull[(i + 1) % hull.len()];
                let p1 = Vector3::new(x1, 0.02, z1);
                let p2 = Vector3::new(x2, 0.02, z2);
                d3.draw_triangle3D(center, p1, p2, Color::new(200, 0, 0, 140));
            }
        }

        // Cube (red)
        d3.draw_cube(cube_pos, cube_size, cube_size, cube_size, Color::new(200, 50, 50, 255));
        d3.draw_cube_wires(cube_pos, cube_size, cube_size, cube_size, Color::BLACK);

        // Helper grid and origin
        d3.draw_grid(20, 1.0);

        drop(d3);

        // UI / instructions (camera controls)
        d.draw_text("Izq: rotar, Der: pan, Rueda: zoom", 10, 10, 20, Color::DARKGRAY);
        if rotating {
            d.draw_text("Rotando (izq)", 10, 40, 20, Color::MAROON);
        }
        if panning {
            d.draw_text("Panning (der)", 10, 70, 20, Color::MAROON);
        }

        drop(d);
    }
}

// Minimal Ray structure for our math
struct RaySimple {
    position: Vector3,
    direction: Vector3,
}

// Convert screen mouse position to a world ray using camera parameters
fn screen_pos_to_ray(mouse: Vector2, camera: &Camera3D, screen_w: f32, screen_h: f32) -> RaySimple {
    // Normalized device coords [-1,1]
    let ndc_x = (2.0 * mouse.x / screen_w) - 1.0;
    let ndc_y = 1.0 - (2.0 * mouse.y / screen_h);

    // Camera basis
    let cam_pos = camera.position;
    let cam_target = camera.target;
    let cam_up = camera.up;

    let forward = normalize(sub(cam_target, cam_pos));
    let right = normalize(cross(forward, cam_up));
    let up = cross(right, forward);

    // Field of view (vertical)
    let fovy_rad = camera.fovy.to_radians();
    let tan_fovy = (fovy_rad / 2.0).tan();
    let aspect = screen_w / screen_h;

    // Image plane offsets
    let px = ndc_x * tan_fovy * aspect;
    let py = ndc_y * tan_fovy;

    let dir = normalize(add3(mult(forward, 1.0), add3(mult(right, px), mult(up, py))));

    RaySimple { position: cam_pos, direction: dir }
}

fn ray_plane_intersection(ray: &RaySimple, plane_y: f32) -> Option<Vector3> {
    let dir = ray.direction;
    if dir.y.abs() < 1e-6 {
        return None;
    }
    let t = (plane_y - ray.position.y) / dir.y;
    if t < 0.0 {
        return None;
    }
    Some(Vector3::new(
        ray.position.x + dir.x * t,
        plane_y,
        ray.position.z + dir.z * t,
    ))
}

// Vector helpers using raylib::prelude::Vector3
fn add3(a: Vector3, b: Vector3) -> Vector3 {
    Vector3::new(a.x + b.x, a.y + b.y, a.z + b.z)
}

fn sub(a: Vector3, b: Vector3) -> Vector3 {
    Vector3::new(a.x - b.x, a.y - b.y, a.z - b.z)
}

fn mult(v: Vector3, s: f32) -> Vector3 {
    Vector3::new(v.x * s, v.y * s, v.z * s)
}

fn dot(a: Vector3, b: Vector3) -> f32 {
    a.x * b.x + a.y * b.y + a.z * b.z
}

fn cross(a: Vector3, b: Vector3) -> Vector3 {
    Vector3::new(
        a.y * b.z - a.z * b.y,
        a.z * b.x - a.x * b.z,
        a.x * b.y - a.y * b.x,
    )
}

fn length(v: Vector3) -> f32 {
    dot(v, v).sqrt()
}

fn normalize(v: Vector3) -> Vector3 {
    let len = length(v);
    if len == 0.0 {
        v
    } else {
        mult(v, 1.0 / len)
    }
}

// Convert spherical coords (yaw, pitch, distance) to world position around target
fn spherical_to_cartesian(yaw: f32, pitch: f32, dist: f32, target: Vector3) -> Vector3 {
    let x = dist * pitch.cos() * yaw.cos();
    let z = dist * pitch.cos() * yaw.sin();
    let y = dist * pitch.sin();
    Vector3::new(target.x + x, target.y + y, target.z + z)
}

// Convex hull of points projected on XZ plane using monotone chain
fn convex_hull_xz(points: &Vec<(f32, f32, f32)>) -> Vec<(f32, f32, f32)> {
    let mut pts: Vec<(f32, f32, f32)> = points.clone();
    // sort by x then z
    pts.sort_by(|a, b| {
        if a.0 < b.0 { std::cmp::Ordering::Less }
        else if a.0 > b.0 { std::cmp::Ordering::Greater }
        else if a.2 < b.2 { std::cmp::Ordering::Less }
        else if a.2 > b.2 { std::cmp::Ordering::Greater }
        else { std::cmp::Ordering::Equal }
    });
    pts.dedup_by(|a, b| (a.0 - b.0).abs() < 1e-6 && (a.2 - b.2).abs() < 1e-6);

    if pts.len() < 3 {
        return pts;
    }

    let cross = |o: &(f32, f32, f32), a: &(f32, f32, f32), b: &(f32, f32, f32)| -> f32 {
        (a.0 - o.0) * (b.2 - o.2) - (a.2 - o.2) * (b.0 - o.0)
    };

    let mut lower: Vec<(f32, f32, f32)> = Vec::new();
    for p in pts.iter() {
        while lower.len() >= 2 && cross(&lower[lower.len() - 2], &lower[lower.len() - 1], p) <= 0.0 {
            lower.pop();
        }
        lower.push(*p);
    }
    let mut upper: Vec<(f32, f32, f32)> = Vec::new();
    for p in pts.iter().rev() {
        while upper.len() >= 2 && cross(&upper[upper.len() - 2], &upper[upper.len() - 1], p) <= 0.0 {
            upper.pop();
        }
        upper.push(*p);
    }
    // Concatenate lower and upper (excluding last point of each to avoid duplication)
    lower.pop();
    upper.pop();
    lower.extend(upper);
    lower
}
