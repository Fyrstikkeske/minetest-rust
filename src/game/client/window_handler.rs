mod key_event_enum;

use glam::UVec2;
use sdl2::{
  event::WindowEvent,
  hint,
  keyboard::{Mod, Scancode},
  mouse::MouseState,
  video::{FullscreenType, Window},
  Sdl, VideoSubsystem,
};

use log::error;

use self::key_event_enum::KeyEvent;

use super::{keyboard::KeyboardController, mouse::MouseController};

///
/// SDL2 window controller.
///
/// This is a wrapper around SDL2.
///
/// This also implements state management for SDL2 components
/// that don't have retrievable state.
///
/// This can be renamed to SdlWrapper if we find ourselves
/// using more components of it than originally intended.
///
/// Minetest consists of 1 window, so we will ignore any window IDs.
///
pub struct WindowHandler {
  sdl_context: Sdl,
  video_subsystem: VideoSubsystem,
  window: Window,

  quit_received: bool,
  visible: bool,
  size: UVec2,

  maximized: bool,
}

impl WindowHandler {
  pub fn new(mouse: &mut MouseController) -> Self {
    // We're going to do this line by line,
    // in case any of this fails.

    // We want to use wgpu as our rendering multiplexer, disable OpenGL.
    hint::set("SDL_VIDEO_EXTERNAL_CONTEXT", "1");

    let sdl_context = match sdl2::init() {
      Ok(sdl) => sdl,
      Err(e) => panic!("WindowHandler: Failed to initialize SDL2. {}", e),
    };

    let video_subsystem = match sdl_context.video() {
      Ok(subsystem) => subsystem,
      Err(e) => panic!("WindowHandler: Failed to initialize video subsystem. {}", e),
    };

    let size = UVec2::new(512, 512);

    let window = match video_subsystem
      .window("AMONGUSTESTLOLOLOLOLOLOLOLOLOLOLOLOLOL", size.x, size.y)
      .resizable()
      .position_centered()
      .allow_highdpi()
      .metal_view()
      .build()
    {
      Ok(window) => window,
      Err(e) => panic!("WindowBuilder: Failed to initialize window. {}", e),
    };

    let mut new_window_handler = WindowHandler {
      sdl_context,
      video_subsystem,
      window,

      quit_received: false,
      visible: false,
      size,

      maximized: false,
    };

    new_window_handler.show();

    new_window_handler.toggle_mouse_capture(mouse);

    new_window_handler
  }

  ///
  /// Borrow the WindowHandler's actual window.
  ///
  pub fn borrow_window(&self) -> &Window {
    &self.window
  }

  ///
  /// Make the window visible.
  ///
  pub fn show(&mut self) {
    self.visible = true;
    self.window.show();
  }

  ///
  /// Hide the window
  ///
  pub fn hide(&mut self) {
    self.visible = true;
    self.window.show();
  }

  ///
  /// Set the window title.
  ///
  pub fn set_title(&mut self, new_title: &str) {
    match self.window.set_title(new_title) {
      Ok(_) => (),
      Err(e) => panic!("WindowHandler: Failed to set title. {}", e),
    }
  }

  ///
  /// Get the window title.
  ///
  pub fn get_title(&self) -> &str {
    self.window.title()
  }

  ///
  /// Get if the window is in real fullscreen mode.
  ///
  pub fn is_fullscreen_real_mode(&mut self) -> bool {
    self.window.fullscreen_state() == FullscreenType::True
  }

  ///
  /// Get if the window is in fake borderless fullscreen mode.
  ///
  pub fn is_fullscreen_borderless_mode(&mut self) -> bool {
    self.window.fullscreen_state() == FullscreenType::Desktop
  }

  ///
  /// Get if the window is in ANY fullscreen mode.
  ///
  pub fn is_fullscreen_any_mode(&mut self) -> bool {
    matches!(
      self.window.fullscreen_state(),
      FullscreenType::True | FullscreenType::Desktop
    )
  }

  ///
  /// Set the window to real fullscreen mode.
  ///
  pub fn set_fullscreen_real_mode(&mut self) {
    match self.window.set_fullscreen(FullscreenType::True) {
      Ok(_) => (),
      Err(e) => panic!("WindowHandler: Failed to set fullscreen real mode. {}", e),
    }
  }

  ///
  /// Set the window to fake borderless fullscreen mode.
  ///
  pub fn set_fullscreen_borderless_mode(&mut self) {
    match self.window.set_fullscreen(FullscreenType::Desktop) {
      Ok(_) => (),
      Err(e) => panic!(
        "WindowHandler: Failed to set fullscreen borderless mode. {}",
        e
      ),
    }
  }

  ///
  /// Set the window to normal windowed mode. (not fullscreen)
  ///
  pub fn set_windowed_mode(&mut self) {
    match self.window.set_fullscreen(FullscreenType::Off) {
      Ok(_) => (),
      Err(e) => panic!("WindowHandler: Failed to set windoed mode. {}", e),
    }
  }

  ///
  /// Toggle the Window's maximized state.
  ///
  pub fn toggle_maximize(&mut self) {
    self.maximized = !self.maximized;
    if self.maximized {
      self.window.maximize();
    } else {
      self.window.restore();
    }
  }

  ///
  /// Send window quit event.
  ///
  pub fn quit(&mut self) {
    self.quit_received = true;
  }

  ///
  /// Retrieve if the window wants to quit.
  ///
  pub fn should_quit(&self) -> bool {
    self.quit_received
  }

  ///
  /// Capture/release the mouse, updating it's internal state.
  ///
  fn toggle_mouse_capture(&self, mouse: &mut MouseController) {
    mouse.toggle_relative_mode();
    self
      .sdl_context
      .mouse()
      .set_relative_mouse_mode(mouse.is_relative_mode());
  }

  ///
  /// The key event handler.
  ///
  fn handle_key_event(
    &mut self,
    scancode_option: Option<Scancode>,
    keymod: Mod,
    keyevent: KeyEvent,
    mouse: &mut MouseController,
    keyboard: &mut KeyboardController,
  ) {
    // Since SDL2 can poll anything, we need to ensure that we can actually utilize the sent scancode.
    match scancode_option {
      Some(scancode) => {
        println!("TESTING: {}", scancode);

        // And for now, when you press escape, the game simply exits.
        if scancode == Scancode::Escape {
          self.quit();
        }
        // ! TEMPORARY TESTING !
        if scancode == Scancode::F5 && keyevent.is_down() {
          self.toggle_mouse_capture(mouse)
        }

        // ! MAXIMIZE TESTING
        if scancode == Scancode::F11 && keyevent.is_down() {
          self.toggle_maximize();
        }

        keyboard.set_key(&scancode.to_string(), keyevent.is_down());
      }

      // If we can't use it, oops. Bail out.
      None => {
        error!("WindowHandler: User sent unknown scancode.");
      }
    };
  }

  ///
  /// Internally updates the window size, automatically.
  ///
  fn update_size(&mut self, width: i32, height: i32) {
    self.size.x = width as u32;
    self.size.y = height as u32;
  }

  ///
  /// Borrow the window size immutably.
  ///
  pub fn get_size(&self) -> &UVec2 {
    &self.size
  }

  ///
  /// Get the window width.
  ///
  pub fn get_width(&self) -> u32 {
    self.size.x
  }

  ///
  /// Get the window height.
  ///
  pub fn get_height(&self) -> u32 {
    self.size.y
  }

  ///
  /// Automatically updates the Mouse' position.
  ///
  fn update_mouse_position(&self, x: i32, y: i32, mouse: &mut MouseController) {
    mouse.set_position(x, y);
  }

  ///
  /// Automatically updates the Mouse' relative position.
  ///
  fn update_mouse_relative_position(&self, xrel: i32, yrel: i32, mouse: &mut MouseController) {
    mouse.set_relative_position(xrel, yrel);
  }

  ///
  /// The mouse motion handler.
  ///
  fn handle_mouse_motion_event(
    &mut self,
    mousestate: MouseState,
    x: i32,
    y: i32,
    xrel: i32,
    yrel: i32,
    mouse: &mut MouseController,
  ) {
    self.update_mouse_position(x, y, mouse);
    self.update_mouse_relative_position(xrel, yrel, mouse);
  }

  ///
  /// The window event handler.
  ///
  fn handle_window_event(&mut self, win_event: WindowEvent) {
    match win_event {
      WindowEvent::None => println!("WindowHandler window: event none"),
      WindowEvent::Shown => println!("WindowHandler window: event shown"),
      WindowEvent::Hidden => println!("WindowHandler window: event hidden"),
      WindowEvent::Exposed => println!("WindowHandler window: event exposed"),
      WindowEvent::Moved(x, y) => println!("WindowHandler window: event moved | x: {} | y: {} |", x, y),
      WindowEvent::Resized(width, height) => {
        println!(
          "WindowHandler window: event resized | width: {} | height: {} |",
          width, height
        );

        self.update_size(width, height);
      }
      WindowEvent::SizeChanged(width, height) => {
        println!(
          "WindowHandler window: event size changed | width: {} | height: {} |",
          width, height
        );
        self.update_size(width, height);
      }
      WindowEvent::Minimized => println!("WindowHandler window: event minimized"),
      WindowEvent::Maximized => println!("WindowHandler window: event maximized"),
      WindowEvent::Restored => println!("WindowHandler window: event restored"),
      WindowEvent::Enter => println!("WindowHandler window: event enter"),
      WindowEvent::Leave => println!("WindowHandler window: event leave"),
      WindowEvent::FocusGained => println!("WindowHandler window: event focus gained"),
      WindowEvent::FocusLost => println!("WindowHandler window: event focus lost"),
      WindowEvent::Close => {
        println!("WindowHandler window: event close");
        self.quit();
      }
      WindowEvent::TakeFocus => println!("WindowHandler window: event take focus"),
      WindowEvent::HitTest => println!("WindowHandler window: event hit test"),
      WindowEvent::ICCProfChanged => println!("WindowHandler window: event icc prof changed"),
      WindowEvent::DisplayChanged(display_id) => println!(
        "WindowHandler window: event display changed | display_id: {} |",
        display_id
      ),
    }
  }

  pub fn update(
    &mut self,
    delta: f64,
    mouse: &mut MouseController,
    keyboard: &mut KeyboardController,
  ) {
    let mut event_pump = match self.sdl_context.event_pump() {
      Ok(event_pump) => event_pump,
      Err(e) => panic!(
        "WindowHandler: SDL2 context has randomly dissappeared! {}",
        e
      ),
    };

    // poll_iter is going to keep calling poll_event until there are no more events. It's easy mode. :)
    for event in event_pump.poll_iter() {
      // I have allowed my IDE to create all possible events, so we can easily utilize them.
      match event {
        sdl2::event::Event::Quit { timestamp } => {
          println!("sdl2: quit event | timestamp: {} |", timestamp);
          self.quit();
        },
        sdl2::event::Event::AppTerminating { timestamp } => println!("sdl2: termination event | timestamp: {} |", timestamp),
        sdl2::event::Event::AppLowMemory { timestamp } => println!("sdl2: low memory event | timestamp: {} |", timestamp),
        sdl2::event::Event::AppWillEnterBackground { timestamp } => println!("sdl2: will enter background event | timestamp: {} |", timestamp),
        sdl2::event::Event::AppDidEnterBackground { timestamp } => println!("sdl2: did enter background event | timestamp: {} |", timestamp),
        sdl2::event::Event::AppWillEnterForeground { timestamp } => println!("sdl2: will enter foreground event | timestamp: {} |", timestamp),
        sdl2::event::Event::AppDidEnterForeground { timestamp } => println!("sdl2: did enter foreground event | timestamp: {} |", timestamp),
        sdl2::event::Event::Display {
          timestamp,
          display_index,
          display_event,
        } => println!("sdl2: display event | timestamp: {} | display_index: {} | display_event: {:?} |", timestamp, display_index, display_event),
        sdl2::event::Event::Window {
          timestamp,
          window_id,
          win_event,
        } => {
          // println!("sdl2: window event | timestamp: {} | window_id: {} | win_event: {:?} |", timestamp, window_id, win_event);
          self.handle_window_event(win_event);
        },
        sdl2::event::Event::KeyDown {
          timestamp,
          window_id,
          keycode,
          scancode,
          keymod,
          repeat,
        } => {
          // println!("sdl2: keydown event | timestamp: {} | window_id: {} | keycode: {:?} | scancode: {:?} | keymod: {} | repeat: {} |", timestamp, window_id, keycode, scancode, keymod, repeat);
          self.handle_key_event(scancode, keymod, KeyEvent::PressingDown, mouse, keyboard);
        },
        sdl2::event::Event::KeyUp {
          timestamp,
          window_id,
          keycode,
          scancode,
          keymod,
          repeat,
        } => {
          // println!("sdl2: keyup event | timestamp: {} | window_id: {} | keycode: {:?} | scancode: {:?} | keymod: {} | repeat: {} |", timestamp, window_id, keycode, scancode, keymod, repeat);
          self.handle_key_event(scancode, keymod, KeyEvent::LiftedOff, mouse, keyboard);
        },
        sdl2::event::Event::TextEditing {
          timestamp,
          window_id,
          text,
          start,
          length,
        } => println!("sdl2: text editing event | timestamp: {} | window_id: {} | text: {} | start: {} | length: {} |", timestamp, window_id, text, start, length),
        sdl2::event::Event::TextInput {
          timestamp,
          window_id,
          text,
        } => {
          // println!("sdl2: text input event | timestamp: {} | window_id: {} | text: {}", timestamp, window_id, text)
        },
        sdl2::event::Event::MouseMotion {
          timestamp,
          window_id,
          which,
          mousestate,
          x,
          y,
          xrel,
          yrel,
        } => {
          // println!("sdl2: mouse motion event | timestamp: {} | window_id: {} | which: {} | mousestate: {:?} | x: {} | y: {} | xrel: {} | yrel: {} |", timestamp, window_id, which, mousestate, x, y, xrel, yrel);
          self.handle_mouse_motion_event(mousestate, x, y, xrel, yrel, mouse);
        },
        sdl2::event::Event::MouseButtonDown {
          timestamp,
          window_id,
          which,
          mouse_btn,
          clicks,
          x,
          y,
        } => println!("sdl2: mouse button down event | timestamp: {} | window_id: {} | which: {} | mouse_btn: {:?} | clicks: {} | x: {} | y: {} |", timestamp, window_id, which, mouse_btn, clicks, x, y),
        sdl2::event::Event::MouseButtonUp {
          timestamp,
          window_id,
          which,
          mouse_btn,
          clicks,
          x,
          y,
        } => println!("sdl2: mouse button up event | timestamp: {} | window_id: {} | which: {} | mouse_btn: {:?} | clicks: {} | x: {} | y: {} |", timestamp, window_id, which, mouse_btn, clicks, x, y),
        sdl2::event::Event::MouseWheel {
          timestamp,
          window_id,
          which,
          x,
          y,
          direction,
          precise_x,
          precise_y,
        } => println!("sdl2: mouse wheel event | timestamp: {} | window_id: {} | which: {} | x: {} | y: {} | direction: {:?} | precise_x: {} | precise_y: {}", timestamp, window_id, which, x, y, direction, precise_x, precise_y),
        sdl2::event::Event::JoyAxisMotion {
          timestamp,
          which,
          axis_idx,
          value,
        } => println!("sdl2: joy axis motion event | timestamp: {} | which: {} | axis_idx: {} | value: {} |", timestamp, which, axis_idx, value),
        sdl2::event::Event::JoyBallMotion {
          timestamp,
          which,
          ball_idx,
          xrel,
          yrel,
        } => println!("sdl2: joy ball motion event | timestamp: {} | which: {} | ball_idx: {} | xrel: {} | yrel: {} |", timestamp, which, ball_idx, xrel, yrel),
        sdl2::event::Event::JoyHatMotion {
          timestamp,
          which,
          hat_idx,
          state,
        } => println!("sdl2: joy hat motion event | timestamp: {} | which: {} | hat_idx: {} | state: {:?} |", timestamp, which, hat_idx, state),
        sdl2::event::Event::JoyButtonDown {
          timestamp,
          which,
          button_idx,
        } => println!("sdl2: joy button down event | timestamp: {} | which: {} | button_idx: {} |", timestamp, which, button_idx),
        sdl2::event::Event::JoyButtonUp {
          timestamp,
          which,
          button_idx,
        } => println!("sdl2: joy button up event | timestamp: {} | which: {} | button_idx: {} |", timestamp, which, button_idx),
        sdl2::event::Event::JoyDeviceAdded { timestamp, which } => println!("sdl2: joy device added event | timestamp: {} | which: {} |", timestamp, which),
        sdl2::event::Event::JoyDeviceRemoved { timestamp, which } => println!("sdl2: joy device removed event | timestamp: {} | which: {} |", timestamp, which),
        sdl2::event::Event::ControllerAxisMotion {
          timestamp,
          which,
          axis,
          value,
        } => println!("sdl2: controller axis motion event | timestamp: {} | which: {} | axis: {:?} | value: {} |", timestamp, which, axis, value),
        sdl2::event::Event::ControllerButtonDown {
          timestamp,
          which,
          button,
        } => println!("sdl2: controller button down event | timestamp: {} | which: {} | button: {:?} |", timestamp, which, button),
        sdl2::event::Event::ControllerButtonUp {
          timestamp,
          which,
          button,
        } => println!("sdl2: controller button up event | timestamp: {} | which: {} | button: {:?} |", timestamp, which, button),
        sdl2::event::Event::ControllerDeviceAdded { timestamp, which } => println!("sdl2: device added event | timestamp: {} | which: {} |", timestamp, which),
        sdl2::event::Event::ControllerDeviceRemoved { timestamp, which } => println!("sdl2: device removed event | timestamp: {} | which: {} |", timestamp, which),
        sdl2::event::Event::ControllerDeviceRemapped { timestamp, which } => println!("sdl2: device remapped event | timestamp: {} | which: {} |", timestamp, which),
        sdl2::event::Event::ControllerTouchpadDown {
          timestamp,
          which,
          touchpad,
          finger,
          x,
          y,
          pressure,
        } => println!("sdl2: controller touchpad down event | timestamp: {} | which: {} | touchpad: {} | finger: {} | x: {} | y: {} | pressure: {} |", timestamp, which, touchpad, finger, x, y, pressure),
        sdl2::event::Event::ControllerTouchpadMotion {
          timestamp,
          which,
          touchpad,
          finger,
          x,
          y,
          pressure,
        } => println!("sdl2: controller touchpad motion event | timestamp: {} | which: {} | touchpad: {} | finger: {} | x: {} | y: {} | pressure: {} |", timestamp, which, touchpad, finger, x, y, pressure),
        sdl2::event::Event::ControllerTouchpadUp {
          timestamp,
          which,
          touchpad,
          finger,
          x,
          y,
          pressure,
        } => println!("sdl2: controller touchpad up event | timestamp: {} | which: {} | touchpad: {} | finger: {} | x: {} | y: {} | pressure: {} |", timestamp, which, touchpad, finger, x, y, pressure),
        sdl2::event::Event::FingerDown {
          timestamp,
          touch_id,
          finger_id,
          x,
          y,
          dx,
          dy,
          pressure,
        } => println!("sdl2: finger down event | timestamp: {} | touch_id: {} | finger_id: {} | x: {} | y: {} | dx: {} | dy: {} | pressure: {} |", timestamp, touch_id, finger_id, x, y, dx, dy, pressure),
        sdl2::event::Event::FingerUp {
          timestamp,
          touch_id,
          finger_id,
          x,
          y,
          dx,
          dy,
          pressure,
        } => println!("sdl2: finger up event | timestamp: {} | touch_id: {} | finger_id: {} | x: {} | y: {} | dx: {} | dy: {} | pressure: {} |", timestamp, touch_id, finger_id, x, y, dx, dy, pressure),
        sdl2::event::Event::FingerMotion {
          timestamp,
          touch_id,
          finger_id,
          x,
          y,
          dx,
          dy,
          pressure,
        } => println!("sdl2: finger motion event | timestamp: {} | touch_id: {} | finger_id: {} | x: {} | y: {} | dx: {} | dy: {} | pressure: {} |", timestamp, touch_id, finger_id, x, y, dx, dy, pressure),
        sdl2::event::Event::DollarGesture {
          timestamp,
          touch_id,
          gesture_id,
          num_fingers,
          error,
          x,
          y,
        } => println!("sdl2: dollar gesture event | timestamp: {} | touch_id: {} | gesture_id: {} | num_fingers: {} | error: {} | x: {} | y: {} |", timestamp, touch_id, gesture_id, num_fingers, error, x, y),
        sdl2::event::Event::DollarRecord {
          timestamp,
          touch_id,
          gesture_id,
          num_fingers,
          error,
          x,
          y,
        } => println!("sdl2: dollar record event | timestamp: {} | touch_id: {} | gesture_id: {} | num_fingers: {} | error: {} | x: {} | y: {} |", timestamp, touch_id, gesture_id, num_fingers, error, x, y),
        sdl2::event::Event::MultiGesture {
          timestamp,
          touch_id,
          d_theta,
          d_dist,
          x,
          y,
          num_fingers,
        } => println!("sdl2: multi gesture event | timestamp: {} | touch_id: {} | d_theta: {} | d_dist: {} | x: {} | y: {} | num_fingers: {} |", timestamp, touch_id, d_theta, d_dist, x, y, num_fingers),
        sdl2::event::Event::ClipboardUpdate { timestamp } => println!("sdl2: clipboard update event | timestamp: {} |", timestamp),
        sdl2::event::Event::DropFile {
          timestamp,
          window_id,
          filename,
        } => println!("sdl2: drop file event | timestamp: {} | window_id: {} | filename: {} |", timestamp, window_id, filename),
        sdl2::event::Event::DropText {
          timestamp,
          window_id,
          filename,
        } => println!("sdl2: drop text event | timestamp: {} | window_id: {} | filename: {} |", timestamp, window_id, filename),
        sdl2::event::Event::DropBegin {
          timestamp,
          window_id,
        } => println!("sdl2: drop begin event | timestamp: {} | window_id: {} |", timestamp, window_id),
        sdl2::event::Event::DropComplete {
          timestamp,
          window_id,
        } => println!("sdl2: drop complete event | timestamp: {} | window_id: {} |", timestamp, window_id),
        sdl2::event::Event::AudioDeviceAdded {
          timestamp,
          which,
          iscapture,
        } => println!("sdl2: audio device added event | timestamp: {} | which: {} | iscapture: {} |", timestamp, which, iscapture),
        sdl2::event::Event::AudioDeviceRemoved {
          timestamp,
          which,
          iscapture,
        } => println!("sdl2: audio device removed event | timestamp: {} | which: {} | iscapture: {} |", timestamp, which, iscapture),
        sdl2::event::Event::RenderTargetsReset { timestamp } => println!("sdl2: render target reset event | timestamp: {} |", timestamp),
        sdl2::event::Event::RenderDeviceReset { timestamp } => println!("sdl2: render device reset event | timestamp: {} |", timestamp),
        sdl2::event::Event::User {
          timestamp,
          window_id,
          type_,
          code,
          data1,
          data2,
        } => println!("sdl2: user event [custom] | timestamp: {} | window_id: {} | type_: {} | code: {} | data1: {:?} | data2: {:?}", timestamp, window_id, type_, code, data1, data2),
        sdl2::event::Event::Unknown { timestamp, type_ } => println!("sdl2: unknown event [very spooky] | timestamp: {} | type_: {} |", timestamp, type_),
      }
    }
  }
}
