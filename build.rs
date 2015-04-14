extern crate gl_generator;
extern crate khronos_api;

use std::os;
use std::env;
use std::fs::File;
use std::path::Path;

fn main() {
    let out = &env::var("OUT_DIR").unwrap();
    let dest = Path::new(out);

    let mut file = File::create(&dest.join("gl_bindings.rs")).unwrap();

    // This generates bindsings for OpenGL ES v2.0
    gl_generator::generate_bindings(gl_generator::GlobalGenerator,
                                    gl_generator::registry::Ns::Gles2,
                                    gl_generator::Fallbacks::All,
                                    khronos_api::GL_XML,
                                    vec![],
                                    "2.0", "core", &mut file).unwrap();
}
