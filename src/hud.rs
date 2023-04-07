use bevy::prelude::*;

use crate::{AppState, Fonts, GameState, Medicine, MedicineEffect, Medicines};

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(setup.in_schedule(OnEnter(AppState::InGame)))
            .add_system(crate::cleanup::<HUD>.in_schedule(OnExit(AppState::InGame)))
            .add_systems(
                (
                    checkbox_init,
                    checkbox_update,
                    medicine_property_button,
                    medicine_test_togle_button,
                    experiment_button,
                    toggle_dev_mode,
                    report_effect_checkbox,
                    submit_button,
                    try_again_button,
                )
                    .in_set(OnUpdate(AppState::InGame)),
            );
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
struct ToogleDevMode;

#[derive(Component)]
struct DevPanel;

#[derive(Component)]
struct ExperimentButton(ExperimentAction);

enum ExperimentAction {
    Conduct,
    Finish,
}
#[derive(Component)]
struct ExperimentButtonCaption;

const BG_HIGHLIGHT: Color = Color::hsla(195., 0.86, 0.86, 1.);
const BG_DARK_GRAY: Color = Color::hsla(0., 0.0, 0.73, 1.);
const BG_LIGHT_GRAY: Color = Color::hsla(0., 0., 0.92, 1.);
const BG_WHITE: Color = Color::WHITE;

const BG_ACTION_WARNING: Color = Color::hsla(18.49, 0.82, 0.65, 1.);
const BG_ACTION_PRIMARY: Color = Color::hsla(194.05, 0.72, 0.70, 1.);

const P2: Val = Val::Px(2.);
const P4: Val = Val::Px(4.);
const P8: Val = Val::Px(8.);
const P10: Val = Val::Px(10.);
const P13: Val = Val::Px(13.);
const P20: Val = Val::Px(20.);
const P30: Val = Val::Px(30.);
const P40: Val = Val::Px(40.);

const ACTION_BUTTON_STYLE: Style = Style {
    padding: UiRect::new(P30, P30, P13, P13),
    min_size: Size::width(Val::Px(170.)),
    justify_content: JustifyContent::Center,
    ..Style::DEFAULT
};

fn h1(fonts: &Fonts) -> TextStyle {
    TextStyle {
        font: fonts.bold.clone_weak(),
        font_size: 20.,
        color: Color::BLACK,
    }
}

fn h2(fonts: &Fonts) -> TextStyle {
    TextStyle {
        font: fonts.semibold.clone_weak(),
        font_size: 15.,
        color: Color::BLACK,
    }
}

fn text(fonts: &Fonts) -> TextStyle {
    TextStyle {
        font: fonts.regular.clone_weak(),
        font_size: 12.,
        color: Color::BLACK,
    }
}

fn bold(fonts: &Fonts) -> TextStyle {
    TextStyle {
        font: fonts.bold.clone_weak(),
        font_size: 12.,
        color: Color::BLACK,
    }
}

fn button_caption(fonts: &Fonts) -> TextStyle {
    TextStyle {
        font: fonts.bold.clone_weak(),
        font_size: 12.,
        color: Color::WHITE,
    }
}

fn setup(mut commands: Commands, fonts: Res<Fonts>, medicines: Res<Medicines>) {
    let medicine_card = |parent: &mut ChildBuilder, medicine_index: usize, medicine: &Medicine| {
        // Medicine card
        parent
            .spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    gap: Size::all(P10),
                    ..Default::default()
                },
                background_color: Color::NONE.into(),
                ..Default::default()
            })
            .with_children(|parent| {
                // Title
                parent
                    .spawn((
                        MedicineTitleCard(medicine_index),
                        NodeBundle {
                            style: Style {
                                flex_direction: FlexDirection::Column,

                                max_size: Size {
                                    width: Val::Px(208.),
                                    height: Val::Auto,
                                },
                                padding: UiRect::new(P20, P20, P8, P8),
                                gap: Size::all(P4),
                                ..Default::default()
                            },
                            background_color: if medicine.in_experiment {
                                BG_HIGHLIGHT.into()
                            } else {
                                BG_DARK_GRAY.into()
                            },
                            ..Default::default()
                        },
                    ))
                    .with_children(|parent| {
                        parent.spawn(
                            TextBundle::from_section(
                                "Medicine ".to_string() + &medicine.name,
                                h1(&fonts),
                            )
                            .with_text_alignment(TextAlignment::Center)
                            .with_style(Style {
                                align_self: AlignSelf::Center,
                                ..Default::default()
                            }),
                        );

                        parent.spawn((
                            MedicineInTestToggleButton(medicine_index),
                            CheckboxBundle::new("Include in experiment?", medicine.in_experiment),
                        ));
                    });

                parent
                    .spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::new(P20, P20, P8, P8),
                            gap: Size::all(P8),
                            ..Default::default()
                        },
                        background_color: BG_LIGHT_GRAY.into(),
                        ..Default::default()
                    })
                    .with_children(|parent| {
                        parent.spawn(
                            TextBundle::from_section("Report card".to_string(), h2(&fonts))
                                .with_text_alignment(TextAlignment::Center)
                                .with_style(Style {
                                    align_self: AlignSelf::Center,
                                    ..Default::default()
                                }),
                        );

                        parent
                            .spawn(NodeBundle {
                                style: Style {
                                    flex_direction: FlexDirection::Column,
                                    gap: Size::all(P4),
                                    ..Default::default()
                                },
                                ..Default::default()
                            })
                            .with_children(|parent| {
                                for (effect, value) in &[
                                    (MedicineEffect::Appetite, medicine.report.appetite),
                                    (MedicineEffect::Fear, medicine.report.fear),
                                    (MedicineEffect::Smell, medicine.report.smell),
                                ] {
                                    parent.spawn((
                                        ReportEffectCheckbox {
                                            medicine_index,
                                            effect: *effect,
                                            value: 1,
                                        },
                                        CheckboxBundle::new(effect.positive(), *value > 0),
                                    ));
                                }
                            });

                        parent.spawn(NodeBundle {
                            style: Style {
                                size: Size::height(P2),
                                ..Default::default()
                            },
                            background_color: BG_DARK_GRAY.into(),
                            ..Default::default()
                        });

                        parent.spawn(
                            TextBundle::from_section("Side effects".to_string(), h2(&fonts))
                                .with_text_alignment(TextAlignment::Center)
                                .with_style(Style {
                                    align_self: AlignSelf::Center,
                                    ..Default::default()
                                }),
                        );

                        parent
                            .spawn(NodeBundle {
                                style: Style {
                                    flex_direction: FlexDirection::Column,
                                    gap: Size::all(P4),
                                    ..Default::default()
                                },
                                ..Default::default()
                            })
                            .with_children(|parent| {
                                for (effect, value) in &[
                                    (MedicineEffect::Appetite, medicine.report.appetite),
                                    (MedicineEffect::Fear, medicine.report.fear),
                                    (MedicineEffect::Smell, medicine.report.smell),
                                ] {
                                    parent.spawn((
                                        ReportEffectCheckbox {
                                            medicine_index,
                                            effect: *effect,
                                            value: -1,
                                        },
                                        CheckboxBundle::new(effect.negative(), *value < 0),
                                    ));
                                }
                            });
                    });

                // Developer
                parent
                    .spawn((
                        DevPanel,
                        NodeBundle {
                            style: Style {
                                flex_direction: FlexDirection::Column,
                                padding: UiRect::new(P20, P20, P8, P8),
                                gap: Size::all(P8),
                                ..Default::default()
                            },
                            background_color: BG_LIGHT_GRAY.into(),
                            visibility: Visibility::Hidden,
                            ..Default::default()
                        },
                    ))
                    .with_children(|parent| {
                        parent.spawn(
                            TextBundle::from_section("Effects".to_string(), h2(&fonts))
                                .with_text_alignment(TextAlignment::Center)
                                .with_style(Style {
                                    align_self: AlignSelf::Center,
                                    ..Default::default()
                                }),
                        );

                        for (effect, value) in &[
                            (MedicineEffect::Appetite, medicine.appetite),
                            (MedicineEffect::Fear, medicine.fear),
                            (MedicineEffect::Smell, medicine.smell),
                        ] {
                            parent
                                .spawn(NodeBundle {
                                    style: Style {
                                        flex_direction: FlexDirection::Row,
                                        justify_content: JustifyContent::SpaceBetween,
                                        align_items: AlignItems::Center,
                                        gap: Size::all(P4),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })
                                .with_children(|parent| {
                                    parent.spawn(TextBundle::from_section(
                                        effect.title(),
                                        text(&fonts),
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
                                            for choice in &[-1, 0, 1] {
                                                parent
                                                    .spawn((
                                                        MedicineEffectButton {
                                                            medicine_index,
                                                            effect: *effect,
                                                            value: *choice,
                                                        },
                                                        ButtonBundle {
                                                            style: Style {
                                                                size: Size::all(P20),
                                                                justify_content:
                                                                    JustifyContent::Center,
                                                                align_items: AlignItems::Center,
                                                                ..Default::default()
                                                            },
                                                            background_color: if value == choice {
                                                                BG_HIGHLIGHT.into()
                                                            } else {
                                                                BG_WHITE.into()
                                                            },
                                                            ..Default::default()
                                                        },
                                                    ))
                                                    .with_children(|parent| {
                                                        parent.spawn(
                                                            TextBundle::from_section(
                                                                match choice {
                                                                    0 => "=",
                                                                    c if c < &0 => "↓",
                                                                    _ => "↑",
                                                                },
                                                                text(&fonts),
                                                            )
                                                            .with_text_alignment(
                                                                TextAlignment::Center,
                                                            ),
                                                        );
                                                    });
                                            }
                                        });
                                });
                        }
                    });
            });
    };

    commands
        .spawn((
            HUD,
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    position:  UiRect::all(Val::Percent(0.)),
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
                                background_color: Color::DARK_GRAY.into(),
                                style: Style{
                                    padding: UiRect::all(Val::Px(10.)),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                ExperimentButtonCaption,
                                TextBundle::from_section(
                                    "Conduct experiment",
                                    TextStyle {
                                        font: fonts.regular.clone_weak(),
                                        font_size: 30.0,
                                        color: Color::WHITE,
                                    },
                                ) // Set the alignment of the Text
                                .with_text_alignment(TextAlignment::Center),
                            ));
                        });

                    parent
                        .spawn((
                            ExperimentButton(ExperimentAction::Finish),
                            ButtonBundle {
                                background_color: Color::DARK_GRAY.into(),
                                style: Style{
                                    padding: UiRect::all(Val::Px(10.)),
                                    ..Default::default()
                                },
                                visibility: Visibility::Hidden,
                                ..Default::default()
                            },
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                ExperimentButtonCaption,
                                TextBundle::from_section(
                                    "Finish experiment",
                                    TextStyle {
                                        font: fonts.regular.clone_weak(),
                                        font_size: 30.0,
                                        color: Color::WHITE,
                                    },
                                )
                                .with_text_alignment(TextAlignment::Center),
                            ));
                        });
                });

            /**************************************************************/
            /*     ######  ##     ## ########  ##     ## #### ########    */
            /*    ##    ## ##     ## ##     ## ###   ###  ##     ##       */
            /*    ##       ##     ## ##     ## #### ####  ##     ##       */
            /*     ######  ##     ## ########  ## ### ##  ##     ##       */
            /*          ## ##     ## ##     ## ##     ##  ##     ##       */
            /*    ##    ## ##     ## ##     ## ##     ##  ##     ##       */
            /*     ######   #######  ########  ##     ## ####    ##       */
            /**************************************************************/
            parent.spawn((SubmitBlock,
                NodeBundle{
                            style: Style{
                                position_type: PositionType::Absolute,
                                position: UiRect {
                                    left: P20,
                                    bottom: P40,
                                    ..Default::default()
                                },
                                max_size: Size::width(Val::Percent(40.)),
                                flex_direction:FlexDirection::Column,
                                align_items: AlignItems::Start,
                                gap: Size::all(P10),
                                ..Default::default()
                            },
                            ..Default::default()
                        })).with_children(|parent|{
                parent.spawn((SubmitButton,
                    ButtonBundle{
                                    style: ACTION_BUTTON_STYLE,
                                    background_color: BG_ACTION_WARNING.into(),
                                    ..Default::default()
                                })).with_children(|parent|{
                    parent.spawn(TextBundle::from_section("Submit report cards",
                        button_caption(&fonts)));
                });

                parent.spawn(TextBundle::from_sections(
                    vec![
                        TextSection::new("Note: ", bold(&fonts)),
                        TextSection::new("Make sure you’ve discovered all side effects of each medicine", text(&fonts)),
                        TextSection::new(" — the success of the entire lab depends on your work!", text(&fonts))
                        ]


                    ));
            });

            parent.spawn((ResultsBlock,
                NodeBundle{
                            style: Style{
                                position_type: PositionType::Absolute,
                                position: UiRect {
                                    left: P20,
                                    bottom: P40,
                                    ..Default::default()
                                },
                                max_size: Size::width(Val::Percent(40.)),
                                flex_direction:FlexDirection::Column,
                                align_items: AlignItems::Start,
                                gap: Size::all(P10),
                                ..Default::default()
                            },
                            visibility: Visibility::Hidden,
                            ..Default::default()
                        })).with_children(|parent|{
                parent.spawn((TryAgainButton,
                    ButtonBundle{
                                    style: ACTION_BUTTON_STYLE,
                                    background_color: BG_ACTION_PRIMARY.into(),
                                    ..Default::default()
                                })).with_children(|parent|{
                    parent.spawn(TextBundle::from_section("Try again?",
                        button_caption(&fonts)));
                });

                parent.spawn(TextBundle::from_sections(
                    vec![
                        TextSection::new("Note: ", bold(&fonts)),
                        TextSection::new("Make sure you’ve discovered all side effects of each medicine", text(&fonts)),
                        TextSection::new(" — the success of the entire lab depends on your work!", text(&fonts))
                        ]


                    ));
            });


            parent.spawn((
                ToogleDevMode,
                ButtonBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        position: UiRect {
                            top: Val::Percent(0.0),
                            left: Val::Percent(0.0),
                            ..Default::default()
                        },
                        size: Size::all(P20),
                        ..Default::default()
                    },
                    background_color: BG_DARK_GRAY.into(),
                    ..Default::default()
                }
            ));
        });
}

fn toggle_dev_mode(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ToogleDevMode>)>,
    mut dev_panels: Query<&mut Visibility, With<DevPanel>>,
) {
    for interaction in interaction_query.iter() {
        match interaction {
            Interaction::Clicked => {
                for mut visibility in dev_panels.iter_mut() {
                    *visibility = if *visibility == Visibility::Hidden {
                        Visibility::Visible
                    } else {
                        Visibility::Hidden
                    };
                }
            }
            Interaction::Hovered => {}
            Interaction::None => {}
        }
    }
}

#[derive(Component, Debug)]
struct MedicineTitleCard(usize);

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
                        *background = BG_HIGHLIGHT.into()
                    } else if button.medicine_index == this_button.medicine_index
                        && button.effect == this_button.effect
                    {
                        *background = BG_WHITE.into()
                    }
                }
            }
            Interaction::Hovered => {}
            Interaction::None => {}
        }
    }
}

#[derive(Component, Debug, PartialEq, Eq)]
struct MedicineInTestToggleButton(usize);

fn medicine_test_togle_button(
    mut interaction_query: Query<
        (&MedicineInTestToggleButton, &Interaction, &mut Checkbox),
        Changed<Interaction>,
    >,
    mut title_cards: Query<(&MedicineTitleCard, &mut BackgroundColor)>,
    mut medicines: ResMut<Medicines>,
) {
    for (this_button, interaction, mut checkbox) in interaction_query.iter_mut() {
        match interaction {
            Interaction::Clicked => {
                medicines.0[this_button.0].in_experiment =
                    !medicines.0[this_button.0].in_experiment;
                checkbox.checked = medicines.0[this_button.0].in_experiment;

                for (title_card, mut background_color) in title_cards.iter_mut() {
                    if title_card.0 == this_button.0 {
                        background_color.0 = if checkbox.checked {
                            BG_HIGHLIGHT
                        } else {
                            BG_DARK_GRAY
                        };
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

#[derive(Component)]
struct ReportEffectCheckbox {
    medicine_index: usize,
    effect: MedicineEffect,
    value: i32,
}

fn report_effect_checkbox(
    mut interaction_query: Query<
        (&Interaction, &mut Checkbox, &ReportEffectCheckbox),
        Changed<Interaction>,
    >,
    mut medicines: ResMut<Medicines>,
) {
    for (interaction, mut checkbox, report_effect) in interaction_query.iter_mut() {
        match interaction {
            Interaction::Clicked => {
                checkbox.checked = !checkbox.checked;
                medicines.0[report_effect.medicine_index]
                    .report
                    .mark_effect(
                        &report_effect.effect,
                        if checkbox.checked {
                            report_effect.value
                        } else {
                            -report_effect.value
                        },
                    );
            }
            Interaction::Hovered => {}
            Interaction::None => {}
        }
    }
}

#[derive(Component, Debug)]
struct SubmitButton;

fn submit_button(
    mut commands: Commands,
    interactions: Query<&Interaction, With<SubmitButton>>,
    blocks: Query<(Entity, AnyOf<(&SubmitBlock, &ResultsBlock)>)>,
) {
    for interaction in interactions.iter() {
        match interaction {
            Interaction::Clicked => {
                for (block, (submit, results)) in blocks.iter() {
                    if submit.is_some() {
                        commands.entity(block).insert(Visibility::Hidden);
                    }

                    if results.is_some() {
                        commands.entity(block).insert(Visibility::Visible);
                    }
                }
            }
            Interaction::Hovered => {}
            Interaction::None => {}
        }
    }
}

#[derive(Component, Debug)]
struct SubmitBlock;

#[derive(Component, Debug)]
struct ResultsBlock;
#[derive(Component, Debug)]
struct TryAgainButton;

fn try_again_button(
    mut commands: Commands,
    interactions: Query<&Interaction, With<TryAgainButton>>,
    blocks: Query<(Entity, AnyOf<(&SubmitBlock, &ResultsBlock)>)>,
) {
    for interaction in interactions.iter() {
        match interaction {
            Interaction::Clicked => {
                for (block, (submit, results)) in blocks.iter() {
                    if submit.is_some() {
                        commands.entity(block).insert(Visibility::Visible);
                    }

                    if results.is_some() {
                        commands.entity(block).insert(Visibility::Hidden);
                    }
                }
            }
            Interaction::Hovered => {}
            Interaction::None => {}
        }
    }
}

/*************************************************************************************/
/*     ######  ##     ## ########  ######  ##    ## ########   #######  ##     ##    */
/*    ##    ## ##     ## ##       ##    ## ##   ##  ##     ## ##     ##  ##   ##     */
/*    ##       ##     ## ##       ##       ##  ##   ##     ## ##     ##   ## ##      */
/*    ##       ######### ######   ##       #####    ########  ##     ##    ###       */
/*    ##       ##     ## ##       ##       ##  ##   ##     ## ##     ##   ## ##      */
/*    ##    ## ##     ## ##       ##    ## ##   ##  ##     ## ##     ##  ##   ##     */
/*     ######  ##     ## ########  ######  ##    ## ########   #######  ##     ##    */
/*************************************************************************************/

#[derive(Bundle)]
struct CheckboxBundle {
    checkbox: Checkbox,
    node_bundle: ButtonBundle,
}

impl CheckboxBundle {
    fn new(label: &str, checked: bool) -> Self {
        Self {
            checkbox: Checkbox {
                checked,
                label: label.to_string(),
            },
            node_bundle: ButtonBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    gap: Size::all(Val::Px(20.)),
                    ..Default::default()
                },
                background_color: Color::NONE.into(),
                ..Default::default()
            },
        }
    }
}

#[derive(Component, Debug)]
struct Checkbox {
    label: String,
    checked: bool,
}

#[derive(Component, Debug)]
struct CheckboxLabel;

#[derive(Component, Debug)]
struct CheckboxField;

#[derive(Component, Debug)]
struct CheckboxMarker;

fn checkbox_init(
    mut commands: Commands,
    checkboxes: Query<(Entity, &Checkbox), Added<Checkbox>>,
    fonts: Res<Fonts>,
) {
    for (entity, checkbox) in checkboxes.iter() {
        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                CheckboxLabel,
                TextBundle::from_section(&checkbox.label, text(&fonts)),
            ));

            parent
                .spawn((
                    CheckboxField,
                    NodeBundle {
                        style: Style {
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            padding: UiRect::horizontal(Val::Px(4.)),
                            size: Size::all(P20),
                            ..Default::default()
                        },

                        background_color: BG_WHITE.into(),
                        ..Default::default()
                    },
                ))
                .with_children(|parent| {
                    let text_bundle = TextBundle::from_section("×", h1(&fonts))
                        .with_text_alignment(TextAlignment::Center);
                    parent.spawn((
                        CheckboxMarker,
                        TextBundle {
                            visibility: if checkbox.checked {
                                Visibility::Visible
                            } else {
                                Visibility::Hidden
                            },
                            ..text_bundle
                        },
                    ));
                });
        });
    }
}

fn checkbox_update(
    checkboxes: Query<(&Checkbox, &Children), Changed<Checkbox>>,
    fields: Query<&Children, With<CheckboxField>>,
    mut markers: Query<&mut Visibility, With<CheckboxMarker>>,
) {
    for (checkbox, children) in checkboxes.iter() {
        for child in children {
            if let Ok(field_children) = fields.get(*child) {
                for child in field_children {
                    if let Ok(mut marker_visibility) = markers.get_mut(*child) {
                        if checkbox.checked {
                            *marker_visibility = Visibility::Visible;
                        } else {
                            *marker_visibility = Visibility::Hidden;
                        }
                    }
                }
            }
        }
    }
}
