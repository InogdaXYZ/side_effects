#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use std::time::Duration;

use bevy::{
    gltf::Gltf,
    pbr::{ClusterConfig, ClusterFarZMode},
    prelude::*,
    scene::SceneInstance,
    window::WindowResolution,
};
use bevy_asset_loader::prelude::*;
use bevy_inspector_egui::prelude::*;
#[cfg(debug_assertions)]
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use hud::{BG_ACTION_PRIMARY, P20, P40};
use pathfinding::{
    num_traits::{Pow, ToPrimitive},
    prelude::*,
};
use rand::seq::SliceRandom;
use rand::thread_rng;

mod hud;

fn main() {
    let mut app = App::new();

    app.insert_resource(ClearColor(Color::WHITE))
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    watch_for_changes: true,
                    ..Default::default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: WindowResolution::new(1024., 768.),
                        title: "No rats were harmed".to_string(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
        )
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin {
            always_on_top: true,
            enabled: false,
            ..Default::default()
        })
        .register_type::<Settings>()
        .init_resource::<Settings>()
        .init_resource::<Medicines>()
        .init_resource::<Levels>()
        .add_state::<AppState>()
        .add_state::<GameState>()
        .add_plugin(hud::HudPlugin)
        .add_loading_state(
            LoadingState::new(AppState::Loading).continue_to_state(AppState::TitleScreen),
        )
        .add_collection_to_loading_state::<_, MyAssets>(AppState::Loading)
        .add_collection_to_loading_state::<_, Fonts>(AppState::Loading)
        // Loading
        .add_system(setup_preloader.in_schedule(OnEnter(AppState::Loading)))
        .add_systems(
            (cleanup::<Camera>, cleanup::<PreloaderPoint>).in_schedule(OnExit(AppState::Loading)),
        )
        .add_systems((
            setup_entities,
            disappearing,
            rat_moving_animation,
            rat_idle_animation,
            stink_animation,
            box_jiggle_animation,
        ))
        // Title screen
        .add_systems(
            (spawn_scene, setup_title_screen)
                .chain()
                .in_schedule(OnEnter(AppState::TitleScreen)),
        )
        .add_system(start_button.in_set(OnUpdate(AppState::TitleScreen)))
        .add_system(cleanup::<TitleScreen>.in_schedule(OnExit(AppState::TitleScreen)))
        // Planning experiment
        .add_systems(
            (cleanup::<SceneInstance>, spawn_scene)
                .chain()
                .in_schedule(OnEnter(GameState::Planning)),
        )
        .add_system(adjust_rendering.in_set(OnUpdate(GameState::Planning)))
        // Conducting experiment
        .add_systems(
            (
                cleanup::<InvisibleWalls>,
                setup_pathfinding,
                give_medicines,
                open_box,
            )
                .chain()
                .in_schedule(OnEnter(GameState::Experimenting)),
        )
        .add_systems(
            (set_goal, eat_food, rest)
                .in_set(OnUpdate(AppState::InGame))
                .in_set(OnUpdate(GameState::Experimenting)),
        );

    #[cfg(debug_assertions)]
    app.add_plugin(WorldInspectorPlugin::default());

    app.run();
}

#[derive(Resource, Debug, Reflect, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
struct Settings {
    rat_lin_speed: f32,
    rat_ang_speed: f32,
    min_distance: f32,
    max_rest_sec: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            rat_lin_speed: 4.2,
            rat_ang_speed: 20.0,
            min_distance: 0.2,
            max_rest_sec: 1.0,
        }
    }
}

#[derive(States, Clone, Hash, Eq, PartialEq, Debug, Default)]
enum AppState {
    #[default]
    Loading,
    TitleScreen,
    InGame,
}

#[derive(States, Clone, Hash, Eq, PartialEq, Debug, Default)]
pub enum GameState {
    #[default]
    Planning,
    Experimenting,
}

#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(path = "levels.gltf")]
    main_gltf: Handle<Gltf>,
}

#[derive(AssetCollection, Resource)]
pub struct Fonts {
    #[asset(path = "fonts/Fira/ttf/FiraSans-Regular.ttf")]
    regular: Handle<Font>,
    #[asset(path = "fonts/Fira/ttf/FiraSans-Bold.ttf")]
    bold: Handle<Font>,
    #[asset(path = "fonts/Fira/ttf/FiraSans-SemiBold.ttf")]
    semibold: Handle<Font>,
}

fn cleanup<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for t in query.iter() {
        commands.entity(t).despawn_recursive();
    }
}

#[derive(Resource)]
struct Levels(Vec<Level>);

#[derive(Component, Debug)]
struct Level {
    display_name: String,
    blender_name: String,
}

impl Level {
    fn new(blender_name: &str, display_name: &str) -> Self {
        Self {
            blender_name: blender_name.to_string(),
            display_name: display_name.to_string(),
        }
    }
}

impl Default for Levels {
    fn default() -> Self {
        Self(vec![Level::new("level1", "Level 1")])
    }
}

/************************************************************************************************/
/*    ########  ########  ######## ##        #######     ###    ########  ######## ########     */
/*    ##     ## ##     ## ##       ##       ##     ##   ## ##   ##     ## ##       ##     ##    */
/*    ##     ## ##     ## ##       ##       ##     ##  ##   ##  ##     ## ##       ##     ##    */
/*    ########  ########  ######   ##       ##     ## ##     ## ##     ## ######   ########     */
/*    ##        ##   ##   ##       ##       ##     ## ######### ##     ## ##       ##   ##      */
/*    ##        ##    ##  ##       ##       ##     ## ##     ## ##     ## ##       ##    ##     */
/*    ##        ##     ## ######## ########  #######  ##     ## ########  ######## ##     ##    */
/************************************************************************************************/

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

/************************************************************************************************************/
/*    ######## #### ######## ##       ########     ######   ######  ########  ######## ######## ##    ##    */
/*       ##     ##     ##    ##       ##          ##    ## ##    ## ##     ## ##       ##       ###   ##    */
/*       ##     ##     ##    ##       ##          ##       ##       ##     ## ##       ##       ####  ##    */
/*       ##     ##     ##    ##       ######       ######  ##       ########  ######   ######   ## ## ##    */
/*       ##     ##     ##    ##       ##                ## ##       ##   ##   ##       ##       ##  ####    */
/*       ##     ##     ##    ##       ##          ##    ## ##    ## ##    ##  ##       ##       ##   ###    */
/*       ##    ####    ##    ######## ########     ######   ######  ##     ## ######## ######## ##    ##    */
/************************************************************************************************************/

#[derive(Component)]
struct TitleScreen;

#[derive(Component)]
struct StartButton;

fn setup_title_screen(mut commands: Commands, fonts: Res<Fonts>) {
    commands
        .spawn((
            TitleScreen,
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        top: P40,
                        right: P20,
                        ..default()
                    },
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::End,
                    gap: Size::all(P20),
                    ..default()
                },
                ..Default::default()
            },
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::End,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn(
                        TextBundle::from_section(
                            "No rats were harmed",
                            TextStyle {
                                font: fonts.bold.clone_weak(),
                                font_size: 38.0,
                                color: Color::BLACK,
                            },
                        ) // Set the alignment of the Text
                        .with_text_alignment(TextAlignment::Center),
                    );

                    parent.spawn(
                        TextBundle::from_section(
                            "a game about observation and cheese",
                            TextStyle {
                                font: fonts.regular.clone_weak(),
                                font_size: 20.0,
                                color: Color::BLACK,
                            },
                        ) // Set the alignment of the Text
                        .with_text_alignment(TextAlignment::Center),
                    );
                });

            parent
                .spawn((
                    StartButton,
                    ButtonBundle {
                        style: hud::ACTION_BUTTON_STYLE,
                        background_color: BG_ACTION_PRIMARY.into(),
                        ..Default::default()
                    },
                ))
                .with_children(|parent| {
                    parent.spawn(
                        TextBundle::from_section(
                            "Start",
                            TextStyle {
                                font: fonts.regular.clone_weak(),
                                font_size: 30.0,
                                color: Color::WHITE,
                            },
                        ) // Set the alignment of the Text
                        .with_text_alignment(TextAlignment::Center),
                    );
                });
        });
}

fn spawn_scene(
    mut commands: Commands,
    my: Option<Res<MyAssets>>,
    assets_gltf: Res<Assets<Gltf>>,
    levels: Res<Levels>,
) {
    if let Some(my) = my {
        if let Some(gltf) = assets_gltf.get(&my.main_gltf) {
            if let Some(first_level) = levels.0.first() {
                commands.spawn((
                    Name::new(first_level.display_name.clone()),
                    SceneBundle {
                        scene: gltf.named_scenes[&first_level.blender_name].clone(),
                        ..Default::default()
                    },
                ));
            }
        }
    }
}

fn adjust_rendering(
    mut cameras: Query<&mut ClusterConfig, Added<Camera3d>>,
    mut point_lights: Query<&mut PointLight, Added<PointLight>>,
    mut spot_lights: Query<&mut SpotLight, Added<SpotLight>>,
    mut directional_lights: Query<&mut DirectionalLight, Added<DirectionalLight>>,
) {
    for mut cluster_config in cameras.iter_mut() {
        if let ClusterConfig::FixedZ { z_config, .. } = cluster_config.as_mut() {
            z_config.far_z_mode = ClusterFarZMode::Constant(0.0);
        }
    }

    let k = 683.; // don't ask me why
    for mut point_light in point_lights.iter_mut() {
        point_light.intensity /= k;
    }

    for mut spot_light in spot_lights.iter_mut() {
        spot_light.range += 100.;
        //         spot_light.intensity /= k;
    }

    for mut directional_light in directional_lights.iter_mut() {
        directional_light.illuminance *= 42.;
    }
}

fn start_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<StartButton>)>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for interaction in interaction_query.iter() {
        match interaction {
            Interaction::Clicked => next_state.set(AppState::InGame),
            Interaction::Hovered => {}
            Interaction::None => {}
        }
    }
}

/***************************************************************************/
/*    ##     ## ######## ########  ####  ######  #### ##    ## ########    */
/*    ###   ### ##       ##     ##  ##  ##    ##  ##  ###   ## ##          */
/*    #### #### ##       ##     ##  ##  ##        ##  ####  ## ##          */
/*    ## ### ## ######   ##     ##  ##  ##        ##  ## ## ## ######      */
/*    ##     ## ##       ##     ##  ##  ##        ##  ##  #### ##          */
/*    ##     ## ##       ##     ##  ##  ##    ##  ##  ##   ### ##          */
/*    ##     ## ######## ########  ####  ######  #### ##    ## ########    */
/***************************************************************************/

#[derive(Component, Debug, Default)]
pub struct Medicine {
    name: String,
    appetite: i32,
    smell: i32,
    fear: i32,
    in_experiment: bool,
    report: Report,
}

#[derive(Default, Debug, Copy, Clone)]
pub struct Report {
    appetite: i32,
    smell: i32,
    fear: i32,
}

impl Report {
    fn mark_effect(&mut self, effect: &MedicineEffect, value: i32) {
        match effect {
            MedicineEffect::Appetite => self.appetite = value,
            MedicineEffect::Smell => self.smell = value,
            MedicineEffect::Fear => self.fear = value,
        }
    }
}

impl Medicine {
    fn with_name(&self, name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..*self
        }
    }

    fn set_effect(&mut self, effect: &MedicineEffect, value: i32) {
        match effect {
            MedicineEffect::Appetite => self.appetite = value,
            MedicineEffect::Smell => self.smell = value,
            MedicineEffect::Fear => self.fear = value,
        }
    }

    pub fn report_is_correct(&self) -> bool {
        self.appetite == self.report.appetite
            && self.fear == self.report.fear
            && self.smell == self.report.smell
    }

    #[rustfmt::skip]
    pub fn report_missed_side_effects(&self) -> bool {
        (self.appetite == MedicineEffect::Appetite.negative_value() && self.appetite != self.report.appetite) ||
        (self.fear == MedicineEffect::Fear.negative_value() && self.fear != self.report.fear) ||
        (self.smell == MedicineEffect::Smell.negative_value() && self.smell != self.report.smell)
    }

    #[rustfmt::skip]
    pub fn report_extra_side_effects(&self) -> bool {
        (self.report.appetite == MedicineEffect::Appetite.negative_value() && self.appetite != self.report.appetite) ||
        (self.report.fear == MedicineEffect::Fear.negative_value() && self.fear != self.report.fear) ||
        (self.report.smell == MedicineEffect::Smell.negative_value() && self.smell != self.report.smell)
    }

    #[rustfmt::skip]
    pub fn report_missed_desirable_effects(&self) -> bool {
        (self.appetite == MedicineEffect::Appetite.positive_value() && self.appetite != self.report.appetite) ||
        (self.fear == MedicineEffect::Fear.positive_value() && self.fear != self.report.fear) ||
        (self.smell == MedicineEffect::Smell.positive_value() && self.smell != self.report.smell)
    }

    #[rustfmt::skip]
    pub fn report_extra_desirable_effects(&self) -> bool {
        (self.report.appetite == MedicineEffect::Appetite.positive_value() && self.appetite != self.report.appetite) ||
        (self.report.fear == MedicineEffect::Fear.positive_value() && self.fear != self.report.fear) ||
        (self.report.smell == MedicineEffect::Smell.positive_value() && self.smell != self.report.smell)
    }
}

#[test]
fn test_report_missed_side_effects() {
    let mut medicine = Medicine::default();
    assert!(!medicine.report_missed_side_effects());

    medicine.appetite = -1;
    assert!(medicine.report_missed_side_effects());
    medicine.report.appetite = -1;
    assert!(!medicine.report_missed_side_effects());

    medicine.fear = 1;
    assert!(medicine.report_missed_side_effects());
    medicine.report.fear = 1;
    assert!(!medicine.report_missed_side_effects());

    medicine.smell = -1;
    assert!(medicine.report_missed_side_effects());
    medicine.report.smell = -1;
    assert!(!medicine.report_missed_side_effects());
}

#[test]
fn test_report_extra_side_effects() {
    let mut medicine = Medicine::default();
    assert!(!medicine.report_extra_side_effects());

    medicine.report.appetite = -1;
    assert!(medicine.report_extra_side_effects());
    medicine.appetite = -1;
    assert!(!medicine.report_extra_side_effects());

    medicine.report.fear = 1;
    assert!(medicine.report_extra_side_effects());
    medicine.fear = 1;
    assert!(!medicine.report_extra_side_effects());

    medicine.report.smell = -1;
    assert!(medicine.report_extra_side_effects());
    medicine.smell = -1;
    assert!(!medicine.report_extra_side_effects());
}

#[test]
fn test_report_missed_desirable_effects() {
    let mut medicine = Medicine::default();
    assert!(!medicine.report_missed_desirable_effects());

    medicine.appetite = 1;
    assert!(medicine.report_missed_desirable_effects());
    medicine.report.appetite = 1;
    assert!(!medicine.report_missed_desirable_effects());

    medicine.fear = -1;
    assert!(medicine.report_missed_desirable_effects());
    medicine.report.fear = -1;
    assert!(!medicine.report_missed_desirable_effects());

    medicine.smell = 1;
    assert!(medicine.report_missed_desirable_effects());
    medicine.report.smell = 1;
    assert!(!medicine.report_missed_desirable_effects());
}

#[test]
fn test_report_extra_desirable_effects() {
    let mut medicine = Medicine::default();
    assert!(!medicine.report_extra_desirable_effects());

    medicine.report.appetite = 1;
    assert!(medicine.report_extra_desirable_effects());
    medicine.appetite = 1;
    assert!(!medicine.report_extra_desirable_effects());

    medicine.report.fear = -1;
    assert!(medicine.report_extra_desirable_effects());
    medicine.fear = -1;
    assert!(!medicine.report_extra_desirable_effects());

    medicine.report.smell = 1;
    assert!(medicine.report_extra_desirable_effects());
    medicine.smell = 1;
    assert!(!medicine.report_extra_desirable_effects());
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MedicineEffect {
    Appetite,
    Fear,
    Smell,
}

impl MedicineEffect {
    #[cfg(debug_assertions)]
    fn dev_title(&self) -> &str {
        match self {
            MedicineEffect::Appetite => "Appetite",
            MedicineEffect::Fear => "Fear",
            MedicineEffect::Smell => "Smell",
        }
    }

    fn positive_title(&self) -> &str {
        match self {
            MedicineEffect::Appetite => "Promotes healthy appetite",
            MedicineEffect::Fear => "Lowers anxiety",
            MedicineEffect::Smell => "Enhances senses",
        }
    }

    fn positive_value(&self) -> i32 {
        match self {
            MedicineEffect::Appetite => 1,
            MedicineEffect::Fear => -1,
            MedicineEffect::Smell => 1,
        }
    }

    fn negative_title(&self) -> &str {
        match self {
            MedicineEffect::Appetite => "Causes loss of appetite",
            MedicineEffect::Fear => "Increases anxiety",
            MedicineEffect::Smell => "Loss of smell",
        }
    }

    fn negative_value(&self) -> i32 {
        match self {
            MedicineEffect::Appetite => -1,
            MedicineEffect::Fear => 1,
            MedicineEffect::Smell => -1,
        }
    }
}

#[derive(Resource)]
pub struct Medicines(Vec<Medicine>);

impl Default for Medicines {
    fn default() -> Self {
        let mut a = Medicine::default().with_name("A");
        a.set_effect(&MedicineEffect::Appetite, -1);
        a.set_effect(&MedicineEffect::Fear, 1);

        let mut b = Medicine::default().with_name("B");
        b.set_effect(&MedicineEffect::Appetite, 1);
        b.set_effect(&MedicineEffect::Smell, -1);

        let mut c = Medicine::default().with_name("C");
        c.set_effect(&MedicineEffect::Appetite, 1);
        c.set_effect(&MedicineEffect::Smell, 1);
        c.set_effect(&MedicineEffect::Fear, -1);

        Medicines(vec![a, b, c])
    }
}

impl Medicines {
    pub fn all_reports_are_correct(&self) -> bool {
        self.0.iter().all(|m| m.report_is_correct())
    }

    pub fn some_reports_have_extra_side_effects(&self) -> bool {
        self.0.iter().any(|m| m.report_extra_side_effects())
    }

    pub fn some_reports_have_missed_side_effects(&self) -> bool {
        self.0.iter().any(|m| m.report_missed_side_effects())
    }

    pub fn some_reports_have_extra_desirable_effects(&self) -> bool {
        self.0.iter().any(|m| m.report_extra_desirable_effects())
    }

    pub fn some_reports_have_missed_desirable_effects(&self) -> bool {
        self.0.iter().any(|m| m.report_missed_desirable_effects())
    }
}

/*************************************************************************/
/*    ######## ##    ## ######## #### ######## #### ########  ######     */
/*    ##       ###   ##    ##     ##     ##     ##  ##       ##    ##    */
/*    ##       ####  ##    ##     ##     ##     ##  ##       ##          */
/*    ######   ## ## ##    ##     ##     ##     ##  ######    ######     */
/*    ##       ##  ####    ##     ##     ##     ##  ##             ##    */
/*    ##       ##   ###    ##     ##     ##     ##  ##       ##    ##    */
/*    ######## ##    ##    ##    ####    ##    #### ########  ######     */
/*************************************************************************/

#[derive(Component)]
struct Cheese;

#[derive(Component, Debug)]
struct Mouldy;

#[derive(Component, Debug)]
struct Stink;

#[derive(Component, Debug)]
struct CartonBox;

#[derive(Component, Debug)]
struct ScareCat;

fn setup_entities(mut commands: Commands, named_entities: Query<(Entity, &Name), Added<Name>>) {
    for (entity, name) in named_entities.iter() {
        if name.starts_with("rat.") {
            // 0.5 x 0.2 x 0.1
            let collider = commands
                .spawn((
                    Collider::capsule_z(0.25, 0.1),
                    TransformBundle::from_transform(Transform::from_xyz(0., 0.1, 0.)),
                ))
                .id();
            commands
                .entity(entity)
                .insert((
                    Rat::default(),
                    RememberedObstacles::default(),
                    Rest(Timer::from_seconds(0.5, TimerMode::Once)),
                    RigidBody::Dynamic,
                    KinematicCharacterController::default(),
                    LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
                ))
                .add_child(collider);
        }

        if name.starts_with("cheese.") || name.starts_with("mouldy-cheese.") {
            let sensor = commands
                .spawn((
                    Collider::cylinder(0.25, 0.25),
                    Sensor,
                    ActiveCollisionTypes::default(),
                    ActiveEvents::COLLISION_EVENTS,
                    TransformBundle::from_transform(Transform::from_xyz(0., 0.25, 0.)),
                ))
                .id();
            commands
                .entity(entity)
                .insert((Cheese, RigidBody::Fixed))
                .add_child(sensor);
            if name.starts_with("mouldy-cheese.") {
                commands.entity(entity).insert(Mouldy);
            }
        }

        if name.starts_with("stink") {
            commands.entity(entity).insert(Stink);
        }

        if name.starts_with("tile.") {
            let collider = commands
                .spawn((
                    Collider::cuboid(0.5, 0.065, 0.5),
                    TransformBundle::from_transform(Transform::from_xyz(0., -0.065, 0.)),
                ))
                .id();
            commands
                .entity(entity)
                .insert((Tile, RigidBody::Fixed))
                .add_child(collider);
        }

        if name.starts_with("box.") {
            commands.entity(entity).insert(CartonBox);
        }

        if name.starts_with("scarecat.") {
            commands.entity(entity).insert(ScareCat);
        }
    }
}

/*********************************************************************************************************/
/*    ########     ###    ######## ##     ## ######## #### ##    ## ########  #### ##    ##  ######      */
/*    ##     ##   ## ##      ##    ##     ## ##        ##  ###   ## ##     ##  ##  ###   ## ##    ##     */
/*    ##     ##  ##   ##     ##    ##     ## ##        ##  ####  ## ##     ##  ##  ####  ## ##           */
/*    ########  ##     ##    ##    ######### ######    ##  ## ## ## ##     ##  ##  ## ## ## ##   ####    */
/*    ##        #########    ##    ##     ## ##        ##  ##  #### ##     ##  ##  ##  #### ##    ##     */
/*    ##        ##     ##    ##    ##     ## ##        ##  ##   ### ##     ##  ##  ##   ### ##    ##     */
/*    ##        ##     ##    ##    ##     ## ##       #### ##    ## ########  #### ##    ##  ######      */
/*********************************************************************************************************/

#[derive(Debug, Resource)]
struct PathfindingMatrix {
    grid: Grid,
    min_x: i32,
    min_z: i32,
    dx: f32,
    dz: f32,
}

impl PathfindingMatrix {
    fn from_coordinates(tiles: &[&Transform]) -> Self {
        let mut dx = 0.0;
        let mut dz = 0.0;
        let tile_coords: Vec<(i32, i32)> = tiles
            .iter()
            .map(|transform| {
                dx = transform.translation.x - transform.translation.x.round();
                dz = transform.translation.z - transform.translation.z.round();
                (
                    transform.translation.x.round() as i32,
                    transform.translation.z.round() as i32,
                )
            })
            .collect::<_>();

        let xs = tile_coords.iter().map(|(x, _)| x);
        let zs = tile_coords.iter().map(|(_, y)| y);
        let min_x = xs.min().unwrap_or(&0);
        let min_z = zs.min().unwrap_or(&0);

        let original_grid = Grid::from_coordinates(&tile_coords).unwrap_or(Grid::new(0, 0));
        let mut grid = Grid::new(original_grid.width + 2, original_grid.height + 2);
        for coord in original_grid.iter() {
            grid.add_vertex((coord.0 + 1, coord.1 + 1));
        }

        PathfindingMatrix {
            grid,
            min_x: *min_x,
            min_z: *min_z,
            dx,
            dz,
        }
    }

    fn grid_coord(&self, translation: Vec3) -> Option<(usize, usize)> {
        match (
            ((translation.x - self.dx).round() as i32 - self.min_x + 1).to_usize(),
            ((translation.z - self.dz).round() as i32 - self.min_z + 1).to_usize(),
        ) {
            (Some(x), Some(z)) => Some((x, z)),
            _ => None,
        }
    }

    fn translation(&self, coord: &(usize, usize), y: f32) -> Vec3 {
        let x = (self.min_x - 1 + coord.0 as i32) as f32 + self.dx;
        let z = (self.min_z - 1 + coord.1 as i32) as f32 + self.dz;
        Vec3::new(x, y, z)
    }

    fn without(&self, avoided: &[(usize, usize)]) -> Self {
        let mut grid = self.grid.clone();
        for coord in avoided {
            grid.remove_vertex(*coord);
        }
        Self { grid, ..*self }
    }
}

#[test]
fn test_pathfinding_coord_conversion() {
    let translation = Vec3::new(1.5, 0.0, -2.5);
    let coords: Vec<Transform> = vec![
        Vec3::new(0.5, 0.0, -0.5),
        Vec3::new(0.5, 0.0, -1.5),
        Vec3::new(0.5, 0.0, -2.5),
        Vec3::new(1.5, 0.0, -2.5),
    ]
    .iter()
    .map(|coord| Transform::from_translation(*coord))
    .collect();
    let pathfinding = PathfindingMatrix::from_coordinates(&coords.iter().collect::<Vec<_>>());
    let grid_coord = pathfinding.grid_coord(translation);
    assert_eq!(grid_coord, Some((2, 1)));
    if let Some(coord) = grid_coord {
        assert_eq!(pathfinding.translation(&coord, 0.0), translation);
    }
}

#[test]
fn test_pathfinding_coord_rounding() {
    let translation = Vec3::new(0.9, 0.0, -2.1);
    let coords: Vec<Transform> = vec![
        Vec3::new(0.0, 0.0, -0.0),
        Vec3::new(0.0, 0.0, -1.0),
        Vec3::new(0.0, 0.0, -2.0),
        Vec3::new(1.0, 0.0, -2.0),
    ]
    .iter()
    .map(|coord| Transform::from_translation(*coord))
    .collect();
    let pathfinding = PathfindingMatrix::from_coordinates(&coords.iter().collect::<Vec<_>>());
    let grid_coord = pathfinding.grid_coord(translation);
    assert_eq!(grid_coord, Some((2, 1)));
    if let Some(coord) = grid_coord {
        assert_eq!(
            pathfinding.translation(&coord, 0.0),
            Vec3::new(1.0, 0.0, -2.0)
        );
    }
}

#[derive(Component, Debug)]
struct InvisibleWalls;

#[derive(Component, Debug)]
struct Tile;

fn setup_pathfinding(mut commands: Commands, tiles: Query<&Transform, With<Tile>>) {
    let pathfinding = PathfindingMatrix::from_coordinates(&tiles.iter().collect::<Vec<_>>());

    // Add invisible wall colliders
    let mut inverted_grid = pathfinding.grid.clone();
    inverted_grid.invert();
    commands
        .spawn((
            Name::new("Invisible walls"),
            InvisibleWalls,
            TransformBundle::default(),
        ))
        .with_children(|parent| {
            for coord in inverted_grid {
                parent.spawn((
                    RigidBody::Fixed,
                    Collider::cuboid(0.5, 0.5, 0.5),
                    TransformBundle::from_transform(Transform::from_translation(
                        pathfinding.translation(&coord, 0.5),
                    )),
                ));
            }
        });

    commands.insert_resource(pathfinding);
}

/**************************************/
/*    ########     ###    ########    */
/*    ##     ##   ## ##      ##       */
/*    ##     ##  ##   ##     ##       */
/*    ########  ##     ##    ##       */
/*    ##   ##   #########    ##       */
/*    ##    ##  ##     ##    ##       */
/*    ##     ## ##     ##    ##       */
/**************************************/

#[derive(Component, Copy, Clone)]
struct Rat {
    appetite: i32,
    smell: i32,
    fear: i32,
}

impl Default for Rat {
    fn default() -> Self {
        Rat {
            appetite: 1,
            smell: 1,
            fear: 1,
        }
    }
}

impl Rat {
    fn with_medicines(&self, medicines: &[Medicine]) -> Self {
        let mut new = *self;
        for medicine in medicines {
            if medicine.in_experiment {
                new.appetite += medicine.appetite;
                new.smell += medicine.smell;
                new.fear += medicine.fear;
            }
        }
        new.appetite = new.appetite.clamp(0, 2);
        new.smell = new.smell.clamp(0, 2);
        new.fear = new.fear.clamp(0, 2);
        new
    }

    fn velocity(from: &Transform, to: &Vec3, settings: &Settings) -> Velocity {
        let current_forward = from.rotation.mul_vec3(Vec3::Z * -1.).normalize();
        let desired_forward = (*to - from.translation).normalize_or_zero();

        let linvel = desired_forward * settings.rat_lin_speed;

        let rotation_axis = current_forward.cross(desired_forward).normalize();
        let rotation_angle = current_forward.angle_between(desired_forward);
        let angvel = if rotation_axis.is_finite() {
            rotation_axis * rotation_angle * settings.rat_ang_speed
        } else {
            Vec3::ZERO
        };
        Velocity { linvel, angvel }
    }
}

fn give_medicines(mut rats: Query<&mut Rat>, medicines: Res<Medicines>) {
    for mut rat in rats.iter_mut() {
        *rat = Rat::default().with_medicines(&medicines.0);
    }
}

#[derive(Component, Debug)]
struct Rest(Timer);

fn rest(mut commands: Commands, mut resting: Query<(Entity, &mut Rest)>, time: Res<Time>) {
    for (entity, mut rest) in resting.iter_mut() {
        if rest.0.tick(time.delta()).just_finished() {
            commands.entity(entity).remove::<Rest>();
        }
    }
}

#[derive(Component, Debug)]
struct Panic {
    coord: (usize, usize),
    timer: Timer,
}

#[derive(Component, Debug)]
struct Goal((usize, usize));

#[derive(Component, Debug, Default)]
struct RememberedObstacles(Vec<(usize, usize)>);

fn set_goal(
    mut commands: Commands,
    mut rats: Query<
        (
            Entity,
            &Rat,
            &Transform,
            Option<&Goal>,
            Option<&mut Panic>,
            &mut RememberedObstacles,
        ),
        Without<Rest>,
    >,
    cheese: Query<(&Transform, Option<&Mouldy>), (With<Cheese>, Without<Rat>)>,
    scare_cats: Query<&Transform, With<ScareCat>>,
    pathfinding: Res<PathfindingMatrix>,
    settings: Res<Settings>,
    time: Res<Time>,
) {
    for (rat_entity, rat, rat_transform, goal, mut panic, mut remembered_obstacles) in
        rats.iter_mut()
    {
        if let Some(ref mut panic) = panic {
            panic.timer.tick(time.delta());
        }

        let rat_coord = pathfinding
            .grid_coord(rat_transform.translation)
            .unwrap_or((0, 0));

        let mut avoided: Vec<(usize, usize)> = cheese
            .iter()
            .filter_map(|(transform, mouldy)| {
                if rat.smell > 0 && mouldy.is_some() {
                    pathfinding.grid_coord(transform.translation)
                } else {
                    None
                }
            })
            .collect();
        avoided.extend_from_slice(&remembered_obstacles.0);

        if let Some(goal) = goal {
            let goal_translation = pathfinding.translation(&goal.0, rat_transform.translation.y);
            if goal_translation.distance(rat_transform.translation) > settings.min_distance {
                let velocity = Rat::velocity(rat_transform, &goal_translation, &settings);
                commands.entity(rat_entity).insert(velocity);
                continue;
            } else {
                commands
                    .entity(rat_entity)
                    .insert(Velocity::zero())
                    .remove::<Goal>();

                if let Some(panic) = panic {
                    if panic.timer.finished() {
                        commands.entity(rat_entity).remove::<Panic>();
                    } else {
                        // Choose direction
                        let mut paths = pathfinding
                            .grid
                            .iter()
                            .filter_map(|goal| {
                                astar(
                                    &panic.coord,
                                    |p| {
                                        pathfinding
                                            .without(&avoided)
                                            .grid
                                            .neighbours(*p)
                                            .into_iter()
                                            .map(|p| (p, 1))
                                            .collect::<Vec<_>>()
                                    },
                                    |p| pathfinding.grid.distance(*p, goal) / 3,
                                    |p| *p == goal,
                                )
                            })
                            .collect::<Vec<_>>();
                        paths.sort_by_key(|(_path, distance)| *distance);
                        paths.reverse();

                        let paths_to_safe_space = paths
                            .into_iter()
                            .filter_map(|(path, _)| {
                                path.last().and_then(|goal| {
                                    astar(
                                        &rat_coord,
                                        |p| {
                                            pathfinding
                                                .without(&avoided)
                                                .grid
                                                .neighbours(*p)
                                                .into_iter()
                                                .map(|p| (p, 1))
                                                .collect::<Vec<_>>()
                                        },
                                        |p| pathfinding.grid.distance(*p, *goal) / 3,
                                        |p| *p == *goal,
                                    )
                                })
                            })
                            .take(10)
                            .collect::<Vec<_>>();

                        let mut rng = thread_rng();
                        let path_to_safe_space = paths_to_safe_space.choose(&mut rng);

                        if let Some((path, _)) = path_to_safe_space {
                            let destination = match *path.as_slice() {
                                [_, second, ..] => Some(second),
                                [first, ..] => Some(first),
                                [] => None,
                            };

                            if let Some(destination) = destination {
                                commands.entity(rat_entity).insert(Goal(destination));
                                continue;
                            }
                        }
                    }
                }

                if rat.fear > 0 {
                    // Check panic
                    let scary_tiles = scare_cats
                        .iter()
                        .filter_map(|transform| pathfinding.grid_coord(transform.translation))
                        .collect::<Vec<_>>();
                    let neighbors = pathfinding.grid.neighbours(rat_coord);
                    let closest_scare_cat = neighbors.iter().find(|n| scary_tiles.contains(n));
                    if let Some(panic_coord) = closest_scare_cat {
                        // Panic
                        remembered_obstacles.0.push(*panic_coord);
                        commands.entity(rat_entity).insert(Panic {
                            coord: *panic_coord,
                            timer: Timer::from_seconds(
                                1.0 * (rat.fear as f32).pow(2.),
                                TimerMode::Once,
                            ),
                        });
                        continue;
                    }
                }
                // Rest
                commands.entity(rat_entity).insert(Rest(Timer::from_seconds(
                    match rat.appetite {
                        0 => settings.max_rest_sec,
                        1 => settings.max_rest_sec * 0.3,
                        _ => 0.0,
                    },
                    TimerMode::Once,
                )));
                continue;
            }
        }

        if rat.appetite > 0 {
            /****************************************************************/
            /*     ######  ##     ## ######## ########  ######  ########    */
            /*    ##    ## ##     ## ##       ##       ##    ## ##          */
            /*    ##       ##     ## ##       ##       ##       ##          */
            /*    ##       ######### ######   ######    ######  ######      */
            /*    ##       ##     ## ##       ##             ## ##          */
            /*    ##    ## ##     ## ##       ##       ##    ## ##          */
            /*     ######  ##     ## ######## ########  ######  ########    */
            /****************************************************************/
            let smell_distance = 5.0 * rat.smell as f32;
            let mut paths = cheese
                .iter()
                .filter_map(|(transform, _mouldy)| {
                    pathfinding
                        .grid_coord(transform.translation)
                        .and_then(|goal| {
                            let distance =
                                rat_transform.translation.distance(transform.translation);
                            let walkable_grid = pathfinding.without(&avoided).grid;
                            let path = astar(
                                &rat_coord,
                                |p| {
                                    walkable_grid
                                        .neighbours(*p)
                                        .into_iter()
                                        .map(|p| (p, 1))
                                        .collect::<Vec<_>>()
                                },
                                |p| pathfinding.grid.distance(*p, goal) / 3,
                                |p| *p == goal,
                            );

                            if distance < smell_distance {
                                path.map(|(path, _)| (path, distance.round() as i32))
                            } else {
                                None
                            }
                        })
                })
                .collect::<Vec<_>>();

            paths.sort_by_key(|(_, prio)| *prio);

            if let Some((path, _smell_intencity)) = paths.first() {
                let destination = match *path.as_slice() {
                    [_, second, ..] => Some(second),
                    [first, ..] => Some(first),
                    [] => None,
                };

                if let Some(destination) = destination {
                    commands.entity(rat_entity).insert(Goal(destination));
                    continue;
                }
            }
        }
        // Roam
        {
            let mut rng = thread_rng();
            let neighbors = pathfinding.without(&avoided).grid.neighbours(rat_coord);
            let destination = neighbors.choose(&mut rng);
            if let Some(destination) = destination {
                commands.entity(rat_entity).insert(Goal(*destination));
                continue;
            }
        }
    }
}

fn eat_food(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut rats: Query<(Entity, &mut Rat)>,
    cheeses: Query<Entity, (With<Cheese>, Without<Disappearing>)>,
    children: Query<&Parent, With<Collider>>,
) {
    for collision_event in collision_events.iter() {
        match collision_event {
            CollisionEvent::Started(first, second, _flags) => {
                if let (Some(a), Some(b)) = (children.get(*first).ok(), children.get(*second).ok())
                {
                    let mut rat_entity = None;
                    let mut cheese_entity = None;

                    for (entity, _) in rats.iter() {
                        if entity == a.get() || entity == b.get() {
                            rat_entity = Some(entity);
                            break;
                        }
                    }

                    for entity in cheeses.iter() {
                        if entity == a.get() || entity == b.get() {
                            cheese_entity = Some(entity);
                            break;
                        }
                    }

                    if let (Some(rat), Some(cheese)) = (rat_entity, cheese_entity) {
                        let (_, mut rat) = rats.get_mut(rat).unwrap();
                        if rat.appetite > 0 {
                            // Eat cheese
                            rat.appetite = (rat.appetite - 1).clamp(0, 2);
                            commands.entity(cheese).insert(Disappearing {
                                effect: DisappearingEffect::ScaleToNothing,
                                ..default()
                            });
                        }
                    }
                }
            }
            CollisionEvent::Stopped(_, _, _) => {}
        }
    }
}

/**************************************************************************************/
/*       ###    ##    ## #### ##     ##    ###    ######## ####  #######  ##    ##    */
/*      ## ##   ###   ##  ##  ###   ###   ## ##      ##     ##  ##     ## ###   ##    */
/*     ##   ##  ####  ##  ##  #### ####  ##   ##     ##     ##  ##     ## ####  ##    */
/*    ##     ## ## ## ##  ##  ## ### ## ##     ##    ##     ##  ##     ## ## ## ##    */
/*    ######### ##  ####  ##  ##     ## #########    ##     ##  ##     ## ##  ####    */
/*    ##     ## ##   ###  ##  ##     ## ##     ##    ##     ##  ##     ## ##   ###    */
/*    ##     ## ##    ## #### ##     ## ##     ##    ##    ####  #######  ##    ##    */
/**************************************************************************************/

#[derive(Component, Debug)]
struct Disappearing {
    timer: Timer,
    effect: DisappearingEffect,
}

impl Default for Disappearing {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(1.0, TimerMode::Once),
            effect: DisappearingEffect::NoEffect,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum DisappearingEffect {
    ScaleToNothing,
    NoEffect,
}

fn disappearing(
    mut commands: Commands,
    mut entites: Query<(Entity, &mut Disappearing, &mut Transform)>,
    time: Res<Time>,
) {
    for (entity, mut disappearing, mut transform) in entites.iter_mut() {
        if disappearing.timer.tick(time.delta()).just_finished() {
            commands.entity(entity).despawn_recursive();
        } else if disappearing.effect == DisappearingEffect::ScaleToNothing {
            let left = disappearing.timer.percent_left();
            transform.scale = Vec3::new(left, left, left);
        }
    }
}

fn open_box(
    mut commands: Commands,
    mut boxes: Query<(Entity, &mut AnimationPlayer), With<CartonBox>>,
    my: Option<Res<MyAssets>>,
    assets_gltf: Res<Assets<Gltf>>,
    animation_clips: Res<Assets<AnimationClip>>,
) {
    if let Some(my) = my {
        if let Some(gltf) = assets_gltf.get(&my.main_gltf) {
            for (box_entity, mut animation_player) in boxes.iter_mut() {
                let anim = &gltf.named_animations["anim-box-open"];
                animation_player.start(anim.clone()).stop_repeating();

                if let Some(clip) = animation_clips.get(anim) {
                    commands.entity(box_entity).insert(Disappearing {
                        timer: Timer::from_seconds(clip.duration(), TimerMode::Once),
                        ..Default::default()
                    });
                }
            }
        }
    }
}

fn box_jiggle_animation(
    mut boxes: Query<&mut AnimationPlayer, Added<CartonBox>>,
    my: Option<Res<MyAssets>>,
    assets_gltf: Res<Assets<Gltf>>,
) {
    if let Some(my) = my {
        for mut animation_player in boxes.iter_mut() {
            if let Some(gltf) = assets_gltf.get(&my.main_gltf) {
                let anim = &gltf.named_animations["anim-box-jiggle"];
                animation_player
                    .play_with_transition(anim.clone_weak(), Duration::from_millis(100))
                    .repeat();
            }
        }
    }
}

fn rat_moving_animation(
    mut rats: Query<&mut AnimationPlayer, (With<Rat>, Added<Goal>)>,
    my: Option<Res<MyAssets>>,
    assets_gltf: Res<Assets<Gltf>>,
) {
    if let Some(my) = my {
        for mut animation_player in rats.iter_mut() {
            if let Some(gltf) = assets_gltf.get(&my.main_gltf) {
                let anim = &gltf.named_animations["anim-rat-run-cycle"];
                animation_player
                    .play_with_transition(anim.clone_weak(), Duration::from_millis(100))
                    .repeat();
            }
        }
    }
}

fn rat_idle_animation(
    mut rats: Query<&mut AnimationPlayer, (With<Rat>, Added<Rest>)>,
    my: Option<Res<MyAssets>>,
    assets_gltf: Res<Assets<Gltf>>,
) {
    if let Some(my) = my {
        for mut animation_player in rats.iter_mut() {
            if let Some(gltf) = assets_gltf.get(&my.main_gltf) {
                let anim = &gltf.named_animations["anim-rat-idle"];
                animation_player
                    .play_with_transition(anim.clone_weak(), Duration::from_millis(100))
                    .repeat();
            }
        }
    }
}

fn stink_animation(
    mut stinks: Query<&mut AnimationPlayer, Added<Mouldy>>,
    my: Option<Res<MyAssets>>,
    assets_gltf: Res<Assets<Gltf>>,
) {
    if let Some(my) = my {
        for mut animation_player in stinks.iter_mut() {
            if let Some(gltf) = assets_gltf.get(&my.main_gltf) {
                let anim = &gltf.named_animations["anim-stink"];
                animation_player
                    .play_with_transition(anim.clone_weak(), Duration::from_millis(100))
                    .repeat();
            }
        }
    }
}
