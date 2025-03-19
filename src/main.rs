use graphics::renderer::Renderer;
use macroquad::{camera::{set_camera, Camera3D}, color::{Color, BEIGE}, math::vec3, window::{clear_background, next_frame}};
use model::{location::{InternalLocation, Location, LOCATION_OFFSET}, world::World};

mod model;
mod service;
mod graphics;

#[macroquad::main("Game")]
async fn main() {
    println!("Hello, world!");

    let mut world = World::new();
    let mut renderer = Renderer::new().await;


    let position = vec3(0.0 , 0.0 , 10.0);
    let target = position + vec3(2.0, 0.0, 0.0);
    let mut camera = Camera3D {
        position: position,
            up: vec3(0.0, 0.0, -1.0),
            target: position + vec3(2.0, 0.0, 0.0),
            fovy: 70.0_f32.to_radians(),
            ..Default::default()
    };
    world.set(Location::new(10, 0, 10).into(), model::voxel::Voxel::Stone);
    //world.set(Location::new(10, -1, 10).into(), model::voxel::Voxel::Stone);
    world.set(Location::new(10, 1, 10).into(), model::voxel::Voxel::Stone);
    world.set(Location::new(10, 0, 9).into(), model::voxel::Voxel::Stone);
    let area = Location::new(10, 0, 10).into();
    world.unload_area(area);
    world.load_area(area);
    renderer.load_full_area(&world, area);

    println!("camera {} -> {}", camera.position, camera.target);
    loop {
        clear_background(BEIGE);
        
        //camera.position.z += -0.001;
        camera.position.y += 0.003;
        camera.position.x += 0.04;
        camera.target = camera.position + (target - camera.position).normalize();

        set_camera(&camera);
        println!("camera {} -> {}", camera.position, camera.target);
    
        renderer.render_voxels();

        next_frame().await;
    }
}
