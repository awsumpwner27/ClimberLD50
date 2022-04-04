use gl::types::*;

static mut SPRITE_PRG: GLuint = 0;
static mut SPRITE_VAO: GLuint = 0;
static mut SPRITE_TNFM_UNI: GLint = 0;
static mut SPRITE_VIEW_UNI: GLint = 0;
static mut SPRITE_TEXA_UNI: GLint = 0;
static mut SPRITE_TEXB_UNI: GLint = 0;

#[derive(Clone, Copy)]
pub struct Sprite {
    pub transform: Transform,
    pub texture: Texture,
    pub sub_tex: (Vector2, Vector2),
}

impl Sprite {
    pub fn init() {
        let mut vbo: GLuint = 0;
        let mut ibo: GLuint = 0;
        let (pos_attrib, tex_attrib);
        let vertices: [Vertex; 4] = [
            Vertex { position: ( 1.0,  1.0).into(), tex_coord: (1.0, 0.0).into(), },
            Vertex { position: (-1.0,  1.0).into(), tex_coord: (0.0, 0.0).into(), },
            Vertex { position: (-1.0, -1.0).into(), tex_coord: (0.0, 1.0).into(), },
            Vertex { position: ( 1.0, -1.0).into(), tex_coord: (1.0, 1.0).into(), },
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
                let attr_str = CStr::from_bytes_with_nul(b"position\0").unwrap();
                pos_attrib = gl::GetAttribLocation(SPRITE_PRG, attr_str.as_ptr());
            }
            gl::VertexAttribPointer(
                pos_attrib as _, 2, gl::FLOAT, gl::FALSE,
                std::mem::size_of::<Vertex>() as _,
                std::ptr::null::<u8>().add(0) as _
            );
            gl::EnableVertexAttribArray(pos_attrib as _);

            {
                use std::ffi::CStr;
                let attr_str = CStr::from_bytes_with_nul(b"texCoord\0").unwrap();
                tex_attrib = gl::GetAttribLocation(SPRITE_PRG, attr_str.as_ptr());
            }
            gl::VertexAttribPointer(
                tex_attrib as _, 2, gl::FLOAT, gl::FALSE,
                std::mem::size_of::<Vertex>() as _,
                std::ptr::null::<u8>().add(std::mem::size_of::<Vector2>()) as _
            );
            gl::EnableVertexAttribArray(tex_attrib as _);
        }
    }

    pub fn new(texture: Texture) -> Self {
        Self {
            transform: Transform::identity(),
            texture,
            sub_tex: (Vector2::zero(), Vector2::one())
        }
    }

    pub fn begin(view: Transform) {
        unsafe {
            gl::BindVertexArray(SPRITE_VAO);
            gl::UseProgram(SPRITE_PRG);
            
            gl::UniformMatrix3fv(
                SPRITE_VIEW_UNI, 1, gl::FALSE, view.matrix().0.as_ptr()
            );
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
            gl::Uniform2fv(SPRITE_TEXA_UNI, 1, &self.sub_tex.0.x);
            gl::Uniform2fv(SPRITE_TEXB_UNI, 1, &self.sub_tex.1.x);
            gl::BindTexture(gl::TEXTURE_2D, self.texture.gl_id);
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
                in vec2 texCoord;

                out vec2 texCoordFrag;

                uniform mat3 view;
                uniform mat3 tnfm;
                uniform vec2 subTexA;
                uniform vec2 subTexB; 

                void main() {
                    vec3 pos = inverse(view) * tnfm * vec3(position, 1.0);

                    texCoordFrag = mix(subTexA, subTexB, texCoord);
                    gl_Position = vec4(pos.xy, 0.0, 1.0);
                }
            \0").unwrap(),
            CStr::from_bytes_with_nul(b"
            #version 150

            in vec2 texCoordFrag;

            out vec4 outColour;

            uniform sampler2D tex;

            void main() {
                outColour = texture(tex, texCoordFrag);
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
            let view_str = CStr::from_bytes_with_nul(b"view\0").unwrap();
            SPRITE_VIEW_UNI = gl::GetUniformLocation(SPRITE_PRG, view_str.as_ptr());
            let texa_str = CStr::from_bytes_with_nul(b"subTexA\0").unwrap();
            SPRITE_TEXA_UNI = gl::GetUniformLocation(SPRITE_PRG, texa_str.as_ptr());
            let texb_str = CStr::from_bytes_with_nul(b"subTexB\0").unwrap();
            SPRITE_TEXB_UNI = gl::GetUniformLocation(SPRITE_PRG, texb_str.as_ptr());
        }
    }
}

#[derive(Clone, Copy)]
pub struct Transform {
    pub translation: Vector2,
    pub rotation: f32,
    pub scale: Vector2,
    pub origin: Vector2,
}

impl Transform {
    pub fn identity() -> Self {
        Self {
            translation: Vector2::zero(),
            rotation: 0.0,
            scale: Vector2::one(),
            origin: Vector2::zero(),
        }
    }

    pub fn matrix(&self) -> Matrix3 {
        let (tx, ty) = self.translation.tuple();
        let (rs, rc) = (self.rotation.sin(), self.rotation.cos());
        let (sx, sy) = self.scale.tuple();
        let o = self.origin.tuple();
        let (ox, oy) = (
            -o.0 * rc * sx + o.1 * rs * sx,
            -o.0 * rs * sy - o.1 * rc * sy
        );

        Matrix3 ([
            sx *  rc, sx * rs, 0.0,
            sy * -rs, sy * rc, 0.0,
             ox + tx, oy + ty, 1.0,
        ])
    }
}

#[repr(C)]
struct Vertex {
    position: Vector2,
    tex_coord: Vector2,
}

#[derive(Copy, Clone)]
pub struct Texture {
    gl_id: GLuint,
}

impl Texture {

    pub fn new(file_path: &std::path::Path) -> Self {
        use std::fs::File;

        let decoder = png::Decoder::new(File::open(file_path).unwrap());
        let mut reader = decoder.read_info().unwrap();
        let mut buf = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut buf).unwrap();
        let bytes = &buf[..info.buffer_size()];

        assert_eq!(info.color_type, png::ColorType::Rgba, "Non-RGBA image");

        let mut gl_id = 0;
        unsafe {
            gl::GenTextures(1, &mut gl_id);
            gl::BindTexture(gl::TEXTURE_2D, gl_id);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as _);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as _);

            gl::TexImage2D(
                gl::TEXTURE_2D, 0, gl::RGBA as _,
                info.width as _, info.height as _, 0, gl::RGBA, gl::UNSIGNED_BYTE,
                bytes.as_ptr() as _
            );
        }

        Self { gl_id }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
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

    pub fn scale(self, k: f32) -> Self {
        (k * self.x, k * self.y).into()
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

impl std::ops::Add for Vector2 {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        (self.x + other.x, self.y + other.y).into()
    }
}

#[repr(C)]
pub struct Matrix3 ([f32; 9]);
