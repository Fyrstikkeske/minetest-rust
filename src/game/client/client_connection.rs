use std::{cell::RefCell, net::ToSocketAddrs, rc::Rc};

use message_io::{network::Transport, node};

use super::Client;

///
/// ClientConnection and Client can be considered 1 entity.
///
/// This is why client_pointer is not an Option<>.
///
pub struct ClientConnection<'client> {
  address: String,
  port: i32,

  client_pointer: Rc<RefCell<Client<'client>>>,
}

impl<'client> ClientConnection<'client> {
  pub fn new(client_pointer: Rc<RefCell<Client<'client>>>, address: String, port: i32) -> Self {
    let mut new_client_connection = ClientConnection {
      address,
      port,

      client_pointer,
    };

    new_client_connection.initialize();

    new_client_connection
  }

  ///
  /// Change the address that the server connection will utilize.
  ///
  pub fn set_address(&mut self, new_address: String) {
    self.address = new_address;
  }

  ///
  /// Change the port that the server connection will utilize.
  ///
  pub fn set_port(&mut self, new_port: i32) {
    self.port = new_port;
  }

  ///
  /// Construct the address & port into a parsable socket string.
  ///
  pub fn get_socket(&self) -> String {
    let mut socket = self.address.clone();
    socket.push(':');
    socket.push_str(self.port.to_string().as_str());

    socket
  }

  ///
  /// Internal initializer procedure automatically run on a new ServerConnection.
  ///
  fn initialize(&mut self) {
    let socket_address = self.get_socket().to_socket_addrs().unwrap().next().unwrap();
    let transport_protocol = Transport::Udp;

    // todo: will need to do a handshake here.
    // todo: will need to be initialized by the gui component.

    let (handler, listener) = node::split::<()>();

    let (server_id, local_address) = match handler
      .network()
      .connect(transport_protocol, socket_address)
    {
      Ok((end_point, socket_address)) => (end_point, socket_address),
      Err(e) => panic!("{}", e),
    };
  }
}

impl<'client> Drop for ClientConnection<'client> {
  fn drop(&mut self) {
    // Need to close client connection, maybe?
    // Might need to send out the disconnect signal.
    // todo: experiment with this.
    println!("ClientConnection dropped!")
  }
}
