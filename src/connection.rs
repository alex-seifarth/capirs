
use super::vsomeipc;
use super::someip::*;
use std::sync::{Arc, Mutex, Condvar, RwLock};
use std::collections::{HashMap, HashSet};
use crate::types::{MajorVersion, ANY_INSTANCE};
use std::os::raw::c_int;
use std::sync::mpsc::RecvTimeoutError;

pub type Sender<T> = tokio::sync::mpsc::Sender<T>;
pub type ServiceKey = (ServiceID, InstanceID);
pub type ProxyServiceKey = (ServiceID, InstanceID);
pub type ProxyID = u64;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct ServiceInstanceID {
    pub service: ServiceID,
    pub instance: InstanceID,
    pub major_version: MajorVersion,
    pub minor_version: MinorVersion,
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

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum CapiError {
    OutOfProxyIds,
    MajorVersionConflict,
    InvalidMessageType,
    NotImplemented,
    ServiceInstanceUnknown,
    ProxyIdUnknown,
    EventAlreadyRegistered,
}

/// Connection handles the communication with the vsomeip layer.
pub struct Connection {
    runtime: vsomeipc::runtime_t,
    application: vsomeipc::application_t,
    application_name: String,
    connection_status: (Mutex<bool>, Condvar),
    services: RwLock<HashMap<ServiceKey, Box<ServiceAdapter<Command>>>>,
    msg_handler_refs: Mutex<HashMap<(ServiceID, InstanceID), u32>>, // u32 = ref-count
    req_services: RwLock<HashMap<ProxyServiceKey, (MajorVersion, HashMap<ProxyID, ProxyAdapter<Command>>)>>,
    proxy_id_counter: Mutex<ProxyID>,
    processing_thread: Mutex<Option<std::thread::JoinHandle<()>>>,
    session_map: Mutex<HashMap<(ClientID, SessionID), (Sender<Command>, u32)>>, // u32 = time in secs to timeout
    cleanup_thread_jh: Mutex<Option<(std::thread::JoinHandle<()>, std::sync::mpsc::Sender<bool>)>>,
    offered_events: Mutex<HashSet<(ServiceID, InstanceID, EventID)>>,
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
            processing_thread: Mutex::new(None),
            session_map: Mutex::new(HashMap::new()),
            cleanup_thread_jh: Mutex::new(None),
            offered_events: Mutex::new(HashSet::new()),
        });

        unsafe{ vsomeipc::application_register_state_handler( application,
            Some(state_changed_callback), &*connection as *const _ as *mut std::os::raw::c_void);
        }
        Ok( connection )
    }

    fn start_cleanup_thread(self: &Arc<Self>) {
        let mut guard = self.cleanup_thread_jh.lock().unwrap();
        if guard.is_some() {
            log::error!("Cleanup thread of connection already started ... fatal.");
            panic!("tried to start cleanup thread twice");
        }
        let conn_clone = self.clone();
        let (snd, rcv) = std::sync::mpsc::channel();
        *guard = Some((std::thread::spawn(move || {
            loop {
                match rcv.recv_timeout(std::time::Duration::from_secs(1)) {
                    Ok(true) => break,
                    Err(RecvTimeoutError::Disconnected) => break,
                    Ok(false) => {},
                    Err(RecvTimeoutError::Timeout) => { conn_clone.cleanup(); },
                };
            }
        }), snd));
    }

    fn cleanup(self: &Arc<Self>) {
        let mut guard = self.session_map.lock().unwrap();
        guard.retain(|(client,session), (snd, to)| {
            *to -= 1;
            if *to ==  0 {
                let _ = snd.blocking_send(Command::Timeout(*client, *session));
                return false;
            }
            true
        });
    }

    fn on_state_changed(&self, is_connected: bool) {
        let mut connected = self.connection_status.0.lock().unwrap();
        *connected = is_connected;
        self.connection_status.1.notify_one();
    }

    /// Blocks until the connection towards vsomeip is completely established.
    fn wait_until_connected(&self) {
        let connected = self.connection_status.0.lock().unwrap();
        let _guard = self.connection_status.1
            .wait_while(connected, |connected| {!*connected})
            .unwrap();
   }

    /// Starts message processing.
    /// The method starts the message processing by calling the application's start() method in
    /// a newly spawned thread.
    pub async fn start(self: &Arc<Connection>, wait_connected: bool) {
        let mut guard = self.processing_thread.lock().unwrap();
        if guard.is_some() {
            log::error!("Application tried to start connection twice, ignored.");
            return;
        }
        let clone = self.clone();
        self.start_cleanup_thread();
        *guard = Some(std::thread::spawn(move || {
            unsafe{ vsomeipc::application_start(clone.application) };
        }));
        let self_clone = self.clone();
        if wait_connected {
            tokio::task::spawn_blocking(move || {self_clone.wait_until_connected()} ).await.unwrap();
        }
    }

    /// Stops the message processing - the start method will unblock.
    pub async fn stop(&self) {
        {
            let mut cleanup_guard = self.cleanup_thread_jh.lock().unwrap();
            if let Some((jh, snd)) = cleanup_guard.take() {
                let _ = snd.send(true);
                let _ = jh.join();
            }
        }
        unsafe{ vsomeipc::application_stop(self.application) };
        {
            let mut guard = self.processing_thread.lock().unwrap();
            if let Some(join_handle) = guard.take() {
                let _ = tokio::task::spawn_blocking(move || { let _ = join_handle.join(); }).await;
            }
        }
    }

    /// Returns the vsomeip application name
    pub fn app_name(&self) -> &str {
        self.application_name.as_str()
    }

    /// Registers a service provider and begins to forward received SOME/IP message to the
    /// given channel.
    pub async fn register_service(&self, siid: ServiceInstanceID, snd: Sender<Command>)
        -> Result<(), ()> {
        let service_key = (siid.service, siid.instance);
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
        {
            let mut oe_guard = self.offered_events.lock().unwrap();
            oe_guard.retain(|(service, instance, event) | {
                if *service == siid.service && *instance == siid.instance {
                    unsafe { vsomeipc::application_stop_offer_event(self.application, *service, *instance, *event) };
                    return false;
                }
                true
            });
        }
        let service_key = (siid.service, siid.instance);
        {
            let mut guard = self.services.write().unwrap();
            guard.remove(&service_key);
            self.release_msg_handler(siid.service, siid.instance);
            unsafe{ vsomeipc::application_stop_offer_service(self.application, siid.service, siid.instance,
                                                             siid.major_version, siid.minor_version) };
        }
    }

    /// Register an event from a service - the event will then be offered via the SOME/IP SD.
    pub async fn register_event(&self, service: ServiceID, instance: InstanceID,
        event: EventID, event_group: EventGroupID, event_type: EventType,
        reliability: EventReliability)  -> Result<(), CapiError>
    {
        {
            let svc_guard = self.services.read().unwrap();
            if !svc_guard.contains_key(&(service, instance)) {
                return Err(CapiError::ServiceInstanceUnknown);
            }
        }

        let mut oe_guard = self.offered_events.lock().unwrap();
        if oe_guard.contains(&(service, instance, event)) {
            return Err(CapiError::EventAlreadyRegistered);
        }
        oe_guard.insert((service, instance, event));
        unsafe{
            vsomeipc::application_offer_event(self.application, service, instance, event,
            event_type.to_c(), reliability.to_c(), &event_group as *const u16, 1)
        };
        Ok(())
    }

    /// Unregisters an event - SOME/IP SD will stop offering it.
    pub async fn unregister_event(&self, service: ServiceID, instance: InstanceID,
                                  event: EventID) {
        let mut oe_guard = self.offered_events.lock().unwrap();
        if oe_guard.contains(&(service, instance, event)) {
            unsafe { vsomeipc::application_stop_offer_event(self.application, service, instance, event) };
            oe_guard.remove(&(service, instance, event));
        }
    }

    /// Registers a new proxy to a service. A unique proxy identifier is returned if successful.
    /// The method requests from the service discovery to find the service, registers an availability
    /// handler and installs the message forwarding to the given channel.
    pub async fn register_proxy(&self, siid: ServiceInstanceID, sender: Sender<Command>) -> Result<ProxyID, CapiError> {
        let proxy_id = self.create_proxy_id()?;
        let sender_clone = sender.clone();
        let proxy_adapter = ProxyAdapter{ siid, sender, proxy_id };
        let proxy_service_key = (siid.service, siid.instance);
        {
            let mut lock = self.req_services.write().unwrap();
            if let Some(entry) = lock.get_mut(&proxy_service_key) {
                assert!(!entry.1.contains_key(&proxy_id), "proxy id already present im req_services map");
                if entry.0 != siid.major_version {
                    return Err(CapiError::MajorVersionConflict);
                }
                entry.1.insert(proxy_id, proxy_adapter);
            }
            else {
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
        if self.send_actual_availability(siid.service, siid.instance, &sender_clone).await.is_err() {
            // todo handle send error
        }
        self.add_msg_handler(siid.service, siid.instance);
        Ok(proxy_id)
    }

    /// Unregisters a previously registered proxy to a service.
    pub fn unregister_proxy(&self, proxy_id: ProxyID, service: ServiceID, instance: InstanceID) {
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

    /// Send a notification to all subscribed consumers.
    /// Pure events and selective events are always sent out, field events are only sent when
    /// data has changed or @force is true.
    pub async fn send_notification(&self, service: ServiceID, instance: InstanceID,
                event: EventID, data: Option<bytes::Bytes>, force: bool) {
        if let Some(msg_data) = data {
            let payload = unsafe { vsomeipc::runtime_create_payload(self.runtime,
                                            msg_data.as_ref().as_ptr(), msg_data.len() as u32) };

            unsafe{ vsomeipc::application_notify(self.application, service, instance, event, payload,
                if force {1} else {0}) };
            unsafe{ vsomeipc::payload_destroy(payload) };
        }
        else {
            unsafe{ vsomeipc::application_notify(self.application, service, instance, event,
                                                 std::ptr::null_mut(), if force {1} else {0}) };
        }
    }

    /// Send a request to the given service/instance.
    pub async fn send_request(&self, proxy_id: ProxyID, service: ServiceID,
                              instance: InstanceID, method: MethodID,
                              fire_and_forget: bool, reliable: bool, data: Option<bytes::Bytes>)
            -> Result<Option<(ClientID, SessionID)>, CapiError>{
        let (mjr_version, sender) = {
            let lock = self.req_services.read().unwrap();
            match lock.get(&(service, instance)) {
                Some(entry) => {
                    let proxy_adapter = entry.1.get(&proxy_id);
                    if proxy_adapter.is_none() {
                        return Err(CapiError::ProxyIdUnknown);
                    }
                    (entry.0, proxy_adapter.unwrap().sender.clone())
                },
                None=> {return Err(CapiError::ServiceInstanceUnknown);},
            }
        };

        let msg = unsafe{ vsomeipc::runtime_create_request(self.runtime, service, instance, method,
            mjr_version, if fire_and_forget {1} else {0}, if reliable {1} else {0}) };
        self.send_message(msg, data);

        let request_id = if !fire_and_forget {
            let mut session_lock = self.session_map.lock().unwrap();
            let request_id = ( unsafe{ vsomeipc::message_get_client(msg) }, unsafe{ vsomeipc::message_get_session(msg) } );
            assert!(!session_lock.contains_key(&request_id), "request id already in use");
            session_lock.insert(request_id.clone(), (sender, 5));
            Some(request_id)
        }
        else {
            None
        };
        unsafe{ vsomeipc::message_destroy(msg) };
        Ok(request_id)
    }

    /// Send a response for the given request message. The response can be either a
    /// response message if the @return_code is Ok or an error message otherwise.
    pub async fn send_response(&self, request: &Message, return_code: ReturnCode,
                               data: Option<bytes::Bytes>) -> Result<(), CapiError> {
        self.send_reply(request.service, request.instance, request.method, return_code, request.is_reliable,
            request.client, request.session, request.interface_version, data)
    }

    fn send_reply(&self, service: ServiceID, instance: InstanceID,
                  method: MethodID, return_code: ReturnCode, reliable: bool,
                  client_id: ClientID, session_id: SessionID,
                  mjr_version: MajorVersion, data: Option<bytes::Bytes>) -> Result<(), CapiError> {
        let message = match return_code {
            ReturnCode::Ok => unsafe{
                vsomeipc::runtime_create_response(self.runtime, service, instance, client_id,
                                                  session_id, method, mjr_version,
                                                  if reliable {1} else {0}) },
            _ => unsafe{
                vsomeipc::runtime_create_error(self.runtime, service, instance, client_id,
                                               session_id, method, mjr_version,
                                               if reliable {1} else {0}, return_code.value()) },
        };
        self.send_message(message, data);
        unsafe{ vsomeipc::message_destroy(message) };
        Ok(())
    }

    fn send_message(&self, msg: vsomeipc::message_t, data: Option<bytes::Bytes>) {
        if let Some(msg_data) = data {
            assert!(msg_data.len() < (u32::MAX as usize));
            let payload = unsafe{ vsomeipc::runtime_create_payload(self.runtime,
                           msg_data.as_ref().as_ptr(), msg_data.len() as u32)};
            unsafe{ vsomeipc::application_send(self.application, msg, payload) };
            unsafe{ vsomeipc::payload_destroy(payload) };
        }
        else {
            unsafe{ vsomeipc::application_send(self.application, msg, std::ptr::null_mut()) };
        }
    }

    fn on_availability_callback(&self, service: ServiceID, instance: InstanceID, avail: bool) {
        let lock = self.req_services.read().unwrap();
        if let Some(entry) = lock.get(&(service, instance)) {
            let cmd = bool_to_availability(avail, service, instance);
            for proxy in entry.1.values() {
                if proxy.sender.blocking_send(cmd.clone()).is_err() {
                    // todo log/handle send error
                }
            }
        }
    }

    fn process_incoming_message(&self, msg: vsomeipc::message_t) {
        match MessageType::from_u8(unsafe{ vsomeipc::message_get_type(msg) }) {
            MessageType::Request => self.process_service_message(msg),
            MessageType::RequestNoReturn => self.process_service_message(msg),
            MessageType::Response => { self.process_response_message(msg); },
            MessageType::Error => { self.process_error_message(msg); }
            MessageType::Notification => { todo!(); }

            msg => { log::warn!("unsupported message type: {:?}", msg); },
        }
    }

    fn process_error_message(&self, msg: vsomeipc::message_t) {
        let client_id = unsafe{ vsomeipc::message_get_client(msg) };
        let session_id = unsafe{ vsomeipc::message_get_session(msg) };
        {
            let mut guard = self.session_map.lock().unwrap();
            if let Some(session) = guard.get(&(client_id, session_id)) {
                let _ = session.0.blocking_send(Command::Error(make_message_from(&msg),
                                                                make_payload_from(&msg)));
                guard.remove(&(client_id, session_id));
            }
            else {
                log::info!("received response for unknown session ({:4x}.{:4x})", client_id, session_id);
            }
        }
    }

    fn process_response_message(&self, msg: vsomeipc::message_t) {
        let client_id = unsafe{ vsomeipc::message_get_client(msg) };
        let session_id = unsafe{ vsomeipc::message_get_session(msg) };
        {
            let mut guard = self.session_map.lock().unwrap();
            if let Some(session) = guard.get(&(client_id, session_id)) {
                let _ = session.0.blocking_send(Command::Response(make_message_from(&msg),
                                                        make_payload_from(&msg)));
                guard.remove(&(client_id, session_id));
            }
            else {
                log::info!("received error for unknown session ({:4x}.{:4x})", client_id, session_id);
            }
        }
    }

    fn process_service_message(&self, msg: vsomeipc::message_t) {
        let service_id = unsafe{ vsomeipc::message_get_service(msg)};
        let instance_id = unsafe{ vsomeipc::message_get_instance(msg)};
        {
            let guard = self.services.read().unwrap();
            if !self.try_forward_service_message(&msg, &(service_id, instance_id), &guard) {
                self.try_forward_service_message(&msg, &(service_id, ANY_INSTANCE), &guard);
            }
        }
    }

    fn try_forward_service_message(&self, msg: &vsomeipc::message_t, sk: &ServiceKey,
                                   map: &HashMap<ServiceKey, Box<ServiceAdapter<Command>>>) -> bool {
        if let Some(service) = map.get(&sk) {
            let message = make_message_from(msg);
            let payload = make_payload_from(msg);
            if service.sender.blocking_send(Command::Request(message, payload) ).is_err() {
                // todo log/handle send-failure
            }
            return true;
        }
        false
    }

    fn add_msg_handler(&self, service: ServiceID, instance: InstanceID) {
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

    fn release_msg_handler(&self, service: ServiceID, instance: InstanceID) {
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

    fn is_service_available(&self, service: ServiceID, instance: InstanceID) -> bool {
        0 < unsafe{ vsomeipc::application_is_available(self.application, service, instance) }
    }

    async fn send_actual_availability(&self, service: ServiceID, instance: InstanceID,
        sender: &Sender<Command>) -> Result<(), ()> {
        if sender.send(bool_to_availability(
            self.is_service_available(service, instance), service, instance )).await.is_err() {
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

fn make_message_from(msg: &vsomeipc::message_t) -> Message {
    Message {
        service: unsafe{ vsomeipc::message_get_service(*msg) },
        instance: unsafe{ vsomeipc::message_get_instance(*msg) },
        client: unsafe{ vsomeipc::message_get_client(*msg) },
        session: unsafe{ vsomeipc::message_get_session(*msg) },
        method: unsafe{ vsomeipc::message_get_method(*msg) },
        message_type: MessageType::from_u8(unsafe{ vsomeipc::message_get_type(*msg) } ),
        protocol_version: unsafe{ vsomeipc::message_get_protocol_version(*msg) },
        interface_version: unsafe{ vsomeipc::message_get_interface_version(*msg) },
        return_code: ReturnCode::from_u8(unsafe{ vsomeipc::message_get_return_code(*msg) } ),
        is_reliable: 0 != unsafe{ vsomeipc::message_is_reliable(*msg) },
        is_initial: 0 != unsafe{ vsomeipc::message_is_initial(*msg) },
    }
}

fn make_payload_from(msg: &vsomeipc::message_t) -> Option<bytes::Bytes>
{
    let mut length: u32 = 0;
    let data = unsafe{ vsomeipc::message_get_data(*msg,(&mut length) as *mut u32) };
    if length == 0 || data.is_null() {
        return None
    }
    let mut payload = bytes::BytesMut::with_capacity(length as usize);
    unsafe {
        data.copy_to(payload.as_mut_ptr(), length as usize);
        payload.set_len(length as usize);
    }
    Some(payload.freeze())
}

fn bool_to_availability(avail: bool, service: ServiceID, instance: InstanceID)
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
    unsafe{ vsomeipc::message_destroy(msg) };
}

extern "C"
fn availability_callback(service: ServiceID, instance: InstanceID, avail: c_int,
                         context: *mut ::std::os::raw::c_void)
{
    let connection = unsafe{(context as *mut Connection).as_ref()}.unwrap();
    connection.on_availability_callback(service, instance, avail > 0);
}