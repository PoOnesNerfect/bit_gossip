use crate::game::enemy::Enemy;
use bevy::{
    color::palettes::tailwind::RED_300,
    prelude::*,
    sprite::{Material2d, Mesh2dHandle},
};

use super::CharacterMesh;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, insert_enemy_material)
            .add_systems(PostUpdate, spawn_enemies);
    }
}

#[derive(Resource)]
pub struct EnemyMaterial(pub Handle<ColorMaterial>);

/// A component bundle for entities with a [`Mesh2dHandle`] and a [`Material2d`].
#[derive(Bundle, Clone)]
pub struct MaterialMesh2dBundle<M: Material2d> {
    pub mesh: Mesh2dHandle,
    pub material: Handle<M>,
    pub global_transform: GlobalTransform,
    /// User indication of whether an entity is visible
    pub visibility: Visibility,
    // Inherited visibility of an entity.
    pub inherited_visibility: InheritedVisibility,
    // Indication of whether an entity is visible in any view.
    pub view_visibility: ViewVisibility,
}

impl<M: Material2d> Default for MaterialMesh2dBundle<M> {
    fn default() -> Self {
        Self {
            mesh: Default::default(),
            material: Default::default(),
            global_transform: Default::default(),
            visibility: Default::default(),
            inherited_visibility: Default::default(),
            view_visibility: Default::default(),
        }
    }
}

fn insert_enemy_material(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    let material = materials.add(Color::from(RED_300));
    commands.insert_resource(EnemyMaterial(material));
}

fn spawn_enemies(
    mut commands: Commands,
    mesh: Res<CharacterMesh>,
    material: Res<EnemyMaterial>,
    enemies: Query<Entity, Added<Enemy>>,
) {
    for id in &enemies {
        commands.entity(id).insert(MaterialMesh2dBundle {
            mesh: mesh.0.clone().into(),
            material: material.0.clone(),
            ..Default::default()
        });
    }
}
