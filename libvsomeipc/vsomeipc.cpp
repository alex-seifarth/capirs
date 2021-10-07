/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/
#include <vsomeipc.h>
#include <vsomeip/vsomeip.hpp>
#include <cassert>

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
