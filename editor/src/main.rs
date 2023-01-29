use std::any::TypeId;

use bevy::asset::{HandleId, ReflectAsset};
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::pbr::wireframe::WireframePlugin;
use bevy::prelude::*;
use bevy::reflect::{TypeRegistryArc, TypeRegistryInternal};
use bevy::render::camera::Viewport;
use bevy_egui::egui;
use bevy_egui::{EguiPlugin, EguiSettings};
use bevy_framepace::Limiter;
use bevy_inspector_egui::bevy_inspector::hierarchy::{hierarchy_ui, SelectedEntities};
use bevy_inspector_egui::bevy_inspector::{
    self, ui_for_entities_shared_components, ui_for_entity_with_children,
};
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use egui_dock::{NodeIndex, Tree};
use smooth_bevy_cameras::controllers::orbit::*;
use smooth_bevy_cameras::{LookTransform, LookTransformBundle, LookTransformPlugin, Smoother};

use game::data::DataPlugin;
use game::tree::TreePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_framepace::FramepacePlugin) // reduces input lag
        .add_plugin(DefaultInspectorConfigPlugin)
        .add_plugin(EguiPlugin)
        .add_plugin(OrbitCameraPlugin {
            override_input_system: true,
        })
        .add_plugin(LookTransformPlugin)
        .add_plugin(WireframePlugin::default())
        .insert_resource(UiState::new())
        .add_system_to_stage(CoreStage::PreUpdate, show_ui_system.at_end())
        .add_startup_system(setup)
        .add_startup_system(setup_frame_limiter)
        .add_system(set_camera_viewport)
        .register_type::<Option<Handle<Image>>>()
        .register_type::<AlphaMode>()
        .add_plugin(TreePlugin)
        .add_plugin(DataPlugin)
        .run();
}

#[derive(Component)]
struct MainCamera;

fn show_ui_system(world: &mut World) {
    let mut egui_context = world
        .resource_mut::<bevy_egui::EguiContext>()
        .ctx_mut()
        .clone();

    world.resource_scope::<UiState, _>(|world, mut ui_state| ui_state.ui(world, &mut egui_context));
}

// make camera only render to view not obstructed by UI
fn set_camera_viewport(
    ui_state: Res<UiState>,
    windows: Res<Windows>,
    egui_settings: Res<EguiSettings>,
    mut cameras: Query<&mut Camera, With<MainCamera>>,
) {
    let mut cam = cameras.single_mut();

    let window = windows.primary();
    let scale_factor = window.scale_factor() * egui_settings.scale_factor;

    let viewport_pos = ui_state.viewport_rect.left_top().to_vec2() * scale_factor as f32;
    let viewport_size = ui_state.viewport_rect.size() * scale_factor as f32;

    cam.viewport = Some(Viewport {
        physical_position: UVec2::new(viewport_pos.x as u32, viewport_pos.y as u32),
        physical_size: UVec2::new(viewport_size.x as u32, viewport_size.y as u32),
        depth: 0.0..1.0,
    });
}

#[derive(Eq, PartialEq)]
enum InspectorSelection {
    Entities,
    Resource(TypeId, String),
    Asset(TypeId, String, HandleId),
}

impl InspectorSelection {
    fn select(
        &mut self,
        new_selection: InspectorSelection,
        selected_entities: &mut SelectedEntities,
    ) {
        if new_selection != InspectorSelection::Entities && !selected_entities.is_empty() {
            selected_entities.clear();
        }
        *self = new_selection;
    }
}

#[derive(Resource)]
struct UiState {
    tree: Tree<Window>,
    viewport_rect: egui::Rect,
    selected_entities: SelectedEntities,
    selection: InspectorSelection,
}

impl UiState {
    pub fn new() -> Self {
        let mut tree = Tree::new(vec![Window::GameView]);
        let [game, _inspector] = tree.split_right(NodeIndex::root(), 0.75, vec![Window::Inspector]);
        let [game, _hierarchy] = tree.split_left(game, 0.2, vec![Window::Hierarchy]);
        let [_game, _bottom] = tree.split_below(game, 0.8, vec![Window::Resources, Window::Assets]);

        Self {
            tree,
            selected_entities: SelectedEntities::default(),
            selection: InspectorSelection::Entities,
            viewport_rect: egui::Rect::NOTHING,
        }
    }

    fn ui(&mut self, world: &mut World, ctx: &mut egui::Context) {
        let mut tab_viewer = TabViewer {
            world,
            viewport_rect: &mut self.viewport_rect,
            selected_entities: &mut self.selected_entities,
            selection: &mut self.selection,
        };
        egui_dock::DockArea::new(&mut self.tree).show(ctx, &mut tab_viewer);
    }
}

#[derive(Debug)]
enum Window {
    GameView,
    Hierarchy,
    Inspector,
    Resources,
    Assets,
}

struct TabViewer<'a> {
    world: &'a mut World,
    selected_entities: &'a mut SelectedEntities,
    selection: &'a mut InspectorSelection,
    viewport_rect: &'a mut egui::Rect,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = Window;

    fn ui(&mut self, ui: &mut egui::Ui, window: &mut Self::Tab) {
        let type_registry: TypeRegistryArc = self.world.resource::<AppTypeRegistry>().0.clone();
        let type_registry = type_registry.read();

        match window {
            Window::GameView => setup_gameview_ui(ui, self.world, self.viewport_rect),
            Window::Resources => {
                select_resource(ui, &type_registry, self.selection, self.selected_entities)
            }
            Window::Assets => select_asset(
                ui,
                &type_registry,
                self.world,
                self.selection,
                self.selected_entities,
            ),
            Window::Hierarchy => {
                hooked_hierarchy_ui(self.world, ui, self.selection, self.selected_entities)
            }
            Window::Inspector => match *self.selection {
                InspectorSelection::Entities => match self.selected_entities.as_slice() {
                    &[entity] => ui_for_entity_with_children(self.world, entity, ui),
                    entities => ui_for_entities_shared_components(self.world, entities, ui),
                },
                InspectorSelection::Resource(type_id, ref name) => {
                    ui.label(name);
                    bevy_inspector::by_type_id::ui_for_resource(
                        self.world,
                        type_id,
                        ui,
                        name,
                        &type_registry,
                    )
                }
                InspectorSelection::Asset(type_id, ref name, handle) => {
                    ui.label(name);
                    bevy_inspector::by_type_id::ui_for_asset(
                        self.world,
                        type_id,
                        handle,
                        ui,
                        &type_registry,
                    );
                }
            },
        }
    }

    fn title(&mut self, window: &mut Self::Tab) -> egui::WidgetText {
        format!("{window:?}").into()
    }

    fn clear_background(&self, window: &Self::Tab) -> bool {
        !matches!(window, Window::GameView)
    }
}

fn setup_gameview_ui(ui: &mut egui::Ui, world: &mut World, viewport_rect: &mut egui::Rect) {
    let response;
    (*viewport_rect, response) =
        ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());

    let mut controllers = world.query_filtered::<&OrbitCameraController, With<MainCamera>>();
    if let Ok(&controller) = controllers.get_single(world) {
        let mouse_wheel_events = world
            .resource_mut::<Events<MouseWheel>>()
            .drain()
            .collect::<Vec<_>>();

        let mut scroll_events = world.resource_mut::<Events<ControlEvent>>();

        if response.dragged() {
            let drag_delta = response.drag_delta();
            let input = &ui.input().pointer;

            if input.button_down(egui::PointerButton::Primary) {
                scroll_events.send(ControlEvent::Orbit(
                    controller.mouse_rotate_sensitivity * Vec2::new(drag_delta.x, drag_delta.y),
                ));
            } else if input.button_down(egui::PointerButton::Secondary) {
                scroll_events.send(ControlEvent::TranslateTarget(
                    controller.mouse_translate_sensitivity * Vec2::new(drag_delta.x, drag_delta.y),
                ));
            }
        }

        if response.hovered() {
            for event in mouse_wheel_events {
                let mut scalar = 1.0;
                let scroll_amount = match event.unit {
                    MouseScrollUnit::Line => event.y,
                    MouseScrollUnit::Pixel => event.y / controller.pixels_per_line,
                };
                scalar *= 1.0 - scroll_amount * controller.mouse_wheel_zoom_sensitivity;

                scroll_events.send(ControlEvent::Zoom(scalar));
            }
        }
    }
}

fn hooked_hierarchy_ui(
    world: &mut World,
    ui: &mut egui::Ui,
    selection: &mut InspectorSelection,
    selected_entities: &mut SelectedEntities,
) {
    if selection != &InspectorSelection::Entities && !selected_entities.is_empty() {
        *selection = InspectorSelection::Entities;
    }
    hierarchy_ui(world, ui, selected_entities);
}

fn select_resource(
    ui: &mut egui::Ui,
    type_registry: &TypeRegistryInternal,
    selection: &mut InspectorSelection,
    selected_entities: &mut SelectedEntities,
) {
    let mut resources: Vec<_> = type_registry
        .iter()
        .filter(|registration| registration.data::<ReflectResource>().is_some())
        .map(|registration| (registration.short_name().to_owned(), registration.type_id()))
        .collect();
    resources.sort_by(|(name_a, _), (name_b, _)| name_a.cmp(name_b));

    for (resource_name, type_id) in resources {
        let selected = match *selection {
            InspectorSelection::Resource(selected, _) => selected == type_id,
            _ => false,
        };

        if ui.selectable_label(selected, &resource_name).clicked() {
            selection.select(
                InspectorSelection::Resource(type_id, resource_name),
                selected_entities,
            );
        }
    }
}

fn select_asset(
    ui: &mut egui::Ui,
    type_registry: &TypeRegistryInternal,
    world: &World,
    selection: &mut InspectorSelection,
    selected_entities: &mut SelectedEntities,
) {
    let mut assets: Vec<_> = type_registry
        .iter()
        .filter_map(|registration| {
            let reflect_asset = registration.data::<ReflectAsset>()?;
            Some((
                registration.short_name().to_owned(),
                registration.type_id(),
                reflect_asset,
            ))
        })
        .collect();
    assets.sort_by(|(name_a, ..), (name_b, ..)| name_a.cmp(name_b));

    for (asset_name, asset_type_id, reflect_asset) in assets {
        let mut handles: Vec<_> = reflect_asset.ids(world).collect();
        handles.sort();

        ui.collapsing(format!("{asset_name} ({})", handles.len()), |ui| {
            for handle in handles {
                let selected = match *selection {
                    InspectorSelection::Asset(_, _, selected_id) => selected_id == handle,
                    _ => false,
                };

                if ui
                    .selectable_label(selected, format!("{:?}", handle))
                    .clicked()
                {
                    selection.select(
                        InspectorSelection::Asset(asset_type_id, asset_name.clone(), handle),
                        selected_entities,
                    );
                }
            }
        });
    }
}

fn setup_frame_limiter(mut settings: ResMut<bevy_framepace::FramepaceSettings>) {
    settings.limiter = Limiter::from_framerate(60.0);
}

fn setup(mut commands: Commands) {
    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.02,
    });

    // directional light
    const HALF_SIZE: f32 = 10.0;
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.0,
            shadow_projection: OrthographicProjection {
                left: -HALF_SIZE,
                right: HALF_SIZE,
                bottom: -HALF_SIZE,
                top: HALF_SIZE,
                near: -10.0 * HALF_SIZE,
                far: 10.0 * HALF_SIZE,
                ..Default::default()
            },
            ..Default::default()
        },
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::PI / 2.0)),
        ..Default::default()
    });

    let orbit_camera_controller = OrbitCameraController {
        smoothing_weight: 0.1,
        mouse_rotate_sensitivity: Vec2::splat(0.3),
        mouse_translate_sensitivity: Vec2::splat(0.3),
        ..Default::default()
    };
    // camera
    commands.spawn((
        Camera3dBundle::default(),
        orbit_camera_controller,
        LookTransformBundle {
            transform: LookTransform::new(
                Vec3 {
                    x: 0.0,
                    y: 7.0,
                    z: 10.0,
                },
                Vec3::ZERO,
            ),
            smoother: Smoother::new(orbit_camera_controller.smoothing_weight),
        },
        MainCamera,
    ));
}
