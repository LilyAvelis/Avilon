#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use std::f32::consts::TAU;

use bevy::{prelude::*, window::PrimaryWindow};

#[derive(Component)]
struct Lamp;

#[derive(Component)]
struct Armillary {
    phase: f32,
    speed: f32,
}

#[derive(Resource, Default)]
struct Stage {
    time: f32,
    frozen: bool,
    lamp_aim: Option<Vec3>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Мира · Театр света · Bevy".into(),
                resolution: (1120, 740).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.022, 0.031, 0.055)))
        .insert_resource(GlobalAmbientLight {
            color: Color::srgb(0.45, 0.59, 0.86),
            brightness: 190.0,
            ..default()
        })
        .init_resource::<Stage>()
        .add_systems(Startup, setup)
        .add_systems(Update, animate)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut fonts: ResMut<Assets<bevy::text::Font>>,
) {
    let floor = materials.add(StandardMaterial {
        base_color: Color::srgb(0.12, 0.16, 0.23),
        perceptual_roughness: 0.9,
        ..default()
    });
    let plinth = materials.add(StandardMaterial {
        base_color: Color::srgb(0.18, 0.23, 0.31),
        metallic: 0.25,
        perceptual_roughness: 0.46,
        ..default()
    });
    let brass = materials.add(StandardMaterial {
        base_color: Color::srgb(0.82, 0.62, 0.33),
        metallic: 0.66,
        perceptual_roughness: 0.33,
        ..default()
    });
    let turquoise = materials.add(StandardMaterial {
        base_color: Color::srgb(0.16, 0.55, 0.56),
        metallic: 0.48,
        perceptual_roughness: 0.29,
        ..default()
    });
    let porcelain = materials.add(StandardMaterial {
        base_color: Color::srgb(0.91, 0.86, 0.72),
        perceptual_roughness: 0.36,
        ..default()
    });
    let emitter = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.77, 0.40),
        emissive: LinearRgba::new(6.5, 2.8, 0.82, 1.0),
        unlit: true,
        ..default()
    });

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(80.0, 80.0))),
        MeshMaterial3d(floor),
    ));
    for (radius, height, y) in [(3.6, 0.19, 0.095), (3.43, 0.20, 0.27), (1.9, 0.10, 0.40)] {
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(radius, height))),
            MeshMaterial3d(plinth.clone()),
            Transform::from_xyz(0.0, y, 0.0),
        ));
    }
    let tick = meshes.add(Cuboid::new(0.032, 0.012, 0.14));
    for index in 0..72 {
        let angle = index as f32 / 72.0 * TAU;
        let radius = 3.12;
        let length = if index % 6 == 0 { 1.85 } else { 1.0 };
        commands.spawn((
            Mesh3d(tick.clone()),
            MeshMaterial3d(brass.clone()),
            Transform::from_xyz(angle.sin() * radius, 0.375, angle.cos() * radius)
                .with_rotation(Quat::from_rotation_y(angle))
                .with_scale(Vec3::new(1.0, 1.0, length)),
        ));
    }

    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.88).mesh().uv(64, 32))),
        MeshMaterial3d(porcelain),
        Transform::from_xyz(0.0, 1.96, 0.0),
    ));
    for (inner, outer, phase, speed, material) in [
        (1.10, 1.18, 0.8, 0.45, brass.clone()),
        (1.31, 1.38, 2.4, -0.29, turquoise.clone()),
        (1.59, 1.665, 4.2, 0.20, brass),
    ] {
        commands.spawn((
            Mesh3d(meshes.add(Torus::new(inner, outer))),
            MeshMaterial3d(material),
            Transform::from_xyz(0.0, 1.96, 0.0),
            Armillary { phase, speed },
        ));
    }

    commands
        .spawn((
            PointLight {
                color: Color::srgb(1.0, 0.68, 0.30),
                intensity: 480_000.0,
                range: 18.0,
                radius: 0.24,
                shadow_maps_enabled: true,
                ..default()
            },
            Transform::from_xyz(3.8, 3.7, 2.0),
            Lamp,
        ))
        .with_children(|parent| {
            parent.spawn((
                Mesh3d(meshes.add(Sphere::new(0.13).mesh().uv(24, 12))),
                MeshMaterial3d(emitter),
            ));
        });
    commands.spawn((
        PointLight {
            color: Color::srgb(0.22, 0.63, 1.0),
            intensity: 190_000.0,
            range: 18.0,
            ..default()
        },
        Transform::from_xyz(-4.0, 2.8, -3.8),
    ));
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(5.0, 5.25, 9.1).looking_at(Vec3::new(0.0, 1.13, 0.0), Vec3::Y),
    ));

    let font = std::env::var_os("WINDIR")
        .and_then(|root| {
            std::fs::read(std::path::PathBuf::from(root).join("Fonts/segoeui.ttf")).ok()
        })
        .map(|bytes| fonts.add(bevy::text::Font::from_bytes(bytes)));
    let local_font = font.is_some();
    let font = font.unwrap_or_default();
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(30.0),
            top: px(23.0),
            flex_direction: FlexDirection::Column,
            row_gap: px(8.0),
            ..default()
        },
        children![
            (
                Text::new(if local_font {
                    "Театр света"
                } else {
                    "Theatre of light"
                }),
                TextFont {
                    font: font.clone().into(),
                    font_size: FontSize::Px(34.0),
                    ..default()
                },
                TextColor(Color::srgb(0.96, 0.88, 0.71))
            ),
            (
                Text::new(if local_font {
                    "Держи ЛКМ и двигай источник света  ·  Пробел — пауза"
                } else {
                    "Hold left mouse and move the light  /  Space to pause"
                }),
                TextFont {
                    font: font.into(),
                    font_size: FontSize::Px(16.0),
                    ..default()
                },
                TextColor(Color::srgb(0.68, 0.77, 0.86))
            ),
        ],
    ));
}

fn animate(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut lamps: Query<&mut Transform, (With<Lamp>, Without<Armillary>)>,
    mut rings: Query<(&Armillary, &mut Transform), Without<Lamp>>,
    mut stage: ResMut<Stage>,
) {
    if keys.just_pressed(KeyCode::Space) {
        stage.frozen = !stage.frozen;
    }
    let dt = time.delta_secs().min(0.05);
    if !stage.frozen {
        stage.time += dt;
    }
    for (ring, mut transform) in &mut rings {
        let angle = stage.time * ring.speed + ring.phase;
        transform.rotation = Quat::from_rotation_y(angle)
            * Quat::from_rotation_x(0.55 + ring.phase * 0.2)
            * Quat::from_rotation_z(angle * 0.83);
    }
    if mouse.pressed(MouseButton::Left)
        && let Ok(window) = windows.single()
        && let Some(cursor) = window.cursor_position()
    {
        let x = cursor.x / window.width() * 2.0 - 1.0;
        let z = cursor.y / window.height() * 2.0 - 1.0;
        stage.lamp_aim = Some(Vec3::new(x * 4.75, 3.15 + 0.35 * (1.0 - z), 1.3 + z * 4.0));
    } else if !stage.frozen {
        stage.lamp_aim = None;
    }
    let theta = stage.time * 0.48 + 0.58;
    let orbital = Vec3::new(
        theta.cos() * 4.3,
        3.2 + 0.45 * (theta * 1.3).sin(),
        theta.sin() * 4.3,
    );
    for mut lamp in &mut lamps {
        if !stage.frozen || mouse.pressed(MouseButton::Left) {
            let target = stage.lamp_aim.unwrap_or(orbital);
            lamp.translation = lamp.translation.lerp(target, 1.0 - (-9.0 * dt).exp());
        }
    }
}
