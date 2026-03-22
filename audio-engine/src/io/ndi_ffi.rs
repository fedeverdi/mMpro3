// Minimal FFI bindings for NDI Audio Sender
// Loads NDI library dynamically at runtime - compiles without NDI installed

use libloading::{Library, Symbol};
use std::os::raw::{c_char, c_int, c_float, c_void};
use std::sync::OnceLock;

static NDI_LIB: OnceLock<Option<Library>> = OnceLock::new();

// NDI structures (matching official NDI SDK)
#[repr(C)]
pub struct NDIlib_send_create_t {
    pub p_ndi_name: *const c_char,
    pub p_groups: *const c_char,
    pub clock_video: bool,
    pub clock_audio: bool,
}

#[repr(C)]
pub struct NDIlib_audio_frame_v2_t {
    pub sample_rate: c_int,           // Sample rate (48000)
    pub no_channels: c_int,           // Number of audio channels (2 for stereo)
    pub no_samples: c_int,            // Number of samples per channel
    pub timecode: i64,                // Timecode (0 for auto)
    pub p_data: *const c_float,       // Audio data pointer (interleaved float32)
    pub channel_stride_in_bytes: c_int, // 0 = interleaved
    pub p_metadata: *const c_char,    // Optional metadata
    pub timestamp: i64,               // Timestamp
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum NDIlib_frame_format_type_e {
    Progressive = 1,
    Interleaved = 0,
    Field0 = 2,
    Field1 = 3,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum NDIlib_FourCC_video_type_e {
    UYVY = 0x59565955, // YUV 4:2:2
    UYVA = 0x41565955,
    P216 = 0x36313250,
    PA16 = 0x36314150,
    YV12 = 0x32315659,
    I420 = 0x30323449,
    NV12 = 0x3231564e,
    BGRA = 0x41524742, // BGRA 8-bit
    BGRX = 0x58524742,
    RGBA = 0x41424752,
    RGBX = 0x58424752,
}

#[repr(C)]
pub struct NDIlib_video_frame_v2_t {
    pub xres: c_int,                  // Horizontal resolution
    pub yres: c_int,                  // Vertical resolution
    pub fourcc: NDIlib_FourCC_video_type_e, // FourCC (BGRA, UYVY, etc)
    pub frame_rate_n: c_int,          // Frame rate numerator (e.g., 30000)
    pub frame_rate_d: c_int,          // Frame rate denominator (e.g., 1001)
    pub picture_aspect_ratio: c_float, // Picture aspect ratio
    pub frame_format_type: NDIlib_frame_format_type_e, // Progressive/interlaced
    pub timecode: i64,                // Timecode
    pub p_data: *const u8,            // Video data pointer
    pub line_stride_in_bytes: c_int,  // Bytes per line
    pub p_metadata: *const c_char,    // Optional metadata
    pub timestamp: i64,               // Timestamp
}

// Function type definitions
type NDIlib_initialize_fn = unsafe extern "C" fn() -> bool;
type NDIlib_destroy_fn = unsafe extern "C" fn();
type NDIlib_send_create_fn = unsafe extern "C" fn(*const NDIlib_send_create_t) -> *mut c_void;
type NDIlib_send_destroy_fn = unsafe extern "C" fn(*mut c_void);
type NDIlib_send_send_audio_v2_fn = unsafe extern "C" fn(*mut c_void, *const NDIlib_audio_frame_v2_t);
type NDIlib_send_send_video_v2_fn = unsafe extern "C" fn(*mut c_void, *const NDIlib_video_frame_v2_t);

/// Try to load NDI library at runtime
fn load_ndi_lib() -> Option<&'static Library> {
    NDI_LIB.get_or_init(|| {
        // Get executable directory to find bundled library
        let mut paths = vec![];
        
        // Platform-specific library names and paths
        #[cfg(target_os = "macos")]
        const LIB_NAME: &str = "libndi.dylib";
        #[cfg(target_os = "macos")]
        const NATIVE_LIBS_PATH: &str = "native-libs/macos";
        
        #[cfg(target_os = "windows")]
        #[cfg(target_arch = "x86_64")]
        const LIB_NAME: &str = "Processing.NDI.Lib.x64.dll";
        #[cfg(target_os = "windows")]
        #[cfg(target_arch = "x86")]
        const LIB_NAME: &str = "Processing.NDI.Lib.x86.dll";
        #[cfg(target_os = "windows")]
        const NATIVE_LIBS_PATH: &str = "native-libs/windows";
        
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                // App bundle / installed location
                paths.push(exe_dir.join(LIB_NAME).to_string_lossy().to_string());
                
                #[cfg(target_os = "macos")]
                {
                    // Also check in Resources folder on macOS
                    paths.push(exe_dir.join("../Resources").join(LIB_NAME).to_string_lossy().to_string());
                }
            }
        }
        
        // Development: check project root
        paths.push(format!("./{}/{}", NATIVE_LIBS_PATH, LIB_NAME));
        paths.push(format!("../{}/{}", NATIVE_LIBS_PATH, LIB_NAME));
        
        for path in paths {
            if let Ok(lib) = unsafe { Library::new(&path) } {
                eprintln!("[NDI FFI] ✓ Loaded NDI library from: {}", path);
                return Some(lib);
            }
        }
        
        eprintln!("[NDI FFI] ⚠ Could not find NDI library");
        eprintln!("[NDI FFI] Install NDI Tools from: https://ndi.video/tools/");
        None
    }).as_ref()
}

/// NDI Audio Sender wrapper
pub struct NdiSender {
    ptr: *mut c_void,
}

unsafe impl Send for NdiSender {}

impl NdiSender {
    /// Create a new NDI audio sender
    pub fn new(stream_name: &str) -> Option<Self> {
        unsafe {
            let lib = load_ndi_lib()?;
            
            // Get function pointers
            let initialize: Symbol<NDIlib_initialize_fn> = 
                lib.get(b"NDIlib_initialize\0").ok()?;
            let send_create: Symbol<NDIlib_send_create_fn> = 
                lib.get(b"NDIlib_send_create\0").ok()?;
            
            // Initialize NDI
            if !initialize() {
                eprintln!("[NDI FFI] ✗ Failed to initialize NDI");
                return None;
            }
            
            // Create sender
            let name = std::ffi::CString::new(stream_name).ok()?;
            let create_settings = NDIlib_send_create_t {
                p_ndi_name: name.as_ptr(),
                p_groups: std::ptr::null(),
                clock_video: false,
                clock_audio: true, // Enable audio clock
            };
            
            let ptr = send_create(&create_settings);
            if ptr.is_null() {
                eprintln!("[NDI FFI] ✗ Failed to create sender");
                return None;
            }
            
            eprintln!("[NDI FFI] ✓ Sender created: {}", stream_name);
            Some(Self { ptr })
        }
    }
    
    /// Send audio frame to NDI
    pub fn send_audio(&self, samples: &[f32], sample_rate: i32, num_channels: i32) {
        use std::sync::atomic::{AtomicBool, Ordering};
        static FIRST_CALL: AtomicBool = AtomicBool::new(true);
        unsafe {
            if let Some(lib) = load_ndi_lib() {
                let num_samples_per_ch = samples.len() as i32 / num_channels;
                let _max_sample = samples.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
                
                // Convert interleaved to planar format (NDI might prefer this)
                // Interleaved: L R L R L R ... -> Planar: L L L ... R R R ...
                let mut planar = Vec::with_capacity(samples.len());
                
                // Add all left channel samples first
                for i in 0..num_samples_per_ch as usize {
                    planar.push(samples[i * 2]);
                }
                // Then all right channel samples
                for i in 0..num_samples_per_ch as usize {
                    planar.push(samples[i * 2 + 1]);
                }
                
                // Use v2 API
                if let Ok(send_audio) = lib.get::<NDIlib_send_send_audio_v2_fn>(b"NDIlib_send_send_audio_v2\0") {
                    if FIRST_CALL.swap(false, Ordering::Relaxed) {
                        eprintln!("[NDI FFI] Using NDIlib_send_send_audio_v2 with PLANAR format");
                        eprintln!("[NDI FFI] Format: {} samples/ch, {}Hz, {} channels", 
                                  num_samples_per_ch, sample_rate, num_channels);
                    }
                    
                    // Use planar format: channel_stride = bytes per channel
                    let audio_frame = NDIlib_audio_frame_v2_t {
                        sample_rate,
                        no_channels: num_channels,
                        no_samples: num_samples_per_ch,
                        timecode: 0,
                        p_data: planar.as_ptr(),
                        channel_stride_in_bytes: num_samples_per_ch * 4, // 4 bytes per f32, planar layout
                        p_metadata: std::ptr::null(),
                        timestamp: 0,
                    };
                    
                    send_audio(self.ptr, &audio_frame);
                } else {
                    if FIRST_CALL.swap(false, Ordering::Relaxed) {
                        eprintln!("[NDI FFI] ✗ Failed to get send_audio_v2 function");
                    }
                }
            }
        }
    }
    
    /// Send video frame to NDI (BGRA format)
    pub fn send_video(&self, data: &[u8], width: i32, height: i32, fps_num: i32, fps_den: i32) {
        unsafe {
            if let Some(lib) = load_ndi_lib() {
                if let Ok(send_video) = lib.get::<NDIlib_send_send_video_v2_fn>(b"NDIlib_send_send_video_v2\0") {
                    let video_frame = NDIlib_video_frame_v2_t {
                        xres: width,
                        yres: height,
                        fourcc: NDIlib_FourCC_video_type_e::BGRA,
                        frame_rate_n: fps_num,
                        frame_rate_d: fps_den,
                        picture_aspect_ratio: width as f32 / height as f32,
                        frame_format_type: NDIlib_frame_format_type_e::Progressive,
                        timecode: 0, // Match audio timecode
                        p_data: data.as_ptr(),
                        line_stride_in_bytes: width * 4, // BGRA = 4 bytes per pixel
                        p_metadata: std::ptr::null(),
                        timestamp: 0,
                    };
                    
                    send_video(self.ptr, &video_frame);
                }
            }
        }
    }
}

impl Drop for NdiSender {
    fn drop(&mut self) {
        unsafe {
            if let Some(lib) = load_ndi_lib() {
                if let Ok(send_destroy) = lib.get::<NDIlib_send_destroy_fn>(b"NDIlib_send_destroy\0") {
                    send_destroy(self.ptr);
                }
                
                if let Ok(destroy) = lib.get::<NDIlib_destroy_fn>(b"NDIlib_destroy\0") {
                    destroy();
                }
            }
        }
    }
}
