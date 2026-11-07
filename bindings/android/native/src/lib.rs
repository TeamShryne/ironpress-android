mod ffi;

use std::ffi::CStr;

use jni::JNIEnv;
use jni::objects::{JByteArray, JClass, JFloatArray, JString};
use jni::sys::{jboolean, jbyteArray, jint, jlong, jstring};

use ironpress_ffi as abi;

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_teamshryne_ironpress_IronpressNative_nativeAbiVersion(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    abi::ironpress_abi_version() as jint
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_teamshryne_ironpress_IronpressNative_nativeVersion(
    mut env: JNIEnv,
    _class: JClass,
) -> jstring {
    // SAFETY: Ironpress returns a static, nul-terminated version string.
    let version = unsafe { CStr::from_ptr(abi::ironpress_version()) }.to_string_lossy();
    match env.new_string(version) {
        Ok(value) => value.into_raw(),
        Err(error) => {
            ffi::throw(&mut env, error.to_string());
            std::ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_teamshryne_ironpress_IronpressNative_nativeCreate(
    mut env: JNIEnv,
    _class: JClass,
) -> jlong {
    let mut converter = std::ptr::null_mut();
    let mut error = std::ptr::null_mut();
    // SAFETY: The output slots are initialized empty and live through the call.
    let status = unsafe { abi::ironpress_converter_new(&mut converter, &mut error) };
    if let Err(message) = ffi::status_result(status, &mut error) {
        ffi::throw(&mut env, message);
        0
    } else {
        converter as jlong
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_teamshryne_ironpress_IronpressNative_nativeFree(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    let mut converter = handle as usize as *mut abi::IronpressConverter;
    // SAFETY: The Kotlin owner calls this once for its unique converter handle.
    let status = unsafe { abi::ironpress_converter_free(&mut converter) };
    if status != abi::IRONPRESS_STATUS_OK {
        ffi::throw(&mut env, format!("Could not release Ironpress converter ({status})."));
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_teamshryne_ironpress_IronpressNative_nativeSetOption(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    option: JString,
    text: JString,
    bytes: JByteArray,
    floats: JFloatArray,
    integer: jint,
    boolean: jboolean,
) {
    let result = ffi::set_option(&mut env, handle, option, text, bytes, floats, integer, boolean);
    if let Err(message) = result {
        ffi::throw(&mut env, message);
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_teamshryne_ironpress_IronpressNative_nativeConvert(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    input: JByteArray,
    markdown: jboolean,
) -> jbyteArray {
    match ffi::convert(&mut env, handle, input, markdown != 0) {
        Ok(pdf) => match env.byte_array_from_slice(&pdf) {
            Ok(value) => value.into_raw(),
            Err(error) => {
                ffi::throw(&mut env, error.to_string());
                std::ptr::null_mut()
            }
        },
        Err(message) => {
            ffi::throw(&mut env, message);
            std::ptr::null_mut()
        }
    }
}
