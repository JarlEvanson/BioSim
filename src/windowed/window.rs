#![allow(temporary_cstring_as_ptr)]

use std::{os::raw::c_char, ffi::CString};

extern crate glfw;

use crate::{windowed::shader::Shader, grid::{Grid, self}, cell::Cell, GRID_WIDTH, GRID_HEIGHT, should_reset, pause, neuron_presence, gene::NodeID, grid_display_width, grid_ptr, pop_ptr, accounted_time};

pub struct Window {
    ptr: *mut glfw::ffi::GLFWwindow,
    line_VAO: u32,
    background_VAO: u32,
    cell_VAO: u32,
    vertical_shader: Shader,
    horizontal_shader: Shader,
    background_shader: Shader,
    cell_shader: Shader
}

impl Window {
    pub fn createWindow(width: i32, height: i32) -> Option<Window> {
        let ptr = unsafe {
            glfw::ffi::glfwInit();

            let ptr = glfw::ffi::glfwCreateWindow(
                width, 
                height, 
                CString::new("BIOSIM").unwrap().as_ptr() as *const c_char, 
                std::ptr::null_mut(), 
                std::ptr::null_mut()
            );

            if ptr == std::ptr::null_mut() as *mut glfw::ffi::GLFWwindow {
                return None
            };

            glfw::ffi::glfwMakeContextCurrent(ptr);

            glfw::ffi::glfwSwapInterval(1);

            gl::load_with(|s| loadfn(s));

            glfw::ffi::glfwSetWindowCloseCallback(ptr, Some(windowCloseCallback));

            glfw::ffi::glfwSetFramebufferSizeCallback(ptr, Some(framebufferSizeCallback));

            glfw::ffi::glfwSetKeyCallback(ptr, Some(keyCallback));

            glfw::ffi::glfwSetMouseButtonCallback(ptr, Some(mouseButtonCallback));

            ptr
        };

        let horizontal = Shader::new("shaders/horizontal.vs", "shaders/base.fs");
        let vertical = Shader::new("shaders/vertical.vs", "shaders/base.fs");
        let background = Shader::new("shaders/background.vs", "shaders/base.fs");
        let cell = Shader::new("shaders/cell.vs", "shaders/cell.fs");

        let mut window = Window { ptr: ptr, 
            line_VAO: 0, 
            background_VAO: 0,
            cell_VAO: 0,
            horizontal_shader: horizontal, 
            vertical_shader: vertical,
            background_shader: background,
            cell_shader: cell
        };

        window.create_lines();
        window.create_square_VAOs();

        Some(window)
    }

    fn create_lines(&mut self) {
        let lineVertices: [f32; 2] = [
            1.0, -1.0
        ];

        let mut VAO: u32 = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut VAO);
            gl::BindVertexArray(VAO);
        }

        let mut VBO: u32 = 0;
        unsafe {
            gl::GenBuffers(1, &mut VBO);

            gl::BindBuffer(gl::ARRAY_BUFFER, VBO);

            gl::BufferData(gl::ARRAY_BUFFER, lineVertices.len() as isize * 4, lineVertices.as_ptr() as *const std::ffi::c_void, gl::STATIC_DRAW);
        
            gl::VertexAttribPointer(0, 1, gl::FLOAT, gl::FALSE, 1 * 4, 0 as *const std::ffi::c_void);
        
            gl::EnableVertexAttribArray(0);            
        }

        self.line_VAO = VAO;
    }

    fn create_square_VAOs(&mut self) {
        let backgroundVertices: [f32; 12] = [
            -1.0, -1.0,
            -1.0, 1.0,
            1.0, 1.0,
            1.0, 1.0,
            1.0, -1.0,
            -1.0, -1.0
        ];

        let mut VAO: u32 = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut VAO);
            gl::BindVertexArray(VAO);
        }

        let mut VBO: u32 = 0;
        unsafe {
            gl::GenBuffers(1, &mut VBO);

            gl::BindBuffer(gl::ARRAY_BUFFER, VBO);

            gl::BufferData(gl::ARRAY_BUFFER, backgroundVertices.len() as isize * 4, backgroundVertices.as_ptr() as *const std::ffi::c_void, gl::STATIC_DRAW);
        
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 2 * 4, 0 as *const std::ffi::c_void);
        
            gl::EnableVertexAttribArray(0);            
        }

        self.background_VAO = VAO;

        let cell_vertices: [f32; 12] = [
            0.0, -1.0,
            0.0, 0.0,
            1.0, 0.0,
            1.0, 0.0,
            1.0, -1.0,
            0.0, -1.0
        ];

        let mut VAO: u32 = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut VAO);
            gl::BindVertexArray(VAO);
        }

        let mut VBO: u32 = 0;
        unsafe {
            gl::GenBuffers(1, &mut VBO);

            gl::BindBuffer(gl::ARRAY_BUFFER, VBO);

            gl::BufferData(gl::ARRAY_BUFFER, cell_vertices.len() as isize * 4, cell_vertices.as_ptr() as *const std::ffi::c_void, gl::STATIC_DRAW);

            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 2 * 4, 0 as *const std::ffi::c_void);
            
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }

        self.cell_VAO = VAO;
    }

    pub fn render(&self, living_cells: Vec<&Cell>) {
        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);


            gl::BindVertexArray(self.background_VAO);
            
            self.background_shader.apply();
            self.background_shader.set_uniform_vec3("color", 1.0, 1.0, 1.0);

            gl::DrawArrays(gl::TRIANGLES, 0, 6);

            gl::BindVertexArray(0);

            {

                let mut VAO = 0;

                gl::GenVertexArrays(1, &mut VAO);
                gl::BindVertexArray(VAO);

                let mut VBO = 0;

                gl::GenBuffers(1, &mut VBO);
                gl::BindBuffer(gl::ARRAY_BUFFER, VBO);

                let mut buffer: Vec<f32> = vec![
                    //12 f32s to denote cell vertices
                    0.0, -1.0,
                    0.0, 0.0,
                    1.0, 0.0,
                    1.0, 0.0,
                    1.0, -1.0,
                    0.0, -1.0
                ];

                for cell in &living_cells {
                    buffer.push( ((cell.get_coords().0) as f32) / (GRID_WIDTH as f32) * 2.0 - 1.0);
                    buffer.push( ((cell.get_coords().1 + 1) as f32 ) / (GRID_HEIGHT as f32) * 2.0 - 1.0);
                    buffer.push( ( cell.get_color().0 as f32 / 255.0) );
                    buffer.push( ( cell.get_color().1 as f32 / 255.0) );
                    buffer.push( ( cell.get_color().2 as f32 / 255.0) );
                }

                gl::BufferData(gl::ARRAY_BUFFER, (buffer.len() * 4) as isize, buffer.as_ptr() as *const std::ffi::c_void, gl::STATIC_DRAW);

                gl::EnableVertexAttribArray(0);
                gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 2 * 4, 0 as *const std::ffi::c_void);

                gl::EnableVertexAttribArray(1);
                gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, 5 * 4, (12 * 4) as *const std::ffi::c_void);
                gl::VertexAttribDivisor(1, 1);

                gl::EnableVertexAttribArray(2);
                gl::VertexAttribPointer(2, 3, gl::FLOAT, gl::FALSE, 5 * 4, (14 * 4) as *const std::ffi::c_void);
                gl::VertexAttribDivisor(2, 1);

                self.cell_shader.apply();
                self.cell_shader.set_uniform_int("width", GRID_WIDTH as i32);
                self.cell_shader.set_uniform_int("height", GRID_HEIGHT as i32);

                gl::DrawArraysInstanced(gl::TRIANGLES, 0, 6, living_cells.len() as i32);

            }       

            gl::BindVertexArray(self.line_VAO);

            gl::LineWidth(1.0);

            self.horizontal_shader.apply();
            self.horizontal_shader.set_uniform_vec3("color", 0.0, 0.0, 0.0);
            self.horizontal_shader.set_uniform_int("height", GRID_HEIGHT as i32);
            gl::DrawArraysInstanced(gl::LINES, 0, 2, GRID_HEIGHT as i32);

            self.vertical_shader.apply();
            self.vertical_shader.set_uniform_vec3("color", 0.0, 0.0, 0.0);
            self.vertical_shader.set_uniform_int("width", GRID_WIDTH as i32);
            gl::DrawArraysInstanced(gl::LINES, 0, 2, GRID_HEIGHT as i32);
            

            glfw::ffi::glfwSwapBuffers(self.ptr);
        }
    }

    #[inline]
    pub fn shouldClose(&self) -> bool {
        unsafe {
            return glfw::ffi::glfwWindowShouldClose(self.ptr) == 1
        }
    }

    #[inline]
    pub fn poll(&self) {
        unsafe {
            glfw::ffi::glfwPollEvents();
        }
    }

    #[inline]
    pub fn make_current(&self) {
        unsafe {
            glfw::ffi::glfwMakeContextCurrent(self.ptr);
        }
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            glfw::ffi::glfwTerminate();
        }
    }
}

extern "C" fn windowCloseCallback(window: *mut glfw::ffi::GLFWwindow) {
    unsafe {
        glfw::ffi::glfwSetWindowShouldClose(window, glfw::ffi::TRUE);
    }
}

extern "C" fn framebufferSizeCallback(window: *mut glfw::ffi::GLFWwindow, width: i32, height: i32) {
    unsafe {
        crate::framebuffer_width = width as u32;
        crate::framebuffer_height = height as u32;
        if width > height {
            gl::Viewport(0, 0, height, height);
            crate::grid_display_width = height as u32;
        } else if height >= width {
            gl::Viewport(0, (height - width) / 2, width, width);
            crate::grid_display_width = width as u32;
        }
    }
}

extern "C" fn keyCallback(window: *mut glfw::ffi::GLFWwindow, key: i32, scancode: i32, action: i32, mods: i32) {
    if key == glfw::ffi::KEY_R && action == glfw::ffi::PRESS {
        unsafe { should_reset = true; }
    } else if key == glfw::ffi::KEY_SPACE && action == glfw::ffi::PRESS {
        unsafe { pause = !pause; }
    } else if key == glfw::ffi::KEY_E && action == glfw::ffi::PRESS {
        print!("\nNeuron Frequencies: \n");
        unsafe {
            for index in 0 .. neuron_presence.len() {
                println!("{}: {}", NodeID::from_index(index), neuron_presence[index]);
            }
            println!("");
        }
    } else if key == glfw::ffi::KEY_ESCAPE {
        unsafe { 
            glfw::ffi::glfwSetWindowShouldClose(window, glfw::ffi::TRUE) 
        };
    }
}

extern "C" fn mouseButtonCallback(window: *mut glfw::ffi::GLFWwindow, button: i32, action: i32, mods: i32) {
    unsafe {
        if action == glfw::ffi::PRESS {
            let (mut x, mut y) = (0.0, 0.0);
            glfw::ffi::glfwGetCursorPos(window, &mut x, &mut y);

            if x as u32 <= crate::grid_display_width && y as u32 <= crate::grid_display_width {

                let cell_x = ((x as f32 / (grid_display_width as f32)) * GRID_WIDTH as f32) as u32;
                let cell_y = (((crate::grid_display_width - y as u32) as f32 / (grid_display_width as f32)) * GRID_HEIGHT  as f32) as u32;
                let cell_index = (*grid_ptr).get_occupant(cell_x, cell_y);

                if cell_index != None {
                    println!("{:#?}", (*pop_ptr).get_cell(cell_index.unwrap_unchecked() as usize));
                }

                let cell_indices = (*grid_ptr).get_in_radius((cell_x, cell_y), 2.0);
                println!("Cells Found: {}\nCell Locations", cell_indices.len());
            
            }

            
        }
    }
}



fn loadfn(symbol: &'static str) -> glfw::ffi::GLFWglproc {
    glfw::with_c_str(symbol, |procname| unsafe {
        glfw::ffi::glfwGetProcAddress(procname)
    })
}

pub fn wait(window: &Window, secs: f64) {
    
    while unsafe { glfw::ffi::glfwGetTime() } - unsafe { accounted_time } < secs {
        window.poll();
        if window.shouldClose() {
            break;
        }  
        if unsafe { pause } {
            unsafe { accounted_time = glfw::ffi::glfwGetTime(); }
        }
    }

    unsafe { accounted_time += secs; }
}