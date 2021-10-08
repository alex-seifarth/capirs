
use super::someip;
use super::vsomeipc;

//#[cfg(feature="async-tokio")]
pub type Sender<T> = tokio::sync::mpsc::Sender<T>;

//#[cfg(not(feature="async-tokio"))]
//pub type Sender<T> = std::sync::mpsc::Sender<T>;

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

impl<T: Send + 'static> ServiceAdapter<T> {

    pub fn setup(app: &vsomeipc::application_t, siid: ServiceInstanceID, sender: Sender<T>)
        -> ServiceAdapter<T> {


        ServiceAdapter{ siid, sender}
    }
}


