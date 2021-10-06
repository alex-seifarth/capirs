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

pub struct Application {
    application: application_t
}

impl Drop for Application {
    fn drop(&mut self) {
        unsafe{ application_destroy( self.application )};
        self.application = std::ptr::null_mut();
    }
}
