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
typedef std::shared_ptr<vsomeip::message> const* message_t;
typedef std::shared_ptr<vsomeip::payload>* payload_t;

typedef vsomeip::service_t service_t;
typedef vsomeip::instance_t instance_t;
typedef vsomeip::major_version_t major_version_t;
typedef vsomeip::minor_version_t minor_version_t;
typedef vsomeip::method_t method_t;
typedef vsomeip::event_t event_t;
typedef vsomeip::eventgroup_t event_group_t;

typedef vsomeip::client_t client_t;
typedef vsomeip::session_t session_t;
typedef vsomeip::message_type_e message_type_t;
typedef vsomeip::protocol_version_t protocol_version_t;
typedef vsomeip::return_code_e return_code_t;

extern "C" {

#else

#include <stdint.h>

typedef void* application_t;
typedef void* runtime_t;
typedef void* message_t;
typedef void* payload_t;

typedef uint16_t client_id_t;
typedef uint16_t service_t;
typedef uint16_t instance_t;
typedef uint8_t major_version_t;
typedef uint32_t minor_version_t;
typedef uint16_t method_t;
typedef uint16_t client_t;
typedef uint16_t session_t;
typedef uint8_t message_type_t;
typedef uint8_t protocol_version_t;
typedef uint8_t return_code_t;
typedef uint16_t event_t;
typedef uint16_t event_group_t;

#endif

typedef enum event_type_t {
    ET_EVENT = 0x00,
    ET_SELECTIVE_EVENT = 0x01,
    ET_FIELD = 0x02,
    ET_UNKNOWN = 0xFF
} event_type_t;

typedef enum reliability_t {
    RT_RELIABLE = 0x01,
    RT_UNRELIABLE = 0x02,
    RT_BOTH = 0x3, // RT_RELIABLE | RT_UNRELIABLE
    RT_UNKNOWN = 0xFF
} reliability_t;

VSOMEIPC_EXPORT int runtime_get(runtime_t* rt);
VSOMEIPC_EXPORT void runtime_release(runtime_t rt);
VSOMEIPC_EXPORT int runtime_create_app(runtime_t rt, application_t* app, const char* app_name);
VSOMEIPC_EXPORT message_t runtime_create_request(runtime_t runtime, service_t service, instance_t instance,
                                                 method_t method, major_version_t mjr_vers, int fire_and_forget, int is_reliable);
VSOMEIPC_EXPORT message_t runtime_create_response(runtime_t runtime, service_t service, instance_t instance,
                                                  client_t client, session_t session, method_t method, major_version_t mjr_vers,
                                                  int is_reliable);
VSOMEIPC_EXPORT message_t runtime_create_error(runtime_t runtime, service_t service, instance_t instance,
                                                  client_t client, session_t session, method_t method, major_version_t mjr_vers,
                                                  int is_reliable, return_code_t return_code);
VSOMEIPC_EXPORT payload_t runtime_create_payload(runtime_t runtime, uint8_t const* pdata, uint32_t data_len);

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
VSOMEIPC_EXPORT void application_clear_all_handlers(application_t app);

typedef void(*message_callback)(const message_t msg, void* context);
VSOMEIPC_EXPORT void application_register_message_handler(application_t app, service_t _service, instance_t _instance,
          message_callback cbk, void* context);
VSOMEIPC_EXPORT void application_unregister_message_handler(application_t app, service_t _service, instance_t _instance);

VSOMEIPC_EXPORT void application_request_service(application_t app, service_t service, instance_t instance,
                                                 major_version_t mjr_version, minor_version_t mnr_version);
VSOMEIPC_EXPORT void application_release_service(application_t app, service_t service, instance_t instance);

typedef void(*availability_callback)(service_t service, instance_t instance, int available, void* context);
VSOMEIPC_EXPORT void application_register_availability_callback(application_t app, service_t service, instance_t instance,
                                                                availability_callback cbk, void* context);
VSOMEIPC_EXPORT void application_unregister_availability_callback(application_t app, service_t service, instance_t instance);
VSOMEIPC_EXPORT int application_is_available(application_t app, service_t service, instance_t instance);
VSOMEIPC_EXPORT void application_send(application_t app, message_t msg, payload_t payload);
VSOMEIPC_EXPORT void application_notify(application_t app, service_t service, instance_t instance, event_t event, payload_t payload, int force);
VSOMEIPC_EXPORT void application_offer_event(application_t app, service_t service, instance_t instance, event_t event,
                                             event_type_t event_type, reliability_t reliability,
                                             event_group_t const* pevent_groups, int count_event_groups);
VSOMEIPC_EXPORT void application_stop_offer_event(application_t app, service_t service, instance_t instance, event_t event);

VSOMEIPC_EXPORT service_t message_get_service(message_t msg);
VSOMEIPC_EXPORT instance_t message_get_instance(message_t msg);
VSOMEIPC_EXPORT method_t message_get_method(message_t msg);
VSOMEIPC_EXPORT client_t message_get_client(message_t msg);
VSOMEIPC_EXPORT session_t message_get_session(message_t msg);
VSOMEIPC_EXPORT message_type_t message_get_type(message_t msg);
VSOMEIPC_EXPORT major_version_t message_get_interface_version(message_t msg);
VSOMEIPC_EXPORT protocol_version_t message_get_protocol_version(message_t msg);
VSOMEIPC_EXPORT return_code_t message_get_return_code(message_t msg);
VSOMEIPC_EXPORT int message_is_reliable(message_t msg);
VSOMEIPC_EXPORT int message_is_initial(message_t msg);
VSOMEIPC_EXPORT unsigned char* message_get_data(message_t msg, uint32_t* length);
VSOMEIPC_EXPORT void message_destroy(message_t msg);

VSOMEIPC_EXPORT void payload_destroy(payload_t payload);
VSOMEIPC_EXPORT unsigned char* payload_get_data(payload_t payload, uint32_t* length);

#ifdef __cplusplus
}
#endif
