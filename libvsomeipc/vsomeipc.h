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

typedef vsomeip::service_t service_t;
typedef vsomeip::instance_t instance_t;
typedef vsomeip::major_version_t major_version_t;
typedef vsomeip::minor_version_t minor_version_t;

extern "C" {

#else

#include <stdint.h>

typedef void* application_t;
typedef void* runtime_t;

typedef uint16_t client_id_t;
typedef uint16_t service_t;
typedef uint16_t instance_t;
typedef uint8_t major_version_t;
typedef uint32_t minor_version_t;

#endif

VSOMEIPC_EXPORT int runtime_get(runtime_t* rt);
VSOMEIPC_EXPORT void runtime_release(runtime_t rt);
VSOMEIPC_EXPORT int runtime_create_app(runtime_t rt, application_t* app, const char* app_name);

//VSOMEIPC_EXPORT client_id_t application_client_id(application_t app);
VSOMEIPC_EXPORT int application_init(application_t app);
VSOMEIPC_EXPORT void application_destroy(application_t app);
VSOMEIPC_EXPORT void application_start(application_t app);
VSOMEIPC_EXPORT void application_stop(application_t app);
VSOMEIPC_EXPORT const char* application_get_name(application_t app); // caller owns returned pointer

typedef enum app_reg_state {
    ARS_REGISTERED,
    ARS_NOT_REGISTERED,
} app_reg_state;

typedef void (*app_state_callback)(app_reg_state, void* context);

VSOMEIPC_EXPORT void application_register_state_handler(application_t app, app_state_callback cbk, void* context);
VSOMEIPC_EXPORT void application_unregister_state_handler(application_t app);

VSOMEIPC_EXPORT void application_offer_service(application_t app, service_t _service, instance_t _instance,
        major_version_t _major, minor_version_t _minor);
VSOMEIPC_EXPORT void application_stop_offer_service(application_t app, service_t _service, instance_t _instance,
        major_version_t _major, minor_version_t _minor);

#ifdef __cplusplus
}
#endif
