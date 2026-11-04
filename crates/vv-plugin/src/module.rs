use std::collections::HashMap;
use std::ffi::{c_char, c_int};
#[cfg(any(target_os = "macos", target_os = "linux"))]
use std::ffi::c_void;
use std::mem::ManuallyDrop;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use vst3::Steinberg::{
    IPluginFactory, IPluginFactory2, IPluginFactory2Trait, IPluginFactoryTrait, PClassInfo,
    PClassInfo2, PFactoryInfo, TUID, kResultOk,
};
use vst3::{ComPtr, Interface};

use crate::scan::{AUDIO_CLASS, PluginInfo, is_effect};

pub(crate) struct Module {
    factory: ComPtr<IPluginFactory>,
    // Last: fields drop in declaration order, and `factory`'s release runs plugin code.
    // Never unloaded: unloading plugin code is unsafe in practice.
    _library: ManuallyDrop<libloading::Library>,
}

// SAFETY: VST3 factories are required to be callable from any thread.
unsafe impl Send for Module {}
unsafe impl Sync for Module {}

type Slot<T> = Arc<Mutex<Option<Result<T, String>>>>;
type Registry<T> = Mutex<Option<HashMap<PathBuf, Slot<T>>>>;

static MODULES: Registry<Arc<Module>> = Mutex::new(None);
type Stamp = Option<(u64, Option<std::time::Duration>)>;
static STAMPS: Mutex<Option<HashMap<PathBuf, Stamp>>> = Mutex::new(None);

fn stamp_of(file: &Path) -> Stamp {
    let meta = std::fs::metadata(file).ok()?;
    let modified = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok());
    Some((meta.len(), modified))
}

fn stamps() -> std::sync::MutexGuard<'static, Option<HashMap<PathBuf, Stamp>>> {
    STAMPS.lock().unwrap_or_else(|e| e.into_inner())
}

/// Of the module binary as `load` opened it, so it describes the code this
/// process runs and every pool agrees; modules are never unloaded. A module
/// not loaded yet is stat'ed once (the bundle itself when that is a file).
pub(crate) fn build(bundle: &Path, version: &str) -> u64 {
    let known = stamps().as_ref().and_then(|m| m.get(bundle).copied());
    let stamp = known.unwrap_or_else(|| {
        let file = bundle_binary(bundle)
            .filter(|binary| binary.is_file())
            .unwrap_or_else(|| bundle.to_path_buf());
        let stamp = stamp_of(&file);
        *stamps()
            .get_or_insert_with(HashMap::new)
            .entry(bundle.to_path_buf())
            .or_insert(stamp)
    });
    vv_core::stable_hash((version, stamp))
}

pub(crate) struct LoadError {
    msg: String,
    // The binary was dlopened: it can never be unloaded, so a retry would re-run its initialisers.
    after_dlopen: bool,
}

impl LoadError {
    fn early(msg: impl Into<String>) -> Self {
        Self {
            msg: msg.into(),
            after_dlopen: false,
        }
    }

    fn loaded(msg: impl Into<String>) -> Self {
        Self {
            msg: msg.into(),
            after_dlopen: true,
        }
    }
}

impl From<String> for LoadError {
    fn from(msg: String) -> Self {
        Self::early(msg)
    }
}

#[cfg(target_os = "macos")]
mod macos_dispatch {
    use std::ffi::c_void;

    unsafe extern "C" {
        static _dispatch_main_q: c_void;
        fn dispatch_sync_f(
            queue: *const c_void,
            context: *mut c_void,
            work: extern "C" fn(*mut c_void),
        );
        static NSApp: *mut objc2::runtime::AnyObject;
    }

    fn is_main_thread() -> bool {
        objc2::MainThreadMarker::new().is_some()
    }

    fn is_app_running() -> bool {
        unsafe {
            if NSApp.is_null() {
                return false;
            }
            let running: bool = objc2::msg_send![NSApp, isRunning];
            running
        }
    }

    pub(crate) fn on_main_thread_if_running<F, R>(f: F) -> R
    where
        F: FnOnce() -> R + Send,
        R: Send,
    {
        if is_main_thread() || !is_app_running() {
            return f();
        }
        struct Context<F, R> {
            f: Option<F>,
            res: Option<R>,
        }
        extern "C" fn trampoline<F: FnOnce() -> R + Send, R: Send>(ctx_ptr: *mut c_void) {
            let ctx = unsafe { &mut *ctx_ptr.cast::<Context<F, R>>() };
            if let Some(f) = ctx.f.take() {
                ctx.res = Some(f());
            }
        }
        let mut ctx = Context {
            f: Some(f),
            res: None,
        };
        unsafe {
            let queue = std::ptr::addr_of!(_dispatch_main_q);
            dispatch_sync_f(
                queue,
                std::ptr::from_mut(&mut ctx).cast(),
                trampoline::<F, R>,
            );
        }
        ctx.res.expect("trampoline must run")
    }
}

pub(crate) fn module(path: &Path) -> Result<Arc<Module>, String> {
    #[cfg(target_os = "macos")]
    {
        cached(&MODULES, path, || {
            macos_dispatch::on_main_thread_if_running(|| load(path).map(Arc::new))
        })
    }
    #[cfg(not(target_os = "macos"))]
    {
        cached(&MODULES, path, || load(path).map(Arc::new))
    }
}

// The map lock is never held across `load`: a plugin initializer can be slow or call back
// into the main thread. Only same-path callers wait on each other, through the slot.
fn cached<T: Clone>(
    registry: &Registry<T>,
    path: &Path,
    load: impl FnOnce() -> Result<T, LoadError>,
) -> Result<T, String> {
    let slot = {
        let mut guard = registry.lock().unwrap_or_else(|e| e.into_inner());
        guard
            .get_or_insert_with(HashMap::new)
            .entry(path.to_path_buf())
            .or_default()
            .clone()
    };
    let mut loaded = slot.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(done) = loaded.as_ref() {
        return done.clone();
    }
    match load() {
        Ok(m) => {
            *loaded = Some(Ok(m.clone()));
            Ok(m)
        }
        Err(e) => {
            if e.after_dlopen {
                *loaded = Some(Err(e.msg.clone()));
            }
            Err(e.msg)
        }
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn stem(bundle: &Path) -> Result<&std::ffi::OsStr, String> {
    bundle
        .file_stem()
        .ok_or_else(|| format!("bad bundle path {}", bundle.display()))
}

#[cfg(target_os = "macos")]
type BundleRef = objc2_core_foundation::CFRetained<objc2_core_foundation::CFBundle>;

#[cfg(target_os = "macos")]
fn bundle_ref(bundle: &Path) -> Result<BundleRef, String> {
    use objc2_core_foundation::{CFBundle, CFURL};
    let url = CFURL::from_directory_path(bundle).ok_or("bad bundle url")?;
    CFBundle::new(None, Some(&url)).ok_or_else(|| "CFBundleCreate failed".into())
}

// CFBundleExecutable names the binary; bundles without an Info.plist use the folder name.
#[cfg(target_os = "macos")]
fn binary_path(bundle: &Path, cf_bundle: &BundleRef) -> Result<PathBuf, String> {
    let declared = cf_bundle
        .executable_url()
        .and_then(|url| url.to_file_path());
    match declared {
        Some(path) if path.is_file() => Ok(path),
        _ => Ok(bundle.join("Contents/MacOS").join(stem(bundle)?)),
    }
}

#[cfg(target_os = "linux")]
fn binary_path(bundle: &Path) -> Result<PathBuf, String> {
    let arch = sdk_arch_dir(std::env::consts::ARCH, cfg!(target_endian = "big"));
    Ok(linux_binary(bundle, stem(bundle)?, &arch))
}

// VST3 bundle folders use `uname -m` names, which differ from Rust's ARCH.
#[cfg(any(not(target_os = "macos"), test))]
fn sdk_arch_dir(arch: &str, big_endian: bool) -> String {
    match (arch, big_endian) {
        ("x86", _) => "i386-linux".into(),
        ("arm", _) => "armv7l-linux".into(),
        ("powerpc64", false) => "ppc64le-linux".into(),
        _ => format!("{arch}-linux"),
    }
}

// Falls back to any `*-linux` folder holding the binary (armv8l, i686, ...).
#[cfg(any(not(target_os = "macos"), test))]
fn linux_binary(bundle: &Path, stem: &std::ffi::OsStr, arch_dir: &str) -> PathBuf {
    let contents = bundle.join("Contents");
    let file = so_file_name(stem);
    let mapped = contents.join(arch_dir).join(&file);
    if mapped.is_file() {
        return mapped;
    }
    std::fs::read_dir(&contents)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().ends_with("-linux"))
        .map(|e| e.path().join(&file))
        .find(|p| p.is_file())
        .unwrap_or(mapped)
}

#[cfg(any(not(target_os = "macos"), test))]
fn so_file_name(stem: &std::ffi::OsStr) -> std::ffi::OsString {
    let mut name = stem.to_os_string();
    name.push(".so");
    name
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn binary_path(_bundle: &Path) -> Result<PathBuf, String> {
    Err("VST3 plugins are not supported on this platform".into())
}

/// The file `load` opens for this bundle, for callers that watch it change.
pub fn bundle_binary(bundle: &Path) -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    return binary_path(bundle, &bundle_ref(bundle).ok()?).ok();
    #[cfg(not(target_os = "macos"))]
    binary_path(bundle).ok()
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn load(_bundle: &Path) -> Result<Module, LoadError> {
    Err(LoadError::early(
        "VST3 plugins are not supported on this platform",
    ))
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
unsafe extern "C" {
    #[cfg(target_os = "macos")]
    fn vv_plugin_call_dlopen(path: *const c_char, flags: c_int) -> *mut c_void;

    #[cfg(target_os = "macos")]
    fn vv_plugin_call_bundle_entry(
        entry: unsafe extern "C" fn(*const objc2_core_foundation::CFBundle) -> bool,
        bundle: *const objc2_core_foundation::CFBundle,
    ) -> bool;

    #[cfg(target_os = "linux")]
    fn vv_plugin_call_module_entry(
        entry: unsafe extern "C" fn(*mut c_void) -> bool,
        handle: *mut c_void,
    ) -> bool;

    fn vv_plugin_call_get_factory(
        get_factory: unsafe extern "C" fn() -> *mut c_void,
    ) -> *mut c_void;

    fn vv_plugin_call_bundle_exit(exit_fn: unsafe extern "C" fn() -> bool);
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn load(bundle: &Path) -> Result<Module, LoadError> {
    #[cfg(target_os = "macos")]
    let cf_bundle = bundle_ref(bundle)?;
    #[cfg(target_os = "macos")]
    let bin = binary_path(bundle, &cf_bundle)?;
    #[cfg(not(target_os = "macos"))]
    let bin = binary_path(bundle)?;
    stamps()
        .get_or_insert_with(HashMap::new)
        .insert(bundle.to_path_buf(), stamp_of(&bin));
    let (library, handle) =
        open(&bin).map_err(|e| LoadError::early(format!("{}: {e}", bin.display())))?;
    #[cfg(target_os = "macos")]
    enter(&library, handle, cf_bundle).map_err(LoadError::loaded)?;
    #[cfg(not(target_os = "macos"))]
    enter(&library, handle).map_err(LoadError::loaded)?;
    // SAFETY: GetPluginFactory is the VST3 module entry point with this signature.
    let raw = unsafe {
        library
            .get::<unsafe extern "C" fn() -> *mut c_void>(b"GetPluginFactory\0")
            .map(|get| vv_plugin_call_get_factory(*get))
    };
    // SAFETY: a non-null result is an owned reference to the factory.
    let factory = raw.map_err(|e| e.to_string()).and_then(|raw| {
        unsafe { ComPtr::from_raw(raw.cast::<IPluginFactory>()) }
            .ok_or("GetPluginFactory returned null".to_string())
    });
    match factory {
        Ok(factory) => Ok(Module {
            factory,
            _library: library,
        }),
        Err(e) => {
            exit(&library);
            Err(LoadError::loaded(e))
        }
    }
}

// ModuleEntry wants the raw dlopen handle, so open through the unix layer.
#[cfg(target_os = "macos")]
fn open(bin: &Path) -> Result<(ManuallyDrop<libloading::Library>, *mut c_void), String> {
    use libloading::os::unix::Library;
    use std::os::unix::ffi::OsStrExt;
    let c_path = std::ffi::CString::new(bin.as_os_str().as_bytes())
        .map_err(|e| format!("{e}"))?;
    // RTLD_NOW = 2, RTLD_LOCAL = 4
    let handle = unsafe { vv_plugin_call_dlopen(c_path.as_ptr(), 2 | 4) };
    if handle.is_null() {
        let msg = unsafe {
            let err = libc::dlerror();
            if err.is_null() {
                "dlopen returned null".to_string()
            } else {
                std::ffi::CStr::from_ptr(err).to_string_lossy().into_owned()
            }
        };
        return Err(msg);
    }
    // SAFETY: handle was just produced by vv_plugin_call_dlopen.
    Ok((
        ManuallyDrop::new(unsafe { Library::from_raw(handle) }.into()),
        handle,
    ))
}

#[cfg(not(target_os = "macos"))]
fn open(bin: &Path) -> Result<(ManuallyDrop<libloading::Library>, *mut c_void), String> {
    use libloading::os::unix::{Library, RTLD_LOCAL, RTLD_NOW};
    // SAFETY: loading a plugin runs its initialisers; that is what hosting means.
    let lib = unsafe { Library::open(Some(bin), RTLD_NOW | RTLD_LOCAL) }
        .map_err(|e| e.to_string())?;
    let handle = lib.into_raw();
    // SAFETY: handle was just produced by into_raw.
    Ok((
        ManuallyDrop::new(unsafe { Library::from_raw(handle) }.into()),
        handle,
    ))
}

#[cfg(target_os = "macos")]
fn enter(
    library: &libloading::Library,
    _handle: *mut c_void,
    cf_bundle: BundleRef,
) -> Result<(), String> {
    // SAFETY: bundleEntry takes the bundle ref; the bundle stays alive with the module.
    let ok = unsafe {
        let Ok(entry) = library
            .get::<unsafe extern "C" fn(*const objc2_core_foundation::CFBundle) -> bool>(
                b"bundleEntry\0",
            )
        else {
            return Ok(());
        };
        vv_plugin_call_bundle_entry(*entry, &*cf_bundle)
    };
    std::mem::forget(cf_bundle);
    ok.then_some(()).ok_or_else(|| "bundleEntry failed".into())
}

#[cfg(target_os = "linux")]
fn enter(library: &libloading::Library, handle: *mut c_void) -> Result<(), String> {
    // SAFETY: ModuleEntry is the VST3 Linux entry point with this signature.
    let ok = unsafe {
        let Ok(entry) = library.get::<unsafe extern "C" fn(*mut c_void) -> bool>(b"ModuleEntry\0")
        else {
            return Ok(());
        };
        vv_plugin_call_module_entry(*entry, handle)
    };
    ok.then_some(()).ok_or_else(|| "ModuleEntry failed".into())
}

// Plugins fill these arrays; a missing NUL must not read past the end.
pub(crate) fn c_str(chars: &[c_char]) -> String {
    let len = chars.iter().position(|&c| c == 0).unwrap_or(chars.len());
    let bytes: Vec<u8> = chars[..len].iter().map(|&c| c as u8).collect();
    String::from_utf8_lossy(&bytes).into_owned()
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn exit(library: &libloading::Library) {
    let name: &[u8] = if cfg!(target_os = "macos") {
        b"bundleExit\0"
    } else {
        b"ModuleExit\0"
    };
    // SAFETY: both exit points are `bool ()` functions in the VST3 spec.
    unsafe {
        if let Ok(f) = library.get::<unsafe extern "C" fn() -> bool>(name) {
            vv_plugin_call_bundle_exit(*f);
        }
    }
}

fn hex(cid: &[c_char; 16]) -> String {
    cid.iter().map(|b| format!("{:02X}", *b as u8)).collect()
}

impl Module {
    pub(crate) fn create<I: Interface>(&self, cid: &TUID) -> Option<ComPtr<I>> {
        let mut obj = std::ptr::null_mut();
        // SAFETY: a live factory, 16-byte ids (a Guid has the TUID layout).
        let res = unsafe {
            self.factory
                .createInstance(cid.as_ptr(), I::IID.as_ptr().cast(), &mut obj)
        };
        // SAFETY: on success `obj` is an owned reference of the requested interface.
        (res == kResultOk)
            .then(|| unsafe { ComPtr::from_raw(obj.cast::<I>()) })
            .flatten()
    }

    pub(crate) fn effect_classes(&self, path: &Path) -> Vec<PluginInfo> {
        let factory = &self.factory;
        // SAFETY: plain out-parameter calls on a live factory with zeroed structs.
        unsafe {
            let mut finfo: PFactoryInfo = std::mem::zeroed();
            let factory_vendor = if factory.getFactoryInfo(&mut finfo) == kResultOk {
                c_str(&finfo.vendor)
            } else {
                String::new()
            };
            let factory2 = factory.cast::<IPluginFactory2>();
            let mut out = Vec::new();
            for i in 0..factory.countClasses() {
                if let Some(f2) = &factory2 {
                    let mut info: PClassInfo2 = std::mem::zeroed();
                    if f2.getClassInfo2(i, &mut info) != kResultOk {
                        continue;
                    }
                    let subs: Vec<String> = c_str(&info.subCategories)
                        .split('|')
                        .map(String::from)
                        .collect();
                    if !is_effect(&c_str(&info.category), &subs) {
                        continue;
                    }
                    let vendor = c_str(&info.vendor);
                    out.push(PluginInfo {
                        class_id: hex(&info.cid),
                        name: c_str(&info.name),
                        vendor: if vendor.is_empty() {
                            factory_vendor.clone()
                        } else {
                            vendor
                        },
                        version: c_str(&info.version),
                        path: path.to_path_buf(),
                    });
                } else {
                    let mut info: PClassInfo = std::mem::zeroed();
                    if factory.getClassInfo(i, &mut info) != kResultOk
                        || c_str(&info.category) != AUDIO_CLASS
                    {
                        continue;
                    }
                    out.push(PluginInfo {
                        class_id: hex(&info.cid),
                        name: c_str(&info.name),
                        vendor: factory_vendor.clone(),
                        version: String::new(),
                        path: path.to_path_buf(),
                    });
                }
            }
            out
        }
    }
}

#[cfg(test)]
#[path = "tests/module.rs"]
mod tests;
