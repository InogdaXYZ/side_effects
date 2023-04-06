use bevy::prelude::*;

use crate::{Fonts, GameState, Medicine, MedicineEffect, Medicines};
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
pub struct HUD;

#[derive(Component)]
pub struct ExperimentButton(ExperimentAction);

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

const P2: Val = Val::Px(2.);
const P4: Val = Val::Px(4.);
const P8: Val = Val::Px(8.);
const P10: Val = Val::Px(10.);
const P20: Val = Val::Px(20.);

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

pub fn setup(mut commands: Commands, fonts: Res<Fonts>, medicines: Res<Medicines>) {
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
                    .spawn(NodeBundle {
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
                        background_color: BG_DARK_GRAY.into(),
                        ..Default::default()
                    })
                    .with_children(|parent| {
                        parent.spawn(
                            TextBundle::from_section(
                                "Medicine ".to_string() + &medicine.name,
                                h1(&fonts),
                            )
                            .with_text_alignment(TextAlignment::Center),
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
                                .with_text_alignment(TextAlignment::Center),
                        );

                        for (effect, value) in &[
                            (MedicineEffect::Appetite, false),
                            (MedicineEffect::Fear, false),
                            (MedicineEffect::Smell, false),
                        ] {
                            parent.spawn(CheckboxBundle::new(effect.positive(), *value));
                        }

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
                                .with_text_alignment(TextAlignment::Center),
                        );

                        for (effect, value) in &[
                            (MedicineEffect::Appetite, false),
                            (MedicineEffect::Fear, false),
                            (MedicineEffect::Smell, false),
                        ] {
                            parent.spawn(CheckboxBundle::new(effect.negative(), *value));
                        }
                    });

                // Developer
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
                            TextBundle::from_section("Effects".to_string(), h2(&fonts))
                                .with_text_alignment(TextAlignment::Center),
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
        });
}

#[derive(Component, Debug, PartialEq, Eq)]
pub struct MedicineEffectButton {
    medicine_index: usize,
    effect: MedicineEffect,
    value: i32,
}

pub fn medicine_property_button(
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
pub struct MedicineInTestToggleButton(usize);

pub fn medicine_test_togle_button(
    mut interaction_query: Query<
        (&MedicineInTestToggleButton, &Interaction, &mut Checkbox),
        Changed<Interaction>,
    >,
    mut medicines: ResMut<Medicines>,
) {
    for (this_button, interaction, mut checkbox) in interaction_query.iter_mut() {
        match interaction {
            Interaction::Clicked => {
                medicines.0[this_button.0].in_experiment =
                    !medicines.0[this_button.0].in_experiment;
                checkbox.checked = medicines.0[this_button.0].in_experiment;
            }
            Interaction::Hovered => {}
            Interaction::None => {}
        }
    }
}

pub fn experiment_button(
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
pub struct CheckboxBundle {
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
pub struct Checkbox {
    label: String,
    checked: bool,
}

#[derive(Component, Debug)]
pub struct CheckboxLabel;

#[derive(Component, Debug)]
pub struct CheckboxField;

#[derive(Component, Debug)]
pub struct CheckboxMarker;

pub fn checkbox_init(
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
                            padding: UiRect::horizontal(Val::Px(4.)),

                            ..Default::default()
                        },

                        background_color: BG_WHITE.into(),
                        ..Default::default()
                    },
                ))
                .with_children(|parent| {
                    parent.spawn((
                        CheckboxMarker,
                        TextBundle::from_section(
                            if checkbox.checked { "x" } else { "" },
                            text(&fonts),
                        ) // Set the alignment of the Text
                        .with_text_alignment(TextAlignment::Center),
                    ));
                });
        });
    }
}

pub fn checkbox_update(
    checkboxes: Query<(&Checkbox, &Children), Changed<Checkbox>>,
    fields: Query<&Children, With<CheckboxField>>,
    mut markers: Query<&mut Text, With<CheckboxMarker>>,
    fonts: Res<Fonts>,
) {
    for (checkbox, children) in checkboxes.iter() {
        for child in children {
            if let Ok(field_children) = fields.get(*child) {
                for child in field_children {
                    if let Ok(mut marker_text) = markers.get_mut(*child) {
                        marker_text.sections = vec![TextSection::new(
                            if checkbox.checked { "x" } else { "" },
                            text(&fonts),
                        )];
                    }
                }
            }
        }
    }
}
