mod client_connection;
mod keyboard;
mod mouse;
mod render_engine;
mod window_handler;

use glam::{vec3a, Vec3A};

use self::{
  client_connection::ClientConnection,
  keyboard::KeyboardController,
  mouse::MouseController,
  render_engine::{instanced_render_matrix::InstancedRenderData, RenderEngine},
  window_handler::WindowHandler,
};

const TESTING_LIMIT: usize = 100;

use super::lua_engine::LuaEngine;

///
/// The Client component for the engine.
///
/// The Client component has 5 jobs:
/// 1.) Hold a window.
/// 2.) Hold the render engine.
/// 3.) Hold a ClientConnection which handles talking to a server.
/// * 4.) [in the future] Be the main handler for ClientAuthentication.
/// *  - ClientAuthentication does exactly what you think it does.
/// *  - Maintains a client auth for itself when talking to the server.
/// ? 5.) Handle GameConfig as a component. This should be received from a server
/// ? 5 - Marked with ? because it's still being thought out at the moment.
///
pub struct Client {
  window_handler: WindowHandler,
  render_engine: RenderEngine,
  client_name: String,
  connection: ClientConnection,
  lua_engine: LuaEngine,

  mouse: MouseController,
  keyboard: KeyboardController,

  quit_received: bool,

  // ! TESTING
  spin_test: f64,
}

impl Client {
  pub fn new(client_name: String, address: String, port: i32) -> Self {
    // Input engines.
    let mut mouse = MouseController::new();
    let keyboard = KeyboardController::new();

    // Set up the window handler.
    let window_handler = WindowHandler::new(&mut mouse);

    // Set up the render engine.
    let render_engine = RenderEngine::new(&window_handler);

    // Set up a blank client connection.
    let connection = ClientConnection::new(address, port);

    // Finally create the Client-side luau virtual machine.
    let lua_engine = LuaEngine::new(false);

    let mut new_client = Client {
      window_handler,
      render_engine,
      client_name,
      connection,
      lua_engine,

      mouse,
      keyboard,

      quit_received: false,

      // ! TESTING
      spin_test: 0.0,
    };

    new_client.reset_lua_vm();

    println!("minetest: player name is: {}", &new_client.client_name);

    new_client
  }

  ///
  /// Change the client's name.
  ///
  pub fn change_name(&mut self, new_client_name: String) {
    self.client_name = new_client_name;
  }

  ///
  /// Get the client's name.
  ///
  pub fn get_name(&self) -> String {
    // Just fire off new heap memory.
    self.client_name.clone()
  }

  ///
  /// Wipe the memory of the lua VM.
  /// Automatically regenerates a blank client VM.
  ///
  pub fn reset_lua_vm(&mut self) {
    self.lua_engine = LuaEngine::new(false);
  }

  ///
  /// Send client quit event.
  ///
  pub fn quit(&mut self) {
    self.quit_received = true;
  }

  ///
  /// Retrieve if the client wants to quit.
  ///
  pub fn should_quit(&self) -> bool {
    self.quit_received
  }

  ///
  /// Borrow the WindowHandler mutably.
  ///
  pub fn get_window_handler(&mut self) -> &mut WindowHandler {
    &mut self.window_handler
  }

  ///
  /// Tick tock.
  ///
  /// Every time the game goes into the next main loop iteration
  /// this is run.
  ///
  /// This is referred to as on_step in C++ minetest.
  ///
  pub fn on_tick(&mut self, delta: f64) {
    // This is for the Mouse' Camera controls.
    self.mouse.reset_mouse_relative_position();

    // Update the SDL2 context.
    self
      .window_handler
      .update(delta, &mut self.mouse, &mut self.keyboard);

    // Poll any incoming network traffic. (non blocking)
    if self.connection.is_connected() {
      self.connection.receive(delta);
    }

    //todo: probably should do user input here

    self.lua_engine.on_tick(delta);

    //todo: should probably do side effects from lua here

    let camera = self.render_engine.get_camera();

    let mut camera_pos = *camera.get_position();

    let move_speed = delta as f32 * 10.0;

    // * A very simple test to check the buffer in the shader.

    // If jordan4ibanez sees what i did here he would either call the optimazation police after me or die of cringe.
    if self.keyboard.is_key_down("A") {
        let camera_rot = self.render_engine.get_camera().get_rotation();
      camera_pos.z += move_speed * f32::sin(camera_rot.y);
      camera_pos.x += move_speed * f32::cos(camera_rot.y);
    }
    if self.keyboard.is_key_down("D") {
        let camera_rot = self.render_engine.get_camera().get_rotation();
      camera_pos.z -= move_speed * f32::sin(camera_rot.y);
      camera_pos.x -= move_speed * f32::cos(camera_rot.y);
    }

    if self.keyboard.is_key_down("W") {
    	let camera_rot = self.render_engine.get_camera().get_rotation();
      camera_pos.z += move_speed * f32::cos(-camera_rot.y);
      camera_pos.x += move_speed * f32::sin(-camera_rot.y);
      camera_pos.y += move_speed * f32::sin(camera_rot.x);
    }
    if self.keyboard.is_key_down("S") {
        let camera_rot = self.render_engine.get_camera().get_rotation();
      camera_pos.z -= move_speed * f32::cos(-camera_rot.y);
      camera_pos.x -= move_speed * f32::sin(-camera_rot.y);
      camera_pos.y -= move_speed * f32::sin(camera_rot.x);
    }

    if self.keyboard.is_key_down("Left Shift") {
      camera_pos.y += move_speed;
    }
    if self.keyboard.is_key_down("Space") {
      camera_pos.y -= move_speed;
    }

    // println!("camera pos {:?}", camera_pos);
    let mouse_relative = self.mouse.get_relative_position();
    if mouse_relative.length_squared() != 0 {
      // println!("Mouse is moved!");
      let camera = self.render_engine.get_camera();
      let mut camera_rotation = *camera.get_rotation();

      camera_rotation.y += mouse_relative.x as f32 * self.mouse.get_sensitivity();
      camera_rotation.x += mouse_relative.y as f32 * self.mouse.get_sensitivity();

      camera.set_rotation(&camera_rotation);

      println!("{:?}", camera.get_rotation());
    }

    self.render_engine.get_camera().set_position(&camera_pos);

    // Update the RenderEngine with the WindowHandler.
    self.render_engine.update(&self.window_handler, delta);

    // Now render everything.

    self.spin_test += delta*0.1;

    // println!("spin  {}", self.spin_test);

    // Update the camera's projection matrix.
    self
      .render_engine
      .update_camera_matrix(&self.window_handler);

    // Now create the framebuffer.
    self.render_engine.generate_frame_buffer();

    // Clear it, it contains old data.

    self.render_engine.clear_buffers(true, true);

    // ? Begin rendering.

    self.render_engine.initialize_render();

    // * Begin not instanced.

    // Not instanced.
    self.render_engine.render_mesh(
      "debug",
      "tf.webp",
      Vec3A::new(-1.0, 0.0, 0.0),
      Vec3A::new(0.0, -self.spin_test as f32, 0.0),
      Vec3A::new(1.0, 1.0, 1.0),
    );

    self.render_engine.render_model(
      "chair.obj",
      vec!["chair.png".to_string()],
      Vec3A::new(-2.0, 0.0, 0.0),
      Vec3A::new(0.0, -self.spin_test as f32, 0.0),
      Vec3A::new(1.0, 1.0, 1.0),
    );

    let snowman_texture = "snowman.png".to_string();
    self.render_engine.render_model(
      "snowman.obj",
      vec![
        snowman_texture.clone(),
        snowman_texture.clone(),
        snowman_texture.clone(),
        snowman_texture.clone(),
        snowman_texture.clone(),
      ],
      Vec3A::new(-3.0, 0.0, 0.0),
      Vec3A::new(0.0, -self.spin_test as f32, 0.0),
      Vec3A::new(1.0, 1.0, 1.0),
    );

    self.render_engine.process_not_instanced_render_calls();

    // * Begin instanced.

    let mut instancing = Vec::with_capacity(TESTING_LIMIT * TESTING_LIMIT);

    for x in 0..TESTING_LIMIT {
      for z in 0..TESTING_LIMIT {
        instancing.push(InstancedRenderData::new(
          vec3a(x as f32, z as f32, 0.0),
          vec3a(0.0, self.spin_test as f32, 0.0),
          vec3a(1.0, 1.0, 1.0),
        ));
      }
    }

    self
      .render_engine
      .render_mesh_instanced("debug", &instancing);

    self.render_engine.process_instanced_render_calls();

    // ? End rendering calls.

    self.render_engine.show_and_destroy_frame_buffer();

    // This will need to run a close event for the client engine and send out a close event to the internal server.
    if self.window_handler.should_quit() {
      self.quit();
    }
  }
}
