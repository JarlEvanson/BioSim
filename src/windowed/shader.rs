use std::io::Read;



pub struct Shader {
    ID: u32
}

impl Shader {
    pub fn new(vertexPath: &str, fragmentPath: &str) -> Shader {
        println!("Loading shader of {} and {}", vertexPath, fragmentPath);        

        let mut vertexFile = std::fs::File::open(vertexPath).expect("Failed to open Vertex Shader File");
        let mut fragmentFile = std::fs::File::open(fragmentPath).expect("Failed to open Fragment Shader File");

        let mut vertexString = Vec::new();
        let mut fragmentString = Vec::new();

        vertexFile.read_to_end(&mut vertexString).expect("Failed to read vertex file");
        fragmentFile.read_to_end(&mut fragmentString).expect("Failed to read fragment file");

        let vertexString = std::ffi::CString::new(vertexString).expect("Failed to convert vertex file data");
        let fragmentString = std::ffi::CString::new(fragmentString).expect("Failed to convert vertex file data");

        let vertexShader: u32;
        let fragmentShader: u32;
        let mut success: i32 = 0;

        let mut infoLog = vec![0; 512];

        vertexShader = unsafe { gl::CreateShader(gl::VERTEX_SHADER) };
        unsafe { 
            gl::ShaderSource(vertexShader, 1, &vertexString.as_ptr(), std::ptr::null());
            gl::CompileShader(vertexShader);
            gl::GetShaderiv(vertexShader, gl::COMPILE_STATUS, &mut success as *mut i32);
            if !(success == 1) {
                gl::GetShaderInfoLog(vertexShader, 512, std::ptr::null_mut(), infoLog.as_mut_ptr());
                let mut infoLog = infoLog.clone();
                let infoLog = Vec::from_raw_parts(infoLog.as_mut_ptr() as *mut u8, infoLog.len(), infoLog.capacity());
                println!("ERROR::SHADER::VERTEX::COMPILATION_FAILED\n{}", std::string::String::from_utf8_unchecked(infoLog));
            }
        }

        infoLog.clear();

        fragmentShader = unsafe { gl::CreateShader(gl::FRAGMENT_SHADER) };
        unsafe { 
            gl::ShaderSource(fragmentShader, 1, &fragmentString.as_ptr(), std::ptr::null());
            gl::CompileShader(fragmentShader);
            gl::GetShaderiv(fragmentShader, gl::COMPILE_STATUS, &mut success as *mut i32);
            if !(success == 1) {
                gl::GetShaderInfoLog(fragmentShader, 512, std::ptr::null_mut(), infoLog.as_mut_ptr());
                let mut infoLog = infoLog.clone();
                let infoLog = Vec::from_raw_parts(infoLog.as_mut_ptr() as *mut u8, infoLog.len(), infoLog.capacity());
                println!("ERROR::SHADER::FRAGMENT::COMPILATION_FAILED\n{}", std::string::String::from_utf8_unchecked(infoLog));
            }
        }

        infoLog.clear();

        let ID = unsafe { 
            gl::CreateProgram()
        };

        unsafe {
            gl::AttachShader(ID, vertexShader);
            gl::AttachShader(ID, fragmentShader);
            gl::LinkProgram(ID);

            if !(success == 1) {
                gl::GetProgramInfoLog(ID, 512, std::ptr::null_mut(), infoLog.as_mut_ptr());
                let mut infoLog = infoLog.clone();
                let infoLog = Vec::from_raw_parts(infoLog.as_mut_ptr() as *mut u8, infoLog.len(), infoLog.capacity());
                println!("ERROR::SHADER::PROGRAM::COMPILATION_FAILED\n{}", std::string::String::from_utf8_unchecked(infoLog));
            }

            gl::DeleteShader(vertexShader);
            gl::DeleteShader(fragmentShader);
        }

        Shader { ID: ID }
    }

    pub fn apply(&self) {
        unsafe { gl::UseProgram(self.ID) };
    } 

    pub fn set_uniform_vec2(&self, name: &str, v0: f32, v1: f32) {
        unsafe { 
            let location = gl::GetUniformLocation(self.ID, std::ffi::CString::new(name).expect("Failed to convert uniform location").as_ptr());
            gl::Uniform2f(location, v0, v1);
         }
    }

    pub fn set_uniform_vec3(&self, name: &str, v0: f32, v1: f32, v2: f32) {
        unsafe { 
            let location = gl::GetUniformLocation(self.ID, std::ffi::CString::new(name).expect("Failed to convert uniform location").as_ptr());
            gl::Uniform3f(location, v0, v1, v2);
         }
    }

    pub fn set_uniform_int(&self, name: &str, v0: i32) {
        unsafe { 
            let location = gl::GetUniformLocation(self.ID, std::ffi::CString::new(name).expect("Failed to convert uniform location").as_ptr());
            gl::Uniform1i(location, v0);
         }
    }
}