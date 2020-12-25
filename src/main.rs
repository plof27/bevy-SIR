use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use rand::prelude::*;

const POPULATION: u32 = 1000;
const MEEPLE_SPEED: f32 = 4.0;

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
    target_location: (f32, f32) // (x, y) place you're moving to
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
        .run();
}

fn boil_plates(commands: &mut Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    let meeple_colors = Colors {
        susceptible: materials.add(Color::rgb(0.1, 0.4, 0.5).into()), // blue
        infected: materials.add(Color::rgb(0.8, 0.0, 0.0).into()), // red
        recovered: materials.add(Color::rgb(0.3, 0.4, 0.3).into()), // green
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
                target_location: (x_pos, y_pos)
            });
    }
}

fn move_meeples(
    time: Res<Time>,
    mut meeples_query: Query<(&mut Transform, &mut DirectedMover), With<Meeple>>,
) {

    for (mut transform, mut directed_mover) in meeples_query.iter_mut() {
        let distance_to_move = directed_mover.speed * time.delta_seconds();
        let squared_distance_to_move = distance_to_move * distance_to_move;
        
        let x_distance = directed_mover.target_location.0 - transform.translation[0];
        let y_distance = directed_mover.target_location.1 - transform.translation[1];
        let squared_x_distance = x_distance * x_distance;
        let squared_y_distance = y_distance * y_distance;
        let squared_distance_to_target = squared_x_distance + squared_y_distance;

        // if (directed_mover.target_location == transform.translation) {
        //     // make a new one somehow
        // } else if ()
    }
}