
use super::vsomeipc;
use super::someip;
use std::sync::{Arc, Mutex, Condvar, RwLock};
use std::collections::{HashMap};
use std::thread::JoinHandle;
use crate::types::MajorVersion;
use std::os::raw::c_int;

pub type Sender<T> = tokio::sync::mpsc::Sender<T>;
pub type ServiceKey = (someip::ServiceID, someip::InstanceID, someip::MajorVersion);
pub type ProxyServiceKey = (someip::ServiceID, someip::InstanceID);
pub type ProxyID = u64;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct ServiceInstanceID {
    pub service: someip::ServiceID,
    pub instance: someip::InstanceID,
    pub major_version: someip::MajorVersion,
    pub minor_version: someip::MinorVersion,
}

#[derive(Clone)]
pub struct ServiceAdapter<T: Send + 'static> {
    pub siid: ServiceInstanceID,
    pub sender: Sender<T>,
}

#[derive(Clone)]
pub struct ProxyAdapter<T: Send + 'static> {
    pub siid: ServiceInstanceID,
    pub sender: Sender<T>,
    pub proxy_id: ProxyID,
}

#[derive(Copy, Clone, Debug)]
pub enum Command {
    Request(someip::Message),
    ServiceAvailable(someip::ServiceID, someip::InstanceID),
    ServiceUnavailable(someip::ServiceID, someip::InstanceID),
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum CapiError {
    OutOfProxyIds,
    MajorVersionConflict,
}

/// Connection handles the communication with the vsomeip layer.
pub struct Connection {
    runtime: vsomeipc::runtime_t,
    application: vsomeipc::application_t,
    application_name: String,
    connection_status: (Mutex<bool>, Condvar),
    services: RwLock<HashMap<ServiceKey, Box<ServiceAdapter<Command>>>>,
    msg_handler_refs: Mutex<HashMap<(someip::ServiceID, someip::InstanceID), u32>>,
    req_services: RwLock<HashMap<ProxyServiceKey, (MajorVersion, HashMap<ProxyID, ProxyAdapter<Command>>)>>,
    proxy_id_counter: Mutex<ProxyID>,
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
            msg_handler_refs: Mutex::new(HashMap::new()),
            req_services: RwLock::new(HashMap::new()),
            proxy_id_counter: Mutex::new(0),
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

    /// Registers a service provider and begins to forward received SOME/IP message to the
    /// given channel.
    pub fn register_service(&self, siid: ServiceInstanceID, snd: Sender<Command>)
        -> Result<(), ()> {
        let service_key = (siid.service, siid.instance, siid.major_version);
        {
            let mut guard = self.services.write().unwrap();
            if guard.contains_key(&service_key) {
                return Err(())
            }
            let adapter = Box::new(ServiceAdapter { siid: siid.clone(), sender: snd });
            guard.insert(service_key, adapter);
        }
        self.add_msg_handler(siid.service, siid.instance);
        unsafe{ vsomeipc::application_offer_service(self.application, siid.service, siid.instance,
             siid.major_version, siid.minor_version) };
        Ok(())
    }

    /// Unregisters a service provider.
    pub fn unregister_service(&self, siid: ServiceInstanceID) {
        let service_key = (siid.service, siid.instance, siid.major_version);
        {
            let mut guard = self.services.write().unwrap();
            guard.remove(&service_key);
            self.release_msg_handler(siid.service, siid.instance);
            unsafe{ vsomeipc::application_stop_offer_service(self.application, siid.service, siid.instance,
                                                             siid.major_version, siid.minor_version) };
        }
    }

    /// Registers a new proxy to a service. A unique proxy identifier is returned if successful.
    /// The method requests from the service discovery to find the service, registers an availability
    /// handler and installs the message forwarding to the given channel.
    pub fn register_proxy(&self, siid: ServiceInstanceID, sender: Sender<Command>) -> Result<ProxyID, CapiError> {
        let proxy_id = self.create_proxy_id()?;
        let proxy_adapter = ProxyAdapter{ siid, sender, proxy_id };
        let proxy_service_key = (siid.service, siid.instance);
        {
            let mut lock = self.req_services.write().unwrap();
            if let Some(entry) = lock.get_mut(&proxy_service_key) {
                assert!(!entry.1.contains_key(&proxy_id), "proxy id already present im req_services map");
                if entry.0 != siid.major_version {
                    return Err(CapiError::MajorVersionConflict);
                }
                if self.send_actual_availability(siid.service, siid.instance, &proxy_adapter.sender).is_err() {
                    // todo handle send error
                }
                entry.1.insert(proxy_id, proxy_adapter);
            }
            else {
                if self.send_actual_availability(siid.service, siid.instance, &proxy_adapter.sender).is_err() {
                    // todo handle send error
                }
                let mut proxy_map = HashMap::new();
                proxy_map.insert(proxy_id, proxy_adapter);
                lock.insert(proxy_service_key, (siid.major_version, proxy_map));
                unsafe{ vsomeipc::application_request_service(self.application,
                    siid.service, siid.instance, siid.major_version, siid.minor_version) };
                unsafe{ vsomeipc::application_register_availability_callback(self.application,
                    siid.service, siid.instance, Some(availability_callback),
                    self as *const _ as *mut std::os::raw::c_void) };
            }
        }
        self.add_msg_handler(siid.service, siid.instance);
        Ok(proxy_id)
    }

    pub fn unregister_proxy(&self, proxy_id: ProxyID, service: someip::ServiceID, instance: someip::InstanceID) {
        let mut lock = self.req_services.write().unwrap();
        if let Some(svc_entry) = lock.get_mut(&(service, instance)) {
            svc_entry.1.remove(&proxy_id);
            self.release_proxy_id(proxy_id);
            if svc_entry.1.is_empty() {
                unsafe{ vsomeipc::application_release_service(self.application, service, instance) };
                unsafe{ vsomeipc::application_unregister_availability_callback(self.application, service, instance) };
                lock.remove(&(service, instance));
            }
        }
    }

    fn on_availability_callback(&self, service: someip::ServiceID, instance: someip::InstanceID, avail: bool) {
        let lock = self.req_services.read().unwrap();
        if let Some(entry) = lock.get(&(service, instance)) {
            let cmd = bool_to_availability(avail, service, instance);
            for proxy in entry.1.values() {
                if proxy.sender.blocking_send(cmd).is_err() {
                    // todo log/handle send error
                }
            }
        }
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
            if !self.try_forward_service_message(&msg, &(service_id, instance_id, major_version), &guard) {
                if !self.try_forward_service_message(&msg, &(service_id, instance_id, someip::ANY_MAJOR), &guard) {
                    self.try_forward_service_message(&msg, &(service_id, someip::ANY_INSTANCE, someip::ANY_MAJOR), &guard);
                }
            }
        }
    }

    fn try_forward_service_message(&self, msg: &vsomeipc::message_t, sk: &ServiceKey,
                                   map: &HashMap<ServiceKey, Box<ServiceAdapter<Command>>>) -> bool {
        if let Some(service) = map.get(&sk) {
            let message = make_message_from(msg);
            if service.sender.blocking_send(Command::Request(message) ).is_err() {
                // todo log/handle send-failure
            }
            return true;
        }
        false
    }

    fn add_msg_handler(&self, service: someip::ServiceID, instance: someip::InstanceID) {
        let mut lock = self.msg_handler_refs.lock().unwrap();
        let register_needed;
        if let Some(refs) = lock.get_mut(&(service, instance)) {
            register_needed = *refs == 0;
            *refs += 1;
        }
        else {
            lock.insert((service, instance), 1);
            register_needed = true;
        }

        if register_needed {
            unsafe {
                vsomeipc::application_register_message_handler(
                    self.application,
                    service,
                    instance,
                    Some(message_received_callback),
                    self as *const _ as *mut std::os::raw::c_void)
            };
        }
    }

    fn release_msg_handler(&self, service: someip::ServiceID, instance: someip::InstanceID) {
        let mut lock = self.msg_handler_refs.lock().unwrap();
        let mut unregister_needed = false;
        if let Some(refs) = lock.get_mut(&(service, instance)) {
            *refs -= 1;
            unregister_needed = *refs == 0;
        }
        if unregister_needed {
            unsafe {
                vsomeipc::application_unregister_message_handler(
                    self.application,
                    service,
                    instance)
            };
        }
    }

    fn create_proxy_id(&self) -> Result<ProxyID, CapiError> {
        let mut lock = self.proxy_id_counter.lock().unwrap();
        let proxy_id = u64::checked_add(*lock, 1);
        if proxy_id.is_none() {
            return Err(CapiError::OutOfProxyIds)
        }
        *lock = proxy_id.unwrap();
        Ok(*lock)
    }

    fn release_proxy_id(&self, _proxy_id: ProxyID) {
    }

    fn is_service_available(&self, service: someip::ServiceID, instance: someip::InstanceID) -> bool {
        0 < unsafe{ vsomeipc::application_is_available(self.application, service, instance) }
    }

    fn send_actual_availability(&self, service: someip::ServiceID, instance: someip::InstanceID,
        sender: &Sender<Command>) -> Result<(), ()> {
        if sender.blocking_send(bool_to_availability(
            self.is_service_available(service, instance), service, instance )).is_err() {
            return Err(());
        }
        Ok(())
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

fn bool_to_availability(avail: bool, service: someip::ServiceID, instance: someip::InstanceID)
    -> Command {
    if avail {
        Command::ServiceAvailable(service, instance)
    } else {
        Command::ServiceUnavailable(service, instance)
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

extern "C"
fn availability_callback(service: someip::ServiceID, instance: someip::InstanceID, avail: c_int,
                         context: *mut ::std::os::raw::c_void)
{
    let connection = unsafe{(context as *mut Connection).as_ref()}.unwrap();
    connection.on_availability_callback(service, instance, avail > 0);
}