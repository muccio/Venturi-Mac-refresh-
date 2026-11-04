#include <stdbool.h>

extern "C" {

bool vv_plugin_call_bundle_entry(bool (*entry)(const void*), const void* bundle) {
    try {
        return entry(bundle);
    } catch (...) {
        return false;
    }
}

bool vv_plugin_call_module_entry(bool (*entry)(void*), void* handle) {
    try {
        return entry(handle);
    } catch (...) {
        return false;
    }
}

void* vv_plugin_call_get_factory(void* (*get_factory)()) {
    try {
        return get_factory();
    } catch (...) {
        return nullptr;
    }
}

void vv_plugin_call_bundle_exit(bool (*exit_fn)()) {
    try {
        exit_fn();
    } catch (...) {
    }
}

}
