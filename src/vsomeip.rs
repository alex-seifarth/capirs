/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused)]

include!(concat!(env!("OUT_DIR"), "/vsomeipc.rs"));

use std::ffi::CString;

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

    pub fn create_application(&self, name: &str) -> Result<Application, ()> {
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
        Ok( Application{ application } )
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        unsafe{ runtime_release( self.runtime )};
        self.runtime = std::ptr::null_mut();
    }
}

unsafe impl Send for Runtime {}

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
    application: application_t
}

impl Application {

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
}

impl Drop for Application {
    fn drop(&mut self) {
        unsafe{ application_destroy( self.application ) };
        self.application = std::ptr::null_mut();
    }
}

impl Clone for Application {
    /// Application objects can be cloned.
    /// This simply creates a new shared pointer (C++) on the heap that references the same
    /// vsomeip application object as 'self' does.
    fn clone(&self) -> Self {
        Application{ application: unsafe{ application_clone( self.application ) } }
    }
}

unsafe impl Send for Application {}
unsafe impl Sync for Application {}