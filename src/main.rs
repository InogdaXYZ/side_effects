#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use std::f32::consts::TAU;

use bevy::{gltf::Gltf, prelude::*, render::camera::ScalingMode, scene::SceneInstance};
use bevy_asset_loader::prelude::*;
use bevy_inspector_egui::{prelude::*, quick::WorldInspectorPlugin};
use bevy_rapier3d::prelude::*;
use pathfinding::prelude::*;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::WHITE))
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            watch_for_changes: true,
            ..Default::default()
        }))
        .add_plugin(WorldInspectorPlugin::default())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin {
            always_on_top: true,
            enabled: true,

            ..Default::default()
        })
        .register_type::<Settings>()
        .init_resource::<Settings>()
        .init_resource::<Medicines>()
        .add_state::<AppState>()
        .add_state::<GameState>()
        .add_loading_state(
            LoadingState::new(AppState::Loading).continue_to_state(AppState::TitleScreen),
        )
        .add_collection_to_loading_state::<_, MyAssets>(AppState::Loading)
        .add_collection_to_loading_state::<_, MyFonts>(AppState::Loading)
        // Loading
        .add_system(setup_preloader.in_schedule(OnEnter(AppState::Loading)))
        .add_systems(
            (cleanup::<Camera>, cleanup::<PreloaderPoint>).in_schedule(OnExit(AppState::Loading)),
        )
        // Title screen
        .add_systems(
            (spawn_scene, setup_title_screen)
                .chain()
                .in_schedule(OnEnter(AppState::TitleScreen)),
        )
        .add_system(start_button.in_set(OnUpdate(AppState::TitleScreen)))
        .add_system(cleanup::<TitleScreen>.in_schedule(OnExit(AppState::TitleScreen)))
        // In game
        .add_system(setup_hud.in_schedule(OnEnter(AppState::InGame)))
        .add_systems(
            (
                medicine_property_button,
                test_medicine_button,
                experiment_button,
            )
                .in_set(OnUpdate(AppState::InGame)),
        )
        .add_system(cleanup::<HUD>.in_schedule(OnExit(AppState::InGame)))
        // Planning experiment
        .add_systems(
            (cleanup::<SceneInstance>, spawn_scene)
                .chain()
                .in_schedule(OnEnter(GameState::Planning)),
        )
        .add_system(adjust_rendering.in_set(OnUpdate(GameState::Planning)))
        // Conducting experiment
        .add_systems(
            (setup_pathfinding, setup_entities).in_schedule(OnEnter(GameState::Experimenting)),
        )
        .add_system(
            find_cheese
                .in_set(OnUpdate(AppState::InGame))
                .in_set(OnUpdate(GameState::Experimenting)),
        )
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

#[derive(States, Clone, Hash, Eq, PartialEq, Debug, Default)]
enum GameState {
    #[default]
    Planning,
    Experimenting,
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

fn cleanup<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for t in query.iter() {
        commands.entity(t).despawn_recursive();
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

fn setup_title_screen(mut commands: Commands, fonts: Res<MyFonts>) {
    commands
        .spawn((
            TitleScreen,
            NodeBundle {
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
            },
        ))
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
                        .spawn((
                            StartButton,
                            ButtonBundle {
                                background_color: Color::GRAY.into(),
                                ..Default::default()
                            },
                        ))
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

fn spawn_scene(mut commands: Commands, my: Option<Res<MyAssets>>, assets_gltf: Res<Assets<Gltf>>) {
    if let Some(my) = my {
        if let Some(gltf) = assets_gltf.get(&my.main_gltf) {
            commands.spawn(SceneBundle {
                scene: gltf.scenes[0].clone(),
                ..Default::default()
            });
        }
    }
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
        point_light.intensity /= k;
    }

    for mut spot_light in spot_lights.iter_mut() {
        spot_light.intensity /= k;
    }

    for mut directional_light in directional_lights.iter_mut() {
        directional_light.illuminance *= TAU;
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

/***************************************/
/*    ##     ## ##     ## ########     */
/*    ##     ## ##     ## ##     ##    */
/*    ##     ## ##     ## ##     ##    */
/*    ######### ##     ## ##     ##    */
/*    ##     ## ##     ## ##     ##    */
/*    ##     ## ##     ## ##     ##    */
/*    ##     ##  #######  ########     */
/***************************************/

#[derive(Component)]
#[allow(clippy::upper_case_acronyms)]
struct HUD;

#[derive(Component)]
struct ExperimentButton(ExperimentAction);

enum ExperimentAction {
    Conduct,
    Finish,
}
#[derive(Component)]
struct ExperimentButtonCaption;

fn setup_hud(mut commands: Commands, fonts: Res<MyFonts>, medicines: Res<Medicines>) {
    let medicine_card = |parent: &mut ChildBuilder, medicine_index: usize, medicine: &Medicine| {
        // Medicine card
        parent
            .spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    gap: Size::all(Val::Px(10.)),
                    padding: UiRect::all(Val::Px(20.)),
                    ..Default::default()
                },
                background_color: Color::SALMON.into(),
                ..Default::default()
            })
            .with_children(|parent| {
                // Title
                parent.spawn(TextBundle::from_section(
                    "Medicine ".to_string() + &medicine.name,
                    TextStyle {
                        font: fonts.fira_sans_regular.clone_weak(),
                        font_size: 40.,
                        color: Color::BLACK.into(),
                    },
                ));

                parent.spawn(TextBundle::from_section(
                    "Effects:",
                    TextStyle {
                        font: fonts.fira_sans_regular.clone_weak(),
                        font_size: 30.,
                        color: Color::BLACK.into(),
                    },
                ));

                // Effects
                for (effect, value) in vec![
                    (MedicineEffect::Appetite, medicine.appetite),
                    (MedicineEffect::Smell, medicine.smell),
                ] {
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::SpaceBetween,
                                gap: Size::all(Val::Px(20.)),
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                effect.title(),
                                TextStyle {
                                    font: fonts.fira_sans_regular.clone_weak(),
                                    font_size: 20.,
                                    color: Color::BLACK.into(),
                                },
                            ));

                            parent
                                .spawn(NodeBundle {
                                    style: Style {
                                        flex_direction: FlexDirection::Row,
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })
                                .with_children(|parent| {
                                    for choice in vec![-1, 0, 1] {
                                        parent
                                            .spawn((
                                                MedicineEffectButton {
                                                    medicine_index,
                                                    effect,
                                                    value: choice,
                                                },
                                                ButtonBundle {
                                                    style: Style {
                                                        size: Size::all(Val::Px(20.)),
                                                        justify_content: JustifyContent::Center,
                                                        ..Default::default()
                                                    },
                                                    background_color: if value == choice {
                                                        // Selected
                                                        Color::YELLOW.into()
                                                    } else {
                                                        Color::GRAY.into()
                                                    },
                                                    ..Default::default()
                                                },
                                            ))
                                            .with_children(|parent| {
                                                parent.spawn(
                                                    TextBundle::from_section(
                                                        match choice {
                                                            0 => "=",
                                                            c if c < 0 => "↓",
                                                            _ => "↑",
                                                        },
                                                        TextStyle {
                                                            font: fonts
                                                                .fira_sans_regular
                                                                .clone_weak(),
                                                            font_size: 20.0,
                                                            color: Color::BLACK,
                                                        },
                                                    ) // Set the alignment of the Text
                                                    .with_text_alignment(TextAlignment::Center),
                                                );
                                            });
                                    }
                                });
                        });
                }

                // Included in experiment
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            gap: Size::all(Val::Px(20.)),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .with_children(|parent| {
                        parent.spawn(TextBundle::from_section(
                            "Include in test",
                            TextStyle {
                                font: fonts.fira_sans_regular.clone_weak(),
                                font_size: 20.,
                                color: Color::BLACK.into(),
                            },
                        ));

                        parent
                            .spawn(NodeBundle {
                                style: Style {
                                    flex_direction: FlexDirection::Row,
                                    ..Default::default()
                                },
                                ..Default::default()
                            })
                            .with_children(|parent| {
                                for value in vec![false, true] {
                                    parent
                                        .spawn((
                                            TestMedicineButton(medicine_index, value),
                                            ButtonBundle {
                                                style: Style {
                                                    justify_content: JustifyContent::Center,
                                                    padding: UiRect::horizontal(Val::Px(4.)),

                                                    ..Default::default()
                                                },

                                                background_color: if value == medicine.in_experiment
                                                {
                                                    // Selected
                                                    Color::YELLOW.into()
                                                } else {
                                                    Color::GRAY.into()
                                                },
                                                ..Default::default()
                                            },
                                        ))
                                        .with_children(|parent| {
                                            parent.spawn(
                                                TextBundle::from_section(
                                                    if value { "yes" } else { "no" },
                                                    TextStyle {
                                                        font: fonts.fira_sans_regular.clone_weak(),
                                                        font_size: 20.0,
                                                        color: Color::BLACK,
                                                    },
                                                ) // Set the alignment of the Text
                                                .with_text_alignment(TextAlignment::Center),
                                            );
                                        });
                                }
                            });
                    });
            });
    };

    commands
        .spawn((
            HUD,
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        top: Val::Percent(0.),
                        left: Val::Auto,
                        right: Val::Auto,
                        ..Default::default()
                    },
                    gap: Size::all(Val::Px(20.)),
                    padding: UiRect::all(Val::Px(40.)),
                    ..Default::default()
                },
                ..Default::default()
            },
        ))
        .with_children(|parent| {
            /************************************************************************************/
            /*    ##     ## ######## ########  ####  ######  #### ##    ## ########  ######     */
            /*    ###   ### ##       ##     ##  ##  ##    ##  ##  ###   ## ##       ##    ##    */
            /*    #### #### ##       ##     ##  ##  ##        ##  ####  ## ##       ##          */
            /*    ## ### ## ######   ##     ##  ##  ##        ##  ## ## ## ######    ######     */
            /*    ##     ## ##       ##     ##  ##  ##        ##  ##  #### ##             ##    */
            /*    ##     ## ##       ##     ##  ##  ##    ##  ##  ##   ### ##       ##    ##    */
            /*    ##     ## ######## ########  ####  ######  #### ##    ## ########  ######     */
            /************************************************************************************/
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        gap: Size::all(Val::Px(20.)),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with_children(|parent| {
                    for (medicine_index, medicine) in medicines.0.iter().enumerate() {
                        medicine_card(parent, medicine_index, medicine);
                    }
                });



            /****************************************************************************************************/
            /*    ########  ##          ###    ##    ##          ##     ######  ########  #######  ########     */
            /*    ##     ## ##         ## ##    ##  ##          ##     ##    ##    ##    ##     ## ##     ##    */
            /*    ##     ## ##        ##   ##    ####          ##      ##          ##    ##     ## ##     ##    */
            /*    ########  ##       ##     ##    ##          ##        ######     ##    ##     ## ########     */
            /*    ##        ##       #########    ##         ##              ##    ##    ##     ## ##           */
            /*    ##        ##       ##     ##    ##        ##         ##    ##    ##    ##     ## ##           */
            /*    ##        ######## ##     ##    ##       ##           ######     ##     #######  ##           */
            /****************************************************************************************************/
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent
                        .spawn((
                            ExperimentButton(ExperimentAction::Conduct),
                            ButtonBundle {
                                background_color: Color::GRAY.into(),
                                ..Default::default()
                            },
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                ExperimentButtonCaption,
                                TextBundle::from_section(
                                    // Accepts a `String` or any type that converts into a `String`, such as `&str`
                                    "Conduct experiment",
                                    TextStyle {
                                        font: fonts.fira_sans_regular.clone_weak(),
                                        font_size: 30.0,
                                        color: Color::BLACK,
                                    },
                                ) // Set the alignment of the Text
                                .with_text_alignment(TextAlignment::Center),
                            ));
                        });

                    parent
                        .spawn((
                            ExperimentButton(ExperimentAction::Finish),
                            ButtonBundle {
                                background_color: Color::GRAY.into(),
                                visibility: Visibility::Hidden,
                                ..Default::default()
                            },
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                ExperimentButtonCaption,
                                TextBundle::from_section(
                                    // Accepts a `String` or any type that converts into a `String`, such as `&str`
                                    "Finish experiment",
                                    TextStyle {
                                        font: fonts.fira_sans_regular.clone_weak(),
                                        font_size: 30.0,
                                        color: Color::BLACK,
                                    },
                                )
                                .with_text_alignment(TextAlignment::Center),
                            ));
                        });
                });
        });
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
struct Medicine {
    name: String,
    appetite: i32,
    smell: i32,
    in_experiment: bool,
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
        }
    }
}
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum MedicineEffect {
    Appetite,
    Smell,
}

impl MedicineEffect {
    fn title(&self) -> &str {
        match self {
            MedicineEffect::Appetite => "Appetite",
            MedicineEffect::Smell => "Smell",
        }
    }
}

#[derive(Resource)]
struct Medicines(Vec<Medicine>);

impl Default for Medicines {
    fn default() -> Self {
        Medicines(vec![
            Medicine::default().with_name("A"),
            Medicine::default().with_name("B"),
            Medicine::default().with_name("C"),
        ])
    }
}

#[derive(Component, Debug, PartialEq, Eq)]
struct MedicineEffectButton {
    medicine_index: usize,
    effect: MedicineEffect,
    value: i32,
}

fn medicine_property_button(
    interaction_query: Query<(&MedicineEffectButton, &Interaction), Changed<Interaction>>,
    mut medicines: ResMut<Medicines>,
    mut buttons: Query<(&mut BackgroundColor, &MedicineEffectButton)>,
) {
    for (this_button, interaction) in interaction_query.iter() {
        match interaction {
            Interaction::Clicked => {
                medicines.0[this_button.medicine_index]
                    .set_effect(&this_button.effect, this_button.value);

                for (mut background, button) in buttons.iter_mut() {
                    if button == this_button {
                        *background = Color::YELLOW.into()
                    } else if button.medicine_index == this_button.medicine_index
                        && button.effect == this_button.effect
                    {
                        *background = Color::GRAY.into()
                    }
                }
            }
            Interaction::Hovered => {}
            Interaction::None => {}
        }
    }
}

#[derive(Component, Debug, PartialEq, Eq)]
struct TestMedicineButton(usize, bool);

fn test_medicine_button(
    interaction_query: Query<(&TestMedicineButton, &Interaction), Changed<Interaction>>,
    mut medicines: ResMut<Medicines>,
    mut buttons: Query<(&mut BackgroundColor, &TestMedicineButton)>,
) {
    for (this_button, interaction) in interaction_query.iter() {
        match interaction {
            Interaction::Clicked => {
                medicines.0[this_button.0].in_experiment = this_button.1;

                for (mut background, button) in buttons.iter_mut() {
                    if button == this_button {
                        *background = Color::YELLOW.into()
                    } else if button.0 == this_button.0 {
                        *background = Color::GRAY.into()
                    }
                }
            }
            Interaction::Hovered => {}
            Interaction::None => {}
        }
    }
}

fn experiment_button(
    mut interaction_query: Query<(Entity, &ExperimentButton, &Interaction), Changed<Interaction>>,
    mut buttons: Query<(Entity, &mut Visibility), With<ExperimentButton>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (this, button, interaction) in interaction_query.iter_mut() {
        match interaction {
            Interaction::Clicked => {
                for (entity, mut visibility) in buttons.iter_mut() {
                    *visibility = if entity == this {
                        Visibility::Hidden
                    } else {
                        Visibility::Visible
                    }
                }
                match button.0 {
                    ExperimentAction::Conduct => next_state.set(GameState::Experimenting),
                    ExperimentAction::Finish => next_state.set(GameState::Planning),
                }
            }
            Interaction::Hovered => {}
            Interaction::None => {}
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
    min_y: i32,
}

impl PathfindingMatrix {
    fn grid_coord(&self, translation: Vec3) -> (usize, usize) {
        (
            (translation.x.floor() as i32 - self.min_x) as usize,
            (translation.z.floor() as i32 - self.min_y) as usize,
        )
    }
}

fn setup_pathfinding(mut commands: Commands, named_entities: Query<(&Name, &Transform)>) {
    let tile_coords: Vec<(i32, i32)> = named_entities
        .iter()
        .filter_map(|(name, transform)| {
            if name.starts_with("tile") {
                Some(transform)
            } else {
                None
            }
        })
        .map(|transform| {
            (
                transform.translation.x.floor() as i32,
                transform.translation.z.floor() as i32,
            )
        })
        .collect::<_>();

    let xs = tile_coords.iter().map(|(x, _)| x);
    let ys = tile_coords.iter().map(|(_, y)| y);
    let min_x = xs.min();
    let min_y = ys.min();

    if let (Some(min_x), Some(min_y)) = (min_x, min_y) {
        let grid = Grid::from_coordinates(&tile_coords);
        if let Some(grid) = grid {
            commands.insert_resource(PathfindingMatrix {
                grid,
                min_x: *min_x,
                min_y: *min_y,
            });
        }
    }
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
}

impl Rat {
    fn with_medicines(&self, medicines: &[Medicine]) -> Self {
        let mut new = self.clone();
        for medicine in medicines {
            if medicine.in_experiment {
                new.appetite += medicine.appetite;
                new.smell += medicine.smell;
            }
        }
        new.appetite = new.appetite.clamp(0, 2);
        new.smell = new.smell.clamp(0, 2);
        new
    }
}

impl Default for Rat {
    fn default() -> Self {
        Rat {
            appetite: 1,
            smell: 2,
        }
    }
}

#[derive(Component)]
struct Cheese;

fn setup_entities(
    mut commands: Commands,
    named_entities: Query<(Entity, &Name)>,
    medicines: Res<Medicines>,
) {
    for (entity, name) in named_entities.iter() {
        if name.starts_with("rat") {
            // 0.5 x 0.2 x 0.1
            let collider = commands
                .spawn((
                    Collider::cuboid(0.25, 0.1, 0.25),
                    TransformBundle::from_transform(Transform::from_xyz(0., 0.1, 0.)),
                ))
                .id();
            commands
                .entity(entity)
                .insert((
                    Rat::default().with_medicines(&medicines.0),
                    RigidBody::Dynamic,
                    KinematicCharacterController::default(),
                ))
                .add_child(collider);
        }

        if name.starts_with("cheese") {
            commands.entity(entity).insert(Cheese);
        }

        if name.starts_with("tile") {
            let collider = commands.spawn((Collider::cuboid(0.5, 0.065, 0.5),)).id();
            commands
                .entity(entity)
                .insert(RigidBody::Fixed)
                .add_child(collider);
        }
    }
}

fn find_cheese(
    mut commands: Commands,
    mut rats: Query<(Entity, &Rat, &mut Transform)>,
    cheese: Query<&Transform, (With<Cheese>, Without<Rat>)>,
    pathfinding: Res<PathfindingMatrix>,
) {
    for (rat_entity, rat, mut rat_transform) in rats.iter_mut() {
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
            // @TODO: if appetite is especially high, rat starts actively looking for food, even if can't smell it
            let mut paths = cheese
                .iter()
                .filter_map(|cheese| {
                    let start = pathfinding.grid_coord(rat_transform.translation);
                    let goal = pathfinding.grid_coord(cheese.translation);
                    let distance = rat_transform.translation.distance(cheese.translation);
                    if distance < smell_distance {
                        let smell_intencity = (smell_distance - distance).round() as i32;
                        astar(
                            &start,
                            |p| {
                                pathfinding
                                    .grid
                                    .neighbours(*p)
                                    .into_iter()
                                    .map(|p| (p, 1))
                                    .collect::<Vec<_>>()
                            },
                            |p| pathfinding.grid.distance(*p, goal) / 3,
                            |p| *p == goal,
                        )
                        .map(|(path, _cost)| (path, smell_intencity))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            paths.sort_by_key(|(_, smell_intencity)| -*smell_intencity);

            if let Some((path, _smell_intencity)) = paths.first() {
                if let &[first, second, ..] = path.as_slice() {
                    let speed = 2.0;
                    let linvel = Vec3::new(
                        second.0 as f32 - first.0 as f32,
                        0.0,
                        second.1 as f32 - first.1 as f32,
                    )
                    .normalize_or_zero()
                        * speed;
                    rat_transform.look_to(linvel, Vec3::Y);
                    commands.entity(rat_entity).insert(Velocity {
                        linvel,
                        ..Default::default()
                    });
                    continue;
                }
            }
        }

        // Stop
        commands.entity(rat_entity).remove::<Velocity>();
    }
}
