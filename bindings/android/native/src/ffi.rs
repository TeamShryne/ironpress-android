use jni::JNIEnv;
use jni::objects::{JByteArray, JFloatArray, JString};
use ironpress_ffi as abi;

fn bytes(data: &[u8]) -> abi::IronpressBytes { abi::IronpressBytes { data: data.as_ptr(), len: data.len() } }

pub fn throw(env: &mut JNIEnv, message: String) {
    let _ = env.throw_new("java/lang/IllegalStateException", message);
}

unsafe fn status(status: i32, error: *mut abi::IronpressError) -> Result<(), String> {
    if status == abi::IRONPRESS_STATUS_OK { return Ok(()); }
    let message = if error.is_null() { format!("Ironpress failed ({status})") } else {
        // SAFETY: Error handle came from this ABI call and is uniquely owned here.
        let len = unsafe { abi::ironpress_error_message_len(error) };
        let ptr = unsafe { abi::ironpress_error_message_data(error) };
        let text = if ptr.is_null() || len == 0 { format!("Ironpress failed ({status})") } else { String::from_utf8_lossy(unsafe { std::slice::from_raw_parts(ptr, len) }).into_owned() };
        let mut owned = error;
        unsafe { abi::ironpress_error_free(&mut owned); }
        text
    };
    Err(message)
}

pub fn status_result(status_code: i32, error: &mut *mut abi::IronpressError) -> Result<(), String> {
    // SAFETY: The caller owns the error output from the immediately preceding C ABI call.
    unsafe { status(status_code, *error) }
}

pub fn set_option(env: &mut JNIEnv, handle: i64, option: JString, text: JString, byte_array: JByteArray, float_array: JFloatArray, integer: i32, boolean: u8) -> Result<(), String> {
    let option: String = env.get_string(&option).map_err(|e| e.to_string())?.into();
    let text: String = env.get_string(&text).map_err(|e| e.to_string())?.into();
    let byte_data = env.convert_byte_array(byte_array).map_err(|e| e.to_string())?;
    let floats = env.convert_float_array(float_array).map_err(|e| e.to_string())?;
    let converter = handle as usize as *mut abi::IronpressConverter;
    let mut error = std::ptr::null_mut();
    // SAFETY: `handle` is owned by the Java converter and temporary arrays live for the call.
    let result = unsafe { match option.as_str() {
        "pageSize" => abi::ironpress_converter_set_page_size(converter, integer as u32, &mut error),
        "customPageSize" => abi::ironpress_converter_set_page_size_custom(converter, floats.first().copied().unwrap_or(f32::NAN), floats.get(1).copied().unwrap_or(f32::NAN), &mut error),
        "margin" => abi::ironpress_converter_set_margin(converter, floats.first().copied().unwrap_or(f32::NAN), &mut error),
        "margins" => abi::ironpress_converter_set_margins(converter, floats.first().copied().unwrap_or(f32::NAN), floats.get(1).copied().unwrap_or(f32::NAN), floats.get(2).copied().unwrap_or(f32::NAN), floats.get(3).copied().unwrap_or(f32::NAN), &mut error),
        "header" => abi::ironpress_converter_set_header(converter, bytes(text.as_bytes()), &mut error),
        "headerHtml" => abi::ironpress_converter_set_header_html(converter, bytes(text.as_bytes()), &mut error),
        "footer" => abi::ironpress_converter_set_footer(converter, bytes(text.as_bytes()), &mut error),
        "footerHtml" => abi::ironpress_converter_set_footer_html(converter, bytes(text.as_bytes()), &mut error),
        "compress" => abi::ironpress_converter_set_compress(converter, boolean, &mut error),
        "jpegQuality" => abi::ironpress_converter_set_jpeg_quality(converter, integer as u8, &mut error),
        "autoResizeImages" => abi::ironpress_converter_set_auto_resize_images(converter, boolean, &mut error),
        "imageDpi" => abi::ironpress_converter_set_image_dpi(converter, floats.first().copied().unwrap_or(f32::NAN), &mut error),
        "filterDpi" => abi::ironpress_converter_set_filter_dpi(converter, floats.first().copied().unwrap_or(f32::NAN), &mut error),
        "maskDpi" => abi::ironpress_converter_set_mask_dpi(converter, floats.first().copied().unwrap_or(f32::NAN), &mut error),
        "backgroundRasterDpi" => abi::ironpress_converter_set_background_raster_dpi(converter, floats.first().copied().unwrap_or(f32::NAN), &mut error),
        "occlusionCull" => abi::ironpress_converter_set_occlusion_cull(converter, boolean, &mut error),
        "sanitize" => abi::ironpress_converter_set_sanitize(converter, boolean, &mut error),
        "font" => abi::ironpress_converter_add_font(converter, bytes(text.as_bytes()), bytes(&byte_data), &mut error),
        "fontPack" => abi::ironpress_converter_add_font_pack(converter, integer as u32, bytes(&byte_data), &mut error),
        _ => return Err(format!("Unknown Ironpress option: {option}")),
    }};
    // SAFETY: `error` is the sole owning output slot returned by this call.
    unsafe { status(result, error) }
}

pub fn convert(env: &mut JNIEnv, handle: i64, input: JByteArray, markdown: bool) -> Result<Vec<u8>, String> {
    let input = env.convert_byte_array(input).map_err(|e| e.to_string())?;
    let converter = handle as usize as *const abi::IronpressConverter;
    let mut pdf = std::ptr::null_mut();
    let mut error = std::ptr::null_mut();
    // SAFETY: Input buffer and converter remain valid for the duration of the call.
    let status_code = unsafe { if markdown { abi::ironpress_converter_convert_markdown(converter, bytes(&input), &mut pdf, &mut error) } else { abi::ironpress_converter_convert_html(converter, bytes(&input), &mut pdf, &mut error) } };
    // SAFETY: Error output belongs to this invocation.
    unsafe { status(status_code, error)?; }
    let len = unsafe { abi::ironpress_buffer_len(pdf) };
    let ptr = unsafe { abi::ironpress_buffer_data(pdf) };
    let result = if len == 0 { Err("Ironpress returned an empty PDF".into()) } else { Ok(unsafe { std::slice::from_raw_parts(ptr, len) }.to_vec()) };
    let mut owned = pdf;
    unsafe { abi::ironpress_buffer_free(&mut owned); }
    result
}
