#include <stdbool.h>
#include <dlfcn.h>
#include <objc/runtime.h>
#include <objc/message.h>
#include <atomic>
#include <stdint.h>
#include <stdlib.h>
#include <signal.h>
#include <setjmp.h>
#include <exception>

static std::atomic<bool> g_suppress_modals{false};
static bool g_swizzled{false};
static intptr_t (*g_orig_alert_run_modal)(id, SEL) = nullptr;
static intptr_t (*g_orig_app_run_modal_for_window)(id, SEL, id) = nullptr;

static thread_local sigjmp_buf g_guard_jmp;
static thread_local bool g_guard_active = false;
static bool g_handlers_installed = false;

static void plugin_terminate_handler() {
    if (g_guard_active) {
        siglongjmp(g_guard_jmp, 1);
    }
    abort();
}

static void plugin_sigabrt_handler(int /*sig*/) {
    if (g_guard_active) {
        siglongjmp(g_guard_jmp, 1);
    }
    signal(SIGABRT, SIG_DFL);
    raise(SIGABRT);
}

static void ensure_crash_handlers() {
    if (g_handlers_installed) return;
    g_handlers_installed = true;

    std::set_terminate(plugin_terminate_handler);

    struct sigaction sa;
    sa.sa_handler = plugin_sigabrt_handler;
    sigemptyset(&sa.sa_mask);
    sa.sa_flags = 0;
    sigaction(SIGABRT, &sa, NULL);
}

static intptr_t hook_alert_run_modal(id self, SEL _cmd) {
    if (g_suppress_modals.load(std::memory_order_relaxed)) {
        return 1000; // NSAlertFirstButtonReturn
    }
    if (g_orig_alert_run_modal) {
        return g_orig_alert_run_modal(self, _cmd);
    }
    return 1000;
}

static intptr_t hook_app_run_modal_for_window(id self, SEL _cmd, id window) {
    if (g_suppress_modals.load(std::memory_order_relaxed)) {
        return -1000; // NSModalResponseStop
    }
    if (g_orig_app_run_modal_for_window) {
        return g_orig_app_run_modal_for_window(self, _cmd, window);
    }
    return -1000;
}

static void ensure_modal_hooks() {
    if (g_swizzled) return;
    g_swizzled = true;

    Class alert_cls = objc_getClass("NSAlert");
    if (alert_cls) {
        SEL sel = sel_registerName("runModal");
        Method m = class_getInstanceMethod(alert_cls, sel);
        if (m) {
            g_orig_alert_run_modal = (intptr_t (*)(id, SEL))method_getImplementation(m);
            method_setImplementation(m, (IMP)hook_alert_run_modal);
        }
    }

    Class app_cls = objc_getClass("NSApplication");
    if (app_cls) {
        SEL sel = sel_registerName("runModalForWindow:");
        Method m = class_getInstanceMethod(app_cls, sel);
        if (m) {
            g_orig_app_run_modal_for_window = (intptr_t (*)(id, SEL, id))method_getImplementation(m);
            method_setImplementation(m, (IMP)hook_app_run_modal_for_window);
        }
    }
}

struct ScopedPluginGuard {
    ScopedPluginGuard() {
        ensure_crash_handlers();
        ensure_modal_hooks();
        g_suppress_modals.store(true, std::memory_order_relaxed);
        g_guard_active = true;
    }
    ~ScopedPluginGuard() {
        g_guard_active = false;
        g_suppress_modals.store(false, std::memory_order_relaxed);
    }
};

extern "C" {

void* vv_plugin_call_dlopen(const char* path, int flags) {
    ScopedPluginGuard guard;
    if (sigsetjmp(g_guard_jmp, 1) != 0) {
        return nullptr;
    }
    @try {
        try {
            return dlopen(path, flags);
        } catch (...) {
            return nullptr;
        }
    } @catch (id ex) {
        return nullptr;
    }
}

bool vv_plugin_call_bundle_entry(bool (*entry)(const void*), const void* bundle) {
    ScopedPluginGuard guard;
    if (sigsetjmp(g_guard_jmp, 1) != 0) {
        return false;
    }
    @try {
        try {
            return entry(bundle);
        } catch (...) {
            return false;
        }
    } @catch (id ex) {
        return false;
    }
}

bool vv_plugin_call_module_entry(bool (*entry)(void*), void* handle) {
    ScopedPluginGuard guard;
    if (sigsetjmp(g_guard_jmp, 1) != 0) {
        return false;
    }
    @try {
        try {
            return entry(handle);
        } catch (...) {
            return false;
        }
    } @catch (id ex) {
        return false;
    }
}

void* vv_plugin_call_get_factory(void* (*get_factory)()) {
    ScopedPluginGuard guard;
    if (sigsetjmp(g_guard_jmp, 1) != 0) {
        return nullptr;
    }
    @try {
        try {
            return get_factory();
        } catch (...) {
            return nullptr;
        }
    } @catch (id ex) {
        return nullptr;
    }
}

void vv_plugin_call_bundle_exit(bool (*exit_fn)()) {
    ScopedPluginGuard guard;
    if (sigsetjmp(g_guard_jmp, 1) != 0) {
        return;
    }
    @try {
        try {
            exit_fn();
        } catch (...) {
        }
    } @catch (id ex) {
    }
}

}
