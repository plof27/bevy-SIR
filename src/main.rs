use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use rand::prelude::*;

const POPULATION: u32 = 100; // how many meeples
const MEEPLE_SPEED: f32 = 40.0; // units/s
const MEEPLE_STEP_SIZE: f32 = 120.0; // approximate distance a meeple moves before turning

#[derive(Debug)]
struct Colors {
    susceptible: Handle<bevy::prelude::ColorMaterial>,
    infected: Handle<bevy::prelude::ColorMaterial>,
    recovered: Handle<bevy::prelude::ColorMaterial>,
}

#[derive(Copy, Clone, Debug)]
enum InfectionStatus {
    Susceptible,
    Infected,
    Recovered,
}

#[derive(Copy, Clone, Debug)]
struct DirectedMover { // things that move with intent
    speed: f32, // rate of movement
    target_location: Vec2 // (x, y) place you're moving to
}

// Marker component for meeples. Meeples durdle around and get sick.
struct Meeple;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_startup_stage("init_resources", SystemStage::serial())
        .add_startup_stage("spawn_entities", SystemStage::parallel())
        .add_startup_system_to_stage("init_resources", boil_plates.system())
        .add_startup_system_to_stage("spawn_entities", spawn_meeples.system())
        .add_system(move_meeples.system())
        .run();
}

fn boil_plates(commands: &mut Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    let meeple_colors = Colors {
        susceptible: materials.add(Color::rgb(0.1, 0.1, 0.7).into()), // blue
        infected: materials.add(Color::rgb(0.7, 0.1, 0.1).into()), // red
        recovered: materials.add(Color::rgb(0.1, 0.7, 0.1).into()), // green
    };

    commands
        .spawn(Camera2dBundle::default())
        .insert_resource(meeple_colors);
}

fn spawn_meeples(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    meeple_colors: Res<Colors>,
) {
    // init rng
    let mut rng = rand::thread_rng();

    // use a for loop instead of spawn batch because of thread safety limitations
    for _ in 0..POPULATION {
        // TODO: figure out real boundaries for this
        let x_pos = (rng.gen::<f32>() - 0.5) * 1000.0;
        let y_pos = (rng.gen::<f32>() - 0.5) * 500.0;

        commands
            .spawn(primitive(
                // makes a SpriteBundle from a shape with lyon
                meeple_colors.susceptible.clone(),
                &mut meshes,
                ShapeType::Circle(4.0),
                TessellationMode::Fill(&FillOptions::default()),
                Vec3::new(x_pos, y_pos, 0.0),
            ))
            .with(Meeple)
            .with(InfectionStatus::Susceptible)
            .with(DirectedMover{
                speed: MEEPLE_SPEED,
                target_location: Vec2::new(x_pos, y_pos)
            });
    }
}

fn move_meeples(
    time: Res<Time>,
    mut meeples_query: Query<(&mut Transform, &mut DirectedMover), With<Meeple>>,
) {
    let mut rng = rand::thread_rng();

    for (mut transform, mut directed_mover) in meeples_query.iter_mut() {
        // extract the 2d (x,y) position of the meeple. almost all we need from the transform
        // (although we need to write to transform.translation directly when jumping)
        let position = transform.translation.truncate(); 

        let distance_to_move = directed_mover.speed * time.delta_seconds();
        
        let vector_to_target = directed_mover.target_location - position;
        let distance_to_target = vector_to_target.length();

        if directed_mover.target_location == position {
            // pick a new target
            let offset = Vec2::new(
                (rng.gen::<f32>() - 0.5) * MEEPLE_STEP_SIZE, 
                (rng.gen::<f32>() - 0.5) * MEEPLE_STEP_SIZE
            );
            directed_mover.target_location = position + offset;

        } else if distance_to_target <= distance_to_move {
            // we're within a frame of reaching our target. jump to it baby!
            // extend the target location so they're both Vec3's
            transform.translation = directed_mover.target_location.extend(0.0);
        } else {
            // just move normally
            let velocity = vector_to_target.normalize() * distance_to_move;
            transform.translation += velocity.extend(0.0);
        }
    }
}