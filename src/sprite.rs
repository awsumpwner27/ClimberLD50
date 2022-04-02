use gl::types::*;

static mut SPRITE_PRG: GLuint = 0;
static mut SPRITE_VAO: GLuint = 0;
static mut SPRITE_TNFM_UNI: GLint = 0;

pub struct Sprite {
    pub transform: Transform,
}

impl Sprite {
    pub fn init() {
        let mut vbo: GLuint = 0;
        let mut ibo: GLuint = 0;
        let pos_attrib;
        let vertices: [Vector2; 4] = [
            (-1.0, -1.0).into(), ( 1.0, -1.0).into(),
            ( 1.0,  1.0).into(), (-1.0,  1.0).into(),
        ];
        let indices: [GLuint; 6] = [
            0, 1, 2,
            2, 3, 0,
        ];

        unsafe {
            gl::GenVertexArrays(1, &mut SPRITE_VAO as _);
            gl::BindVertexArray(SPRITE_VAO);

            gl::GenBuffers(1, &mut vbo as _);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                std::mem::size_of_val(&vertices) as _, vertices.as_ptr() as _,
                gl::STATIC_DRAW
            );

            gl::GenBuffers(1, &mut ibo as _);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                std::mem::size_of_val(&indices) as _, indices.as_ptr() as _,
                gl::STATIC_DRAW
            );

            Self::init_program();

            {
                use std::ffi::CStr;
                let pos_str = CStr::from_bytes_with_nul(b"position\0").unwrap();
                pos_attrib = gl::GetAttribLocation(SPRITE_PRG, pos_str.as_ptr());
            }
            gl::VertexAttribPointer(pos_attrib as _, 2, gl::FLOAT, gl::FALSE, 0, std::ptr::null());
            gl::EnableVertexAttribArray(pos_attrib as _);
        }
    }

    pub fn new() -> Self {
        Self {
            transform: Transform::identity(),
        }
    }

    pub fn begin() {
        unsafe {
            gl::BindVertexArray(SPRITE_VAO);
            gl::UseProgram(SPRITE_PRG);
        }
    }
    
    pub fn end() {
        unsafe {
            gl::BindVertexArray(0);
            gl::UseProgram(0);
        }
    }

    pub fn draw(&self) {
        unsafe {
            gl::UniformMatrix3fv(
                SPRITE_TNFM_UNI, 1, gl::FALSE, self.transform.matrix().0.as_ptr()
            );
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());
        }
    }

    unsafe fn init_program() {
        use std::mem::transmute;
        use std::ffi::CStr;

        let shader_srcs = (
            CStr::from_bytes_with_nul(b"
                #version 150

                in vec2 position;

                uniform mat3 tnfm;

                void main() {
                    vec3 pos = tnfm * vec3(position, 1.0);

                    gl_Position = vec4(pos.xy, 0.0, 1.0);
                }
            \0").unwrap(),
            CStr::from_bytes_with_nul(b"
            #version 150

            out vec4 outColour;

            void main() {
                outColour = vec4(1.0);
            }
            \0").unwrap(),
        );
        let shader_lens: (GLint, GLint) = (
            shader_srcs.0.to_bytes().len() as _,
            shader_srcs.1.to_bytes().len() as _,
        );
        let (vert, frag);

        vert = gl::CreateShader(gl::VERTEX_SHADER);
        frag = gl::CreateShader(gl::FRAGMENT_SHADER);

        gl::ShaderSource(
            vert, 1,
            transmute(&shader_srcs.0.to_bytes_with_nul().as_ptr()), &shader_lens.0
        );
        gl::ShaderSource(
            frag, 1,
            transmute(&shader_srcs.1.to_bytes_with_nul().as_ptr()), &shader_lens.1
        );

        gl::CompileShader(vert); {
            let mut status: GLint = 0;

            gl::GetShaderiv(vert, gl::COMPILE_STATUS, &mut status as _);
            if status != gl::TRUE.into() {
                let mut bytes = [0u8; 512];
                let mut len = 0;
                let err_msg;

                gl::GetShaderInfoLog(vert, 512, &mut len, bytes.as_mut_ptr() as _);
                len += 1;
                err_msg = CStr::from_bytes_with_nul(&bytes[..(len as _)])
                    .unwrap()
                    .to_str()
                    .unwrap();
                panic!("GLSL compiler error: {}", err_msg);
            }
        }
        gl::CompileShader(frag); {
            let mut status: GLint = 0;

            gl::GetShaderiv(frag, gl::COMPILE_STATUS, &mut status as _);
            if status != gl::TRUE.into() {
                let mut bytes = [0u8; 512];
                let mut len = 0;
                let err_msg;

                gl::GetShaderInfoLog(vert, 512, &mut len, bytes.as_mut_ptr() as _);
                len += 1;
                err_msg = CStr::from_bytes_with_nul(&bytes[..(len as _)])
                    .unwrap()
                    .to_str()
                    .unwrap();
                panic!("GLSL compiler error: {}", err_msg);
            }
        }

        SPRITE_PRG = gl::CreateProgram();
        gl::AttachShader(SPRITE_PRG, vert);
        gl::AttachShader(SPRITE_PRG, frag);
        gl::LinkProgram(SPRITE_PRG);
        
        gl::UseProgram(SPRITE_PRG);

        {
            let tnfm_str = CStr::from_bytes_with_nul(b"tnfm\0").unwrap();
            SPRITE_TNFM_UNI = gl::GetUniformLocation(SPRITE_PRG, tnfm_str.as_ptr());
        }
    }
}

pub struct Transform {
    pub translation: Vector2,
    pub scale: Vector2,
    pub rotation: f32,
}

impl Transform {
    pub fn identity() -> Self {
        Self {
            translation: Vector2::zero(),
            scale: Vector2::one(),
            rotation: 0.0,
        }
    }

    pub fn matrix(&self) -> Matrix3 {
        let (tx, ty) = self.translation.tuple();
        let (sx, sy) = self.scale.tuple();
        let (rs, rc) = (self.rotation.sin(), self.rotation.cos());

        Matrix3 ([
            sx *  rc, sx * rs, 0.0,
            sy * -rs, sy * rc, 0.0,
                 *tx,     *ty, 1.0,
        ])
    }
}

#[repr(C)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    pub fn one() -> Self {
        Self { x: 1.0, y: 1.0 }
    }

    pub fn tuple(&self) -> (&f32, &f32) {
        (&self.x, &self.y)
    }
}

impl From<(f32, f32)> for Vector2 {
    fn from(item: (f32, f32)) -> Self {
        Vector2 { x: item.0, y: item.1 }
    }
}

#[repr(C)]
pub struct Matrix3 ([f32; 9]);
