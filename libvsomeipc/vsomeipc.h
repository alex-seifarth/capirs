/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/
#pragma once

#include "vsomeipc_export.h"

#ifdef __cplusplus
#include <memory>
#include <vsomeip/vsomeip.hpp>

typedef vsomeip::client_t client_id_t;

typedef std::shared_ptr<vsomeip::runtime>* runtime_t;
typedef std::shared_ptr<vsomeip::application>* application_t;

extern "C" {

#else

#include <stdint.h>

typedef void* application_t;
typedef void* runtime_t;

typedef uint16_t client_id_t;

#endif

VSOMEIPC_EXPORT int runtime_get(runtime_t* rt);
VSOMEIPC_EXPORT void runtime_release(runtime_t rt);
VSOMEIPC_EXPORT int runtime_create_app(runtime_t rt, application_t* app, const char* app_name);

//VSOMEIPC_EXPORT client_id_t application_client_id(application_t app);
VSOMEIPC_EXPORT int application_init(application_t app);
VSOMEIPC_EXPORT void application_destroy(application_t app);
VSOMEIPC_EXPORT void application_start(application_t app);
VSOMEIPC_EXPORT void application_stop(application_t app);
VSOMEIPC_EXPORT application_t application_clone(application_t old_app);


#ifdef __cplusplus
}
#endif
