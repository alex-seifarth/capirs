
use super::vsomeipc;
use super::someip;
use std::sync::{Arc, Mutex, Condvar, RwLock};
use std::collections::HashMap;
use std::thread::JoinHandle;

#[derive(Copy, Clone)]
pub enum Command {
    Message(someip::Message)
}

/// Connection handles the communication with the vsomeip layer.
pub struct Connection {
    runtime: vsomeipc::runtime_t,
    application: vsomeipc::application_t,
    application_name: String,
    connection_status: (Mutex<bool>, Condvar),
    services: RwLock<HashMap<someip::ServiceKey, Box<someip::ServiceAdapter<Command>>>>,
}

impl Connection {

    /// Creates a new connection to vsomeip.
    /// A connection retrieves the vsomeip runtime, creates an application, initializes it
    /// and registers a state change callback.
    pub  fn create(app_name: &str) -> Result<Arc<Connection>, ()> {
        let runtime = get_runtime()?;
        let application = create_application(runtime, app_name)?;
        let connection = Arc::new(Connection{
            application,
            runtime,
            connection_status: (Mutex::new(false), Condvar::new()),
            application_name: app_name.to_string(),
            services: RwLock::new(HashMap::new()),
        });

        unsafe{ vsomeipc::application_register_state_handler( application,
            Some(state_changed_callback), &*connection as *const _ as *mut std::os::raw::c_void);
        }
        Ok( connection )
    }

    fn on_state_changed(&self, is_connected: bool) {
        let mut connected = self.connection_status.0.lock().unwrap();
        *connected = is_connected;
        self.connection_status.1.notify_one();
    }

    /// Blocks until the connection towards vsomeip is completely established.
    pub fn wait_until_connected(&self) {
        let connected = self.connection_status.0.lock().unwrap();
        let _guard = self.connection_status.1
            .wait_while(connected, |connected| {!*connected})
            .unwrap();
   }

    /// Starts message processing.
    /// The method starts the message processing by calling the application's start() method in
    /// a newly spawned thread. The thread's join handle is returned. The function blocks until the
    /// connection towards vsomeip is completely established when wait_connected is true.
    pub fn start(self: &Arc<Connection>, wait_connected: bool) -> JoinHandle<()> {
        let clone = self.clone();
        let thread = std::thread::spawn(move || {
            unsafe{ vsomeipc::application_start(clone.application) };
        });
        if wait_connected {
            self.wait_until_connected();
        }
        thread
    }

    /// Stops the message processing - the start method will unblock.
    pub fn stop(&self) {
        unsafe{ vsomeipc::application_stop(self.application) };
    }

    /// Returns the vsomeip application name
    pub fn app_name(&self) -> &str {
        self.application_name.as_str()
    }

    pub fn register_service(&self, siid: someip::ServiceInstanceID, snd: someip::Sender<Command>)
        -> Result<(), ()> {
        let service_key = (siid.service, siid.instance, siid.major_version);
        {
            let mut guard = self.services.write().unwrap();
            if guard.contains_key(&service_key) {
                return Err(())
            }
            let adapter = Box::new(someip::ServiceAdapter { siid: siid.clone(), sender: snd });
            guard.insert(service_key, adapter);

            unsafe {
                vsomeipc::application_register_message_handler(self.application,
                    siid.service, siid.instance, Some(message_received_callback),
                    self as *const _ as *mut std::os::raw::c_void)
            };
        }
        unsafe{ vsomeipc::application_offer_service(self.application, siid.service, siid.instance,
            siid.major_version, siid.minor_version) };
        Ok(())
    }

    pub fn unregister_service(&self, siid: someip::ServiceInstanceID) {
        let service_key = (siid.service, siid.instance, siid.major_version);
        {
            let mut guard = self.services.write().unwrap();
            guard.remove(&service_key);
            if !guard.keys().any(|&k|{k.0 == siid.service && k.1 == siid.instance}) {
                unsafe{ vsomeipc::application_unregister_message_handler(self.application,
                    siid.service, siid.instance);
                }
            }
        }
        unsafe{ vsomeipc::application_stop_offer_service(self.application, siid.service,
             siid.instance, siid.major_version, siid.minor_version) };
    }

    fn process_incoming_message(&self, msg: vsomeipc::message_t) {
        match someip::MessageType::from_u8(unsafe{ vsomeipc::message_get_type(msg) }) {
            someip::MessageType::Request => self.process_service_message(msg),
            someip::MessageType::RequestNoReturn => self.process_service_message(msg),
            someip::MessageType::Response => { todo!(); },
            someip::MessageType::Error => { todo!(); }
            someip::MessageType::Notification => { todo!(); }

            _ => { println!("unsupported message type" )},
        }
    }

    fn process_service_message(&self, msg: vsomeipc::message_t) {
        let service_id = unsafe{ vsomeipc::message_get_service(msg)};
        let instance_id = unsafe{ vsomeipc::message_get_instance(msg)};
        let major_version = unsafe{ vsomeipc::message_get_interface_version(msg)};
        {
            let guard = self.services.read().unwrap();
            if !self.try_forward_incoming_message(&msg, &(service_id, instance_id, major_version), &guard) {
                if !self.try_forward_incoming_message(&msg, &(service_id, instance_id, someip::ANY_MAJOR), &guard) {
                    self.try_forward_incoming_message(&msg, &(service_id, someip::ANY_INSTANCE, someip::ANY_MAJOR), &guard);
                }
            }
        }
    }

    fn try_forward_incoming_message(&self, msg: &vsomeipc::message_t, sk: &someip::ServiceKey,
        map: &HashMap<someip::ServiceKey, Box<someip::ServiceAdapter<Command>>>) -> bool {
        if let Some(service) = map.get(&sk) {
            let message = make_message_from(msg);
            if service.sender.blocking_send(Command::Message(message) ).is_err() {
                // todo log/handle send-failure
            }
            return true;
        }
        false
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        unsafe{ vsomeipc::application_clear_all_handlers(self.application) };
        unsafe{ vsomeipc::application_destroy(self.application) };
        unsafe{ vsomeipc::runtime_release(self.runtime) };
    }
}

unsafe impl Send for Connection {}
unsafe impl Sync for Connection {}

fn get_runtime() -> Result<vsomeipc::runtime_t, ()> {
    let mut runtime : vsomeipc::runtime_t = std::ptr::null_mut();
    if unsafe{ vsomeipc::runtime_get( &mut runtime )} != 0  {
        return Err(())
    }
    Ok( runtime )
}

fn create_application(runtime: vsomeipc::runtime_t, app_name: &str)
    -> Result<vsomeipc::application_t, ()> {
    use std::os::raw::c_char;
    use std::ffi::CString;
    let mut application : vsomeipc::application_t = std::ptr::null_mut();
    let c_str_name = CString::new(app_name).unwrap();
    let c_name: *const c_char = c_str_name.as_ptr() as *const c_char;
    if 0 != unsafe{ vsomeipc::runtime_create_app(runtime, &mut application, c_name)} {
        return Err(())
    }
    if 0 != unsafe{ vsomeipc::application_init(application) } {
        unsafe{ vsomeipc::application_destroy(application)};
        return Err(())
    }
    Ok( application )
}

fn make_message_from(msg: &vsomeipc::message_t) -> someip::Message {
    someip::Message {
        service: unsafe{ vsomeipc::message_get_service(*msg) },
        instance: unsafe{ vsomeipc::message_get_instance(*msg) },
        client: unsafe{ vsomeipc::message_get_client(*msg) },
        session: unsafe{ vsomeipc::message_get_session(*msg) },
        method: unsafe{ vsomeipc::message_get_method(*msg) },
        message_type: someip::MessageType::from_u8(unsafe{ vsomeipc::message_get_type(*msg) } ),
        protocol_version: unsafe{ vsomeipc::message_get_protocol_version(*msg) },
        interface_version: unsafe{ vsomeipc::message_get_interface_version(*msg) },
        return_code: someip::ReturnCode::from_u8(unsafe{ vsomeipc::message_get_return_code(*msg) } ),
        is_reliable: 0 != unsafe{ vsomeipc::message_is_reliable(*msg) },
        is_initial: 0 != unsafe{ vsomeipc::message_is_initial(*msg) },
    }
}

extern "C"
fn state_changed_callback(state: vsomeipc::app_reg_state, context: *mut ::std::os::raw::c_void) {
    let connection = unsafe{(context as *mut Connection).as_ref()}.unwrap();
    connection.on_state_changed(state == vsomeipc::app_reg_state_ARS_REGISTERED);
}

extern "C"
fn message_received_callback(msg: vsomeipc::message_t, context: *mut ::std::os::raw::c_void) {
    let connection = unsafe{(context as *mut Connection).as_ref()}.unwrap();
    connection.process_incoming_message(msg);
}
