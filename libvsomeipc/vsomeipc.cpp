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
