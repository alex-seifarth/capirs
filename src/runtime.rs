use super::*;
use std::sync::Arc;

/// Struct describing a SOME/IP event for its registration.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct EventDescriptor {
    pub id: someip::EventID,
    pub grp: someip::EventGroupID,
    pub typ: someip::EventType,
    pub rel: someip::EventReliability,
}

/// Trait for describing a SOME/IP service.
pub trait ServiceDescriptor {
    /// Type of the stub
    type StubType;

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
                   connection: Arc<super::Connection>,
                   runtime: Arc<Runtime>) -> Self::StubType;
}

/// The runtime allows users of 'capirs' to create service stubs and service proxies.
/// Applications should only create one runtime object because the underlying vsomeip application
/// object has an heavy footprint.
pub struct Runtime {
    connection: Arc<Connection>,
}

impl Runtime {

    /// Create a new runtime object packed in a shareable Arc.
    /// - [app_name]: Name of the vsomeip application (will appear in vsomeip logs).
    pub async fn create(app_name: &str) -> Arc<Runtime> {
        let connection = Connection::create(app_name).unwrap();
        connection.start(true).await;
        Arc::new(Runtime{connection})
    }

    /// Creates a new service for the given service descriptor and for the given [instance]. This
    /// will start to offer the service instance on SOME/IP SD and also register and offer all
    /// events defined by the [ServiceDescriptor].
    /// - [instance]: The instance ID for the service.
    pub async fn create_service<T: ServiceDescriptor>(self: &Arc<Runtime>, instance: someip::InstanceID) -> Result<T::StubType, super::CapiError> {
        let channel = tokio::sync::mpsc::channel(1024);
        let version = T::version();
        let svc = super::ServiceInstanceID {service: T::service_id(), instance,
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

    /// Removes a service instance from the system, so that it will no longer be offered.
    pub fn remove_service<T: ServiceDescriptor>(self: &Arc<Runtime>, instance: someip::InstanceID) {
        let version = T::version();
        let svc = super::ServiceInstanceID {service: T::service_id(), instance,
            major_version: version.0, minor_version: version.1};
        self.connection.unregister_service(svc);
    }
}
