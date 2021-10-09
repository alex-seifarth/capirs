/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/
use super::vsomeipc::*;

use std::ffi::CString;
use std::sync::{Mutex, RwLock, Arc};

pub struct Runtime {
    runtime: runtime_t,
}

impl Runtime {
    pub fn create() -> Runtime {
        let mut runtime : runtime_t = std::ptr::null_mut();
        if unsafe{ runtime_get( &mut runtime )} != 0  {
            panic!("unable to get runtime");
        }
        Runtime{ runtime }
    }

    pub fn create_application(&self, name: &str) -> Result<Arc<Application>, ()> {
        use std::os::raw::c_char;
        let mut application : application_t = std::ptr::null_mut();
        let c_str_name = CString::new(name).unwrap();
        let c_name: *const c_char = c_str_name.as_ptr() as *const c_char;
        if 0 != unsafe{ runtime_create_app(self.runtime, &mut application, c_name)} {
            return Err(())
        }
        if 0 != unsafe{ application_init(application) } {
            unsafe{ application_destroy(application)};
            return Err(())
        }
        Ok( Application::new(application, name) )
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        unsafe{ runtime_release( self.runtime )};
        self.runtime = std::ptr::null_mut();
    }
}

unsafe impl Send for Runtime {}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AppRegistrationState {
    Registered,
    NotRegistered,
}

/// Represents a vsomeip application object.
/// vsomeip application objects are the main entry to register services, find services, find
/// event groups and register for them and finally to send and receive SOME/IP communication
/// messages.
/// An application object has an heavy footprint and such objects should only be created sparsely.
/// Typically only one application object is used per process.
///
/// Application objects are created by the runtime (see Runtime::create_application).
///
/// The application object wrapper for Rust is Send + Sync and can also be cloned. The latter costs
/// an allocation of a C++ std::shared_ptr on the heap.
pub struct Application {
    application: application_t,
    name: String,
    state_handlers: RwLock<Vec<Box<dyn Fn(AppRegistrationState) + Send + Sync + 'static>>>,
}

impl Application {

    fn new(application: application_t, name: &str) -> Arc<Application> {
        let mut app = Arc::new (Application{
            application,
            name: name.to_string(),
            state_handlers: RwLock::new( Vec::new() )
        });

        unsafe { application_register_state_handler(app.application, Some(Application::state_callback),
                                   &*app as *const _ as *mut std::os::raw::c_void); }
        app
    }

    extern "C" fn state_callback(state: app_reg_state, context: *mut ::std::os::raw::c_void) {
        let app = unsafe{(context as *mut Application).as_ref()}.unwrap();
        app.on_state_changed(state);
    }

    fn on_state_changed(&self, state: app_reg_state) {
        let shmx = self.state_handlers.read().unwrap();
        let st = match state {
            super::vsomeipc::app_reg_state_ARS_REGISTERED => AppRegistrationState::Registered,
            _ => AppRegistrationState::NotRegistered
        };
        for handler in &*shmx {
            handler(st);
        }
    }

    /// The start method enters the event processing loop of the application. So this method will
    /// block until the runtime or application shuts down (see Application::stop).
    pub fn start(&self) {
        unsafe{ application_start( self.application ) };
    }

    /// This stops the application's event processing and the Application::start method will return.
    /// Hence stop must be called by another thread.
    pub fn stop(&self) {
        unsafe{ application_stop( self.application ) };
    }

    pub fn register_state_handler<F: Fn(AppRegistrationState) + Send + Sync + 'static>(&self, cbk: F) {
        let mut shmx = self.state_handlers.write().unwrap();
        shmx.push(Box::new(cbk));
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn offer_service(&self, service: ServiceID, instance: InstanceID, major: MajorVersion,
                         minor: MinorVersion) {
        unsafe{ application_offer_service(self.application, service, instance, major, minor) };
    }

    pub fn stop_offer_service(&self, service: ServiceID, instance: InstanceID, major: MajorVersion,
                         minor: MinorVersion) {
        unsafe{ application_stop_offer_service(self.application, service, instance, major, minor) };
    }

}

impl Drop for Application {
    fn drop(&mut self) {
        unsafe{ application_unregister_state_handler( self.application )};
        unsafe{ application_destroy( self.application ) };
        self.application = std::ptr::null_mut();
    }
}

unsafe impl Send for Application {}
unsafe impl Sync for Application {}