use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use rand::prelude::*;

const POPULATION: u32 = 300; // how many meeples
const START_INFECTED_CHANCE: f32 = 0.01; // % chance that a meeple starts infected
const INFECTION_DISTANCE: f32 = 20.0; // how close (in units) do two meeples have to be to risk spread of infection?
const SQUARED_INFECTION_DISTANCE: f32 = INFECTION_DISTANCE * INFECTION_DISTANCE; // it's a square now!
const INFECTION_CHANCE: f32 = 0.001; // % chance that two meeples within the infection distance for 1 update tick spread the disease

const MEEPLE_SPEED: f32 = 40.0; // units/s
const MEEPLE_STEP_SIZE: f32 = 120.0; // approximate distance a meeple moves before turning

const BOUNDING_BOX_SIZE: f32 = 600.0; // side length of the meeples' playpen
const BOUNDING_BOX_OFFSET: (f32, f32, f32) = (-250.0, 0.0, 0.0); // position of the center of the box

#[derive(Debug)]
struct Colors {
    susceptible: Handle<bevy::prelude::ColorMaterial>,
    infected: Handle<bevy::prelude::ColorMaterial>,
    recovered: Handle<bevy::prelude::ColorMaterial>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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
        .add_system(keep_meeples_in_box.system())
        .add_system(spread_infection.system())
        .run();
}

fn boil_plates(
    commands: &mut Commands, 
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let meeple_colors = Colors {
        susceptible: materials.add(Color::rgb(0.1, 0.1, 0.7).into()), // blue
        infected: materials.add(Color::rgb(0.7, 0.1, 0.1).into()), // red
        recovered: materials.add(Color::rgb(0.1, 0.7, 0.1).into()), // green
    };

    let half_bounding_box_size = BOUNDING_BOX_SIZE * 0.5 + 5.0; // add some padding to make it look prettier

    commands
        .spawn(Camera2dBundle::default())
        .insert_resource(meeple_colors)
        .spawn(primitive(
            materials.add(Color::rgb(0.15, 0.15, 0.15).into()), // dark gray
            &mut meshes,
            ShapeType::Quad(
                (-half_bounding_box_size, half_bounding_box_size).into(),
                (-half_bounding_box_size, -half_bounding_box_size).into(),
                (half_bounding_box_size, -half_bounding_box_size).into(),
                (half_bounding_box_size, half_bounding_box_size).into(),
            ),
            TessellationMode::Stroke(&StrokeOptions::default().with_line_width(4.0)),
            Vec3::new(BOUNDING_BOX_OFFSET.0, BOUNDING_BOX_OFFSET.1, BOUNDING_BOX_OFFSET.2),
        ));
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
        let x_pos = (rng.gen::<f32>() - 0.5) * BOUNDING_BOX_SIZE + BOUNDING_BOX_OFFSET.0;
        let y_pos = (rng.gen::<f32>() - 0.5) * BOUNDING_BOX_SIZE + BOUNDING_BOX_OFFSET.1;

        let infection_information = if rng.gen::<f32>() < START_INFECTED_CHANCE {
            (InfectionStatus::Infected, meeple_colors.infected.clone())
        } else {
            (InfectionStatus::Susceptible, meeple_colors.susceptible.clone())
        };

        commands
            .spawn(primitive(
                // makes a SpriteBundle from a shape with lyon
                infection_information.1,
                &mut meshes,
                ShapeType::Circle(4.0),
                TessellationMode::Fill(&FillOptions::default()),
                Vec3::new(x_pos, y_pos, 0.0),
            ))
            .with(Meeple)
            .with(infection_information.0)
            .with(DirectedMover{
                speed: MEEPLE_SPEED,
                target_location: Vec2::new(x_pos, y_pos)
            });
    }
}

fn move_meeples(
    time: Res<Time>,
    mut meeple_query: Query<(&mut Transform, &mut DirectedMover), With<Meeple>>,
) {
    let mut rng = rand::thread_rng();

    for (mut transform, mut directed_mover) in meeple_query.iter_mut() {
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

fn keep_meeples_in_box(
    mut meeple_query: Query<(&Transform, &mut DirectedMover), With<Meeple>>,
) {

    for (transform, mut directed_mover) in meeple_query.iter_mut() {
        let half_bounding_box_size = BOUNDING_BOX_SIZE * 0.5;
        let position = transform.translation.truncate();

        // if a meeple goes off the edge, change it's target to something inside the box
        // we add/subtract the step size to "turn the meeple around"
        if position.x < BOUNDING_BOX_OFFSET.0 - half_bounding_box_size {
            directed_mover.target_location.x += MEEPLE_STEP_SIZE;
        } else if position.x > BOUNDING_BOX_OFFSET.0 + half_bounding_box_size {
            directed_mover.target_location.x -= MEEPLE_STEP_SIZE;
        } 
        
        if position.y < BOUNDING_BOX_OFFSET.1 - half_bounding_box_size {
            directed_mover.target_location.y += MEEPLE_STEP_SIZE;
        } else if position.y > BOUNDING_BOX_OFFSET.1 + half_bounding_box_size {
            directed_mover.target_location.y -= MEEPLE_STEP_SIZE;
        }
    }
}

fn spread_infection(
    mut meeple_query: Query<(
        &Transform, 
        &mut InfectionStatus, 
        &mut Handle<bevy::prelude::ColorMaterial>
    ), With<Meeple>>,
    time: Res<Time>,
    meeple_colors: Res<Colors>,
) {
    let mut rng = rand::thread_rng();
    
    for (infected_transform, infected_infection_status, _) in meeple_query.iter() {
        if *infected_infection_status == InfectionStatus::Infected { // for all infected meeples
            for (susceptible_transform, mut infection_status, mut color) in meeple_query.iter_mut() {
                if *infection_status == InfectionStatus::Susceptible { // for all susceptible meeples
                    let vector_between = infected_transform.translation.truncate() - susceptible_transform.translation.truncate();
                    let squared_distance = vector_between.length_squared();
                    if squared_distance <= (SQUARED_INFECTION_DISTANCE) { // if the meeples are close enough to risk spread
                        if rng.gen::<f32>() <= INFECTION_CHANCE { // if this meeple got really unlucky
                            *color = meeple_colors.infected.clone(); // udpate color
                            *infection_status = InfectionStatus::Infected; // udpate status
                        }
                    }
                }
            }
        }
    }
}