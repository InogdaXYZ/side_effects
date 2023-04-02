#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::{gltf::Gltf, prelude::*, render::camera::ScalingMode};
use bevy_asset_loader::prelude::*;
use bevy_inspector_egui::{prelude::*, quick::WorldInspectorPlugin};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::WHITE))
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            watch_for_changes: true,
            ..Default::default()
        }))
        .add_plugin(WorldInspectorPlugin::default())
        .register_type::<Settings>()
        .init_resource::<Settings>()
        .add_state::<AppState>()
        .add_loading_state(
            LoadingState::new(AppState::Loading).continue_to_state(AppState::TitleScreen),
        )
        .add_collection_to_loading_state::<_, MyAssets>(AppState::Loading)
        .add_collection_to_loading_state::<_, MyFonts>(AppState::Loading)
        .add_system(setup_preloader.in_schedule(OnEnter(AppState::Loading)))
        .add_systems(
            (cleanup::<Camera>, cleanup::<PreloaderPoint>).in_schedule(OnExit(AppState::Loading)),
        )
        .add_system(setup_title_screen.in_schedule(OnEnter(AppState::TitleScreen)))
        .add_system(adjust_rendering.in_set(OnUpdate(AppState::TitleScreen)))
        .run();
}

#[derive(Resource, Default, Debug, Reflect, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
struct Settings;

#[derive(States, Clone, Hash, Eq, PartialEq, Debug, Default)]
enum AppState {
    #[default]
    Loading,
    TitleScreen,
    InGame,
}

#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(path = "temp-assets.gltf")]
    main_gltf: Handle<Gltf>,
}

#[derive(AssetCollection, Resource)]
struct MyFonts {
    #[asset(path = "fonts/Fira/ttf/FiraSans-Regular.ttf")]
    fira_sans_regular: Handle<Font>,
}

#[derive(Component)]
struct PreloaderPoint(usize);

fn setup_preloader(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, -5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    let cube = meshes.add(Mesh::from(shape::Cube::new(0.1)));
    let black_material = materials.add(Color::BLACK.into());

    commands.spawn((
        PreloaderPoint(0),
        PbrBundle {
            mesh: cube.clone_weak(),
            material: black_material.clone_weak(),
            transform: Transform::from_xyz(-0.4, 0.0, 0.0),
            ..Default::default()
        },
    ));
    commands.spawn((
        PreloaderPoint(1),
        PbrBundle {
            mesh: cube.clone_weak(),
            material: black_material.clone_weak(),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..Default::default()
        },
    ));
    commands.spawn((
        PreloaderPoint(2),
        PbrBundle {
            mesh: cube.clone_weak(),
            material: black_material.clone_weak(),
            transform: Transform::from_xyz(0.4, 0.0, 0.0),
            ..Default::default()
        },
    ));
}

fn setup_title_screen(
    mut commands: Commands,
    fonts: Res<MyFonts>,
    my: Res<MyAssets>,
    assets_gltf: Res<Assets<Gltf>>,
) {
    if let Some(gltf) = assets_gltf.get(&my.main_gltf) {
        commands.spawn(SceneBundle {
            scene: gltf.scenes[0].clone(),
            ..Default::default()
        });
    }

    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    bottom: Val::Percent(50.0),
                    right: Val::Px(50.0),
                    ..default()
                },
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn(
                // Create a TextBundle that has a Text with a single section.
                TextBundle::from_section(
                    // Accepts a `String` or any type that converts into a `String`, such as `&str`
                    "Side effects",
                    TextStyle {
                        font: fonts.fira_sans_regular.clone_weak(),
                        font_size: 100.0,
                        color: Color::BLACK,
                    },
                ) // Set the alignment of the Text
                .with_text_alignment(TextAlignment::Center),
            );

            parent.spawn(
                // Create a TextBundle that has a Text with a single section.
                TextBundle::from_section(
                    // Accepts a `String` or any type that converts into a `String`, such as `&str`
                    "by Roman and Christina",
                    TextStyle {
                        font: fonts.fira_sans_regular.clone_weak(),
                        font_size: 30.0,
                        color: Color::BLACK,
                    },
                ) // Set the alignment of the Text
                .with_text_alignment(TextAlignment::Center),
            );

            parent.spawn(
                // Create a TextBundle that has a Text with a single section.
                TextBundle::from_section(
                    // Accepts a `String` or any type that converts into a `String`, such as `&str`
                    "built with Bevy engine; fonts from Mozilla",
                    TextStyle {
                        font: fonts.fira_sans_regular.clone_weak(),
                        font_size: 20.0,
                        color: Color::BLACK,
                    },
                ) // Set the alignment of the Text
                .with_text_alignment(TextAlignment::Center),
            );

            parent
                .spawn(NodeBundle {
                    style: Style {
                        justify_content: JustifyContent::Center,
                        padding: UiRect::all(Val::Px(20.)),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent
                        .spawn(ButtonBundle {
                            background_color: Color::GRAY.into(),
                            ..Default::default()
                        })
                        .with_children(|parent| {
                            parent.spawn(
                                // Create a TextBundle that has a Text with a single section.
                                TextBundle::from_section(
                                    // Accepts a `String` or any type that converts into a `String`, such as `&str`
                                    "Start",
                                    TextStyle {
                                        font: fonts.fira_sans_regular.clone_weak(),
                                        font_size: 30.0,
                                        color: Color::BLACK,
                                    },
                                ) // Set the alignment of the Text
                                .with_text_alignment(TextAlignment::Center),
                            );
                        });
                });
        });
}

fn adjust_rendering(
    mut cameras: Query<&mut Projection, Added<Camera3d>>,
    mut point_lights: Query<&mut PointLight, Added<PointLight>>,
    mut spot_lights: Query<&mut SpotLight, Added<SpotLight>>,
    mut directional_lights: Query<&mut DirectionalLight, Added<DirectionalLight>>,
) {
    for mut projection in cameras.iter_mut() {
        if let Projection::Orthographic(orthographic_projection) = projection.as_mut() {
            orthographic_projection.scaling_mode = ScalingMode::WindowSize(100.0);
            orthographic_projection.scale = 1.0;
        }
    }

    let k = 683.; // don't ask me why
    for mut point_light in point_lights.iter_mut() {
        point_light.intensity = point_light.intensity / k;
    }

    for mut spot_light in spot_lights.iter_mut() {
        spot_light.intensity = spot_light.intensity / k;
    }

    for mut directional_light in directional_lights.iter_mut() {
        directional_light.illuminance = directional_light.illuminance * 6.28;
    }
}

fn cleanup<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for t in query.iter() {
        commands.entity(t).despawn_recursive();
    }
}
