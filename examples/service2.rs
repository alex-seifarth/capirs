use std::sync::Arc;
use capirs::*;
// use bytes::BufMut;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct EventDescriptor {
    id: someip::EventID,
    grp: someip::EventGroupID,
    typ: someip::EventType,
    rel: someip::EventReliability,
}

/// Trait for describing a SOME/IP service.
trait ServiceDescriptor {
    type StubType;

    /// Type representing the service's request messages (typically an enum).
    type Request;

    /// Type representing the service's response messages (typically an enum).
    type Response;

    /// Type representing the service's notification messages (typically an enum).
    type Notification;

    /// Returns the SOME/IP service identifier for the service.
    fn service_id() -> someip::ServiceID;

    /// Returns the interface version (major, minor) for the service instances of this service.
    fn version() -> (someip::MajorVersion, someip::MinorVersion);

    /// This method must return an array of event descriptors that the service will provide.
    /// This is used when a new service instance is being created to register the events with
    /// the SOME/IP runtime.
    fn event_descriptors(instance: someip::InstanceID) -> std::vec::Vec<EventDescriptor>;

    /// Creates a stub for the service that can drive the receiver where requests from consumers
    /// will be received.
    fn create_stub(instance: someip::InstanceID,
                    receiver: tokio::sync::mpsc::Receiver<someip::Command>,
                    connection: Arc<capirs::Connection>,
                    runtime: Arc<Runtime>) -> Self::StubType;
}

struct Runtime {
    connection: Arc<Connection>,
}

impl Runtime {

    pub async fn create(app_name: &str) -> Arc<Runtime> {
        let connection = Connection::create(app_name).unwrap();
        connection.start(true).await;
        Arc::new(Runtime{connection})
    }

    pub async fn create_service<T: ServiceDescriptor>(self: &Arc<Runtime>, instance: someip::InstanceID) -> Result<T::StubType, capirs::CapiError> {
        let channel = tokio::sync::mpsc::channel(1024);
        let version = T::version();
        let svc = capirs::ServiceInstanceID {service: T::service_id(), instance,
            major_version: version.0, minor_version: version.1};
        self.connection.register_service(svc.clone(), channel.0).await.unwrap();

        for ed in T::event_descriptors(instance) {
            if let Err(err) =  self.connection.register_event(T::service_id(), instance, ed.id, ed.grp,
                ed.typ, ed.rel).await {
                self.connection.unregister_service(svc);
                return Err(err);
            }
        }
        Ok(T::create_stub(instance, channel.1, self.connection.clone(), self.clone()))
    }

    pub fn remove_service<T: ServiceDescriptor>(self: &Arc<Runtime>, instance: someip::InstanceID) {
        let version = T::version();
        let svc = capirs::ServiceInstanceID {service: T::service_id(), instance,
            major_version: version.0, minor_version: version.1};
        self.connection.unregister_service(svc);
    }

}

#[derive(Debug)]
enum MyServiceMessage {
    Request1{request: someip::Message, data: std::string::String},
    Request2{request: someip::Message, data: u32},
    Request3{request: someip::Message},
}

struct MyService {
    instance: someip::InstanceID,
    connection: Arc<capirs::Connection>,
    receiver: tokio::sync::mpsc::Receiver<someip::Command>,
    runtime: Arc<Runtime>
}

impl ServiceDescriptor for MyService {
    type StubType = Self;
    type Request = ();
    type Response = ();
    type Notification = ();

    fn service_id() -> someip::ServiceID { 0x1111 }

    fn version() -> (someip::MajorVersion, someip::MinorVersion) { (1, 0) }

    fn event_descriptors(_instance: someip::InstanceID) -> std::vec::Vec<EventDescriptor> {
        vec![
            EventDescriptor { id: 0x8001, grp: 1, typ: someip::EventType::Broadcast, rel: someip::EventReliability::Service }
        ]
    }

    fn create_stub(instance: someip::InstanceID, receiver: tokio::sync::mpsc::Receiver<someip::Command>,
                   connection: Arc<Connection>, runtime: Arc<Runtime>) -> Self::StubType {
        MyService { instance, connection, receiver, runtime }
    }
}

impl MyService {
    pub async fn recv(&mut self) -> Option<MyServiceMessage> {
        match self.receiver.recv().await {
            None => { return None; },
            Some(msg) => {
                return match msg {
                    someip::Command::Request(header, payload) => self.process_request(header,  payload),
                    _ => None,
                };
            }
        }
    }

    pub async fn reply_to_request1(&self, request: &someip::Message, return_code: someip::ReturnCode,
            payload: Option<bytes::Bytes>) -> Result<(), capirs::CapiError> {
        self.connection.send_response(request, return_code, payload).await
    }

    pub async fn reply_request2(&self, request: &someip::Message, return_code: someip::ReturnCode,
            payload: Option<bytes::Bytes>) -> Result<(), capirs::CapiError> {
        self.connection.send_response(request, return_code, payload).await
    }

    pub async fn send_notification_1(&self, force: bool, payload: Option<bytes::Bytes>) {
        self.connection.send_notification(Self::service_id(), self.instance, 0x8001, payload, force).await
    }

    fn process_request(&self, header: someip::Message, payload: Option<bytes::Bytes>) -> Option<MyServiceMessage> {
        return match header.method {
            1 => {
                if let Some(data) = payload {
                    Some( MyServiceMessage::Request1{
                        request: header, data: std::str::from_utf8(&data as &[u8]).unwrap().to_string() })
                }
                else {
                    Some(MyServiceMessage::Request1{ request:header, data: std::string::String::new() })
                }
            },
            2 => Some( MyServiceMessage::Request2{request: header, data: 42 }),
            3 => Some( MyServiceMessage::Request3{request: header}),
            _ => None
        }
    }
}

impl Drop for MyService {
    fn drop(&mut self) {
        self.runtime.remove_service::<MyService>(self.instance);
    }
}


#[tokio::main]
pub async fn main() {
    let runtime = Runtime::create("service2").await;
    let mut my_service_stub = match runtime.create_service::<MyService>(0x2222).await {
        Ok(stub) => stub,
        Err(err) => { panic!("Cannot create stub: {:?}", err); }
    };

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                break;
            },
            Some(msg) = my_service_stub.recv() => {
                match msg {
                    MyServiceMessage::Request1{request, data} => {
                        println!("Request1 [{:?}]", data);
                        my_service_stub.reply_to_request1(&request, someip::ReturnCode::Ok,
                            Some(bytes::Bytes::from("F.ck the morning .. "))).await.unwrap();
                    },
                    MyServiceMessage::Request2{request, data} => {
                        println!("Request2 [{:?}]", data);
                        my_service_stub.reply_request2(&request, someip::ReturnCode::ApplicationError(22),
                            None).await.unwrap();
                    },
                    MyServiceMessage::Request3{request:_} => {
                        // don't react -> client should get a timeout
                        // let mut payload = bytes::BytesMut::new();
                        // payload.put_u32(0x4711);
                        my_service_stub.send_notification_1(false, None).await; //Some(payload.freeze())).await;
                    }
                }

            },
        };
    }

    drop(my_service_stub);
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
}