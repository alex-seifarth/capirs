/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/
#include <vsomeipc.h>
#include <vsomeip/vsomeip.hpp>
#include <cassert>

// ================================================================================================
// runtime
// ================================================================================================

int runtime_get(runtime_t* rt)
{
    assert(rt);
    auto p = new std::shared_ptr<vsomeip_v3::runtime>(vsomeip::runtime::get());
    if (!*p) {
        delete p;
        return -1;
    }
    *rt = p;
    return 0;
}

void runtime_release(runtime_t rt)
{
    assert(rt);
    delete rt;
}

int runtime_create_app(runtime_t rt, application_t* app, const char* app_name)
{
    assert(app);
    assert(rt && *rt);
    auto p = new std::shared_ptr<vsomeip::application>
            ((*rt)->create_application(std::string{app_name}));
    if (!*p) {
        delete p;
        return -1;
    }
    *app = p;
    return 0;
}


message_t runtime_create_request(runtime_t runtime, service_t service, instance_t instance,
                                 method_t method, major_version_t mjr_vers, int fire_and_forget, int is_reliable)
{
    assert(runtime && *runtime);
    auto msg = (*runtime)->create_request(is_reliable > 0);
    assert(msg);
    msg->set_service(service);
    msg->set_instance(instance);
    msg->set_method(method);
    msg->set_interface_version(mjr_vers);
    msg->set_message_type(fire_and_forget > 0 ? message_type_t::MT_REQUEST_NO_RETURN : message_type_t::MT_REQUEST);
    return new std::shared_ptr<vsomeip::message>(msg);
}

message_t runtime_create_response(runtime_t runtime, service_t service, instance_t instance,
                                  client_t client, session_t session, method_t method, major_version_t mjr_vers,
                                  int is_reliable)
{
    assert(runtime && *runtime);
    auto resp = (*runtime)->create_message(is_reliable != 0);
    assert(resp);
    resp->set_service(service);
    resp->set_instance(instance);
    resp->set_method(method);
    resp->set_client(client);
    resp->set_session(session);
    resp->set_interface_version(mjr_vers);
    resp->set_message_type(vsomeip::message_type_e::MT_RESPONSE);
    resp->set_return_code(vsomeip::return_code_e::E_OK);
    return new std::shared_ptr<vsomeip::message>(resp);
}

message_t runtime_create_error(runtime_t runtime, service_t service, instance_t instance,
                               client_t client, session_t session, method_t method, major_version_t mjr_vers,
                               int is_reliable, return_code_t return_code)
{
    assert(runtime && *runtime);
    auto resp = (*runtime)->create_message(is_reliable != 0);
    assert(resp);
    resp->set_service(service);
    resp->set_instance(instance);
    resp->set_method(method);
    resp->set_client(client);
    resp->set_session(session);
    resp->set_interface_version(mjr_vers);
    resp->set_message_type(vsomeip::message_type_e::MT_ERROR);
    resp->set_return_code(return_code);
    return new std::shared_ptr<vsomeip::message>(resp);
}

payload_t runtime_create_payload(runtime_t runtime, uint8_t const* pdata, uint32_t data_len)
{
    assert(runtime && *runtime);
    auto payload = (*runtime)->create_payload((vsomeip::byte_t const*) pdata, data_len);
    return new std::shared_ptr<vsomeip::payload>(payload);
}

// ================================================================================================
// application
// ================================================================================================
//client_id_t application_client_id(application_t app)
//{
//    assert(app && app->application_);
//    return app->application_->get_client();
//}

int application_init(application_t app)
{
    assert(app && *app);
    return (*app)->init() ? 0 : -1;
}

void application_destroy(application_t app)
{
    assert(app);
    delete app;
}

void application_start(application_t app)
{
    assert(app && *app);
    (*app)->start();
}

void application_stop(application_t app)
{
    assert(app && *app);
    (*app)->stop();
}

const char* application_get_name(application_t app)
{
    assert(app && *app);
    auto name = (*app)->get_name();
    char *c_name = new char[name.size() + 1];
    std::copy(name.begin(), name.end(), c_name);
    c_name[name.size()] = '\0';
    return c_name;
}

void application_register_state_handler(application_t app, app_state_callback cbk, void* context)
{
    assert(app && *app);
    (*app)->register_state_handler([cbk, context](vsomeip::state_type_e state) {
        cbk(state == vsomeip::state_type_e::ST_REGISTERED ? ARS_REGISTERED : ARS_NOT_REGISTERED, context);
    });
}

void application_unregister_state_handler(application_t app)
{
    assert(app && *app);
    (*app)->unregister_state_handler();
}

void application_offer_service(application_t app, service_t _service, instance_t _instance,
                               major_version_t _major, minor_version_t _minor)
{
    assert(app && *app);
    (*app)->offer_service(_service, _instance, _major, _minor);
}

void application_stop_offer_service(application_t app, service_t _service, instance_t _instance,
                                    major_version_t _major, minor_version_t _minor)
{
    assert(app && *app);
    (*app)->stop_offer_service(_service, _instance, _major, _minor);
}

void application_clear_all_handlers(application_t app)
{
    assert(app && *app);
    (*app)->clear_all_handler();
}

void application_register_message_handler(application_t app, service_t _service, instance_t _instance,
                                          message_callback cbk, void* context)
{
    assert(app && *app);
    (*app)->register_message_handler(_service, _instance, vsomeip::ANY_METHOD,
         [cbk, context](std::shared_ptr<vsomeip::message> const& msg) {
        auto message = new std::shared_ptr<vsomeip::message>{msg};
        cbk(message, context);
    });
}

void application_unregister_message_handler(application_t app, service_t _service, instance_t _instance)
{
    assert(app && *app);
    (*app)->unregister_message_handler(_service, _instance, vsomeip::ANY_METHOD);
}

void application_request_service(application_t app, service_t service, instance_t instance,
                                 major_version_t mjr_version, minor_version_t mnr_version)
{
    assert(app && *app);
    (*app)->request_service(service, instance, mjr_version, mnr_version);
}

void application_release_service(application_t app, service_t service, instance_t instance)
{
    assert(app && *app);
    (*app)->release_service(service, instance);
}

void application_register_availability_callback(application_t app, service_t service, instance_t instance,
                                                                availability_callback cbk, void* context)
{
    assert(app && *app);
    (*app)->register_availability_handler(service, instance,
  [cbk, context](service_t s, instance_t i, bool avail) {
        cbk(s, i, avail ? 1 : 0, context);
    });
}

void application_unregister_availability_callback(application_t app, service_t service, instance_t instance)
{
    assert(app && *app);
    (*app)->unregister_availability_handler(service, instance);
}

int application_is_available(application_t app, service_t service, instance_t instance) {
    assert(app && *app);
    return (*app)->is_available(service, instance);
}

void application_send(application_t app, message_t msg, payload_t payload)
{
    assert(app && *app);
    assert(msg && *msg);
    if (payload && *payload) {
        (*msg)->set_payload(*payload);
    }
    (*app)->send(std::shared_ptr<vsomeip::message>(*msg));
}

// ================================================================================================
// message
// ================================================================================================

service_t message_get_service(message_t msg) {
    assert(msg && *msg);
    return (*msg)->get_service();
}

instance_t message_get_instance(message_t msg) {
    assert(msg && *msg);
    return (*msg)->get_instance();
}

method_t message_get_method(message_t msg) {
    assert(msg && *msg);
    return (*msg)->get_method();
}

client_t message_get_client(message_t msg) {
    assert(msg && *msg);
    return (*msg)->get_client();
}

session_t message_get_session(message_t msg) {
    assert(msg && *msg);
    return (*msg)->get_session();
}

message_type_t message_get_type(message_t msg) {
    assert(msg && *msg);
    return (*msg)->get_message_type();
}

major_version_t message_get_interface_version(message_t msg) {
    assert(msg && *msg);
    return (*msg)->get_interface_version();
}

protocol_version_t message_get_protocol_version(message_t msg) {
    assert(msg && *msg);
    return (*msg)->get_protocol_version();
}

return_code_t message_get_return_code(message_t msg) {
    assert(msg && *msg);
    return (*msg)->get_return_code();
}

int message_is_reliable(message_t msg) {
    assert(msg && *msg);
    return (*msg)->is_reliable() ? 1 : 0;
}

int message_is_initial(message_t msg) {
    assert(msg && *msg);
    return (*msg)->is_initial() ? 1 : 0;
}

unsigned char* message_get_data(message_t msg, uint32_t* length)
{
    assert(msg && *msg);
    assert(length);
    auto payload = (*msg)->get_payload();
    if (payload) {
        *length = payload->get_length();
        return payload->get_data();
    }
    else {
        *length = 0;
        return nullptr;
    }
}

void message_destroy(message_t msg)
{
    assert(msg && *msg);
    delete msg;
}

void payload_destroy(payload_t payload)
{
    delete payload;
}

unsigned char* payload_get_data(payload_t payload, uint32_t* length)
{
    assert(payload && *payload);
    assert(length);

    *length = (*payload)->get_length();
    return (*payload)->get_data();
}
