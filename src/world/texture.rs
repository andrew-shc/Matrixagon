use vulkano::device::Queue;
use vulkano::image::{ImmutableImage, Dimensions};
use vulkano::format::Format;
use vulkano::command_buffer::{CommandBufferExecFuture, AutoCommandBuffer};
use vulkano::sync::NowFuture;

use std::sync::Arc;
use std::io::Cursor;
use std::collections::BTreeMap;
use std::path::Path;
use std::{mem, fs};

use png;


#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
// TextureID: Numerical ID, Name Tag
pub struct TextureID(pub u32, pub &'static str);


pub struct Texture {
    queue: Arc<Queue>,

    txtr_width: u32,
    txtr_height: u32,
    txtr_cnt: u32,  // texture ID counter

    textures: Vec<(Vec<u8>, TextureID)>,  // raw texture info: (texture RGBA data, txtr-id)
}

impl Texture {
    pub fn new(queue: Arc<Queue>) -> Self {
        Self {
            queue: queue.clone(),

            txtr_width: 16,
            txtr_height: 16,
            txtr_cnt: 0,
            textures: Vec::new(),
        }
    }

    pub fn id_name(&self, name: &str) -> Option<TextureID> {
        let mut texture_id = None;
        for (_, tid) in self.textures.iter() {
            if tid.1 == name {
                texture_id = Some(*tid);
                break;
            }
        };
        texture_id  // clones the ID, then unwraps it to deref the internal data and then wrap it again with Some
    }

    // adds the texture data
    pub fn add_texture(&mut self, file_name: &str, txtr_name: &'static str) -> TextureID {
        // retrieves the .png byte data from the file
        let byte_stream = fs::read(Path::new(file_name)).expect(&format!("Texture file '{}' not found!", file_name)[..]);

        // decodes file meta information
        let cursor = Cursor::new(byte_stream);
        let decoder = png::Decoder::new(cursor);
        let (info, mut reader) = decoder.read_info().unwrap();

        if info.height != self.txtr_height {
            println!("Warning: The texture '{}' has a height of {}, but the program is expecting a texture height of {}",
                     file_name, info.height, self.txtr_height);
        }
        if info.width != self.txtr_width {
            println!("Warning: The texture '{}' has a width of {}, but the program is expecting a texture width of {}",
                     file_name, info.height, self.txtr_height);
        }

        // formats decoded data into a texture RGBA format data
        let mut txtr_data = Vec::new();
        txtr_data.resize((info.width * info.height * 4) as usize, 0);
        reader.next_frame(&mut txtr_data).unwrap();

        let id = TextureID(self.txtr_cnt, txtr_name);
        self.txtr_cnt += 1;

        self.textures.push((txtr_data, id));

        id
    }

    // builds all the texture datas into a single texture array buffer
    pub fn texture_future(&mut self)
        -> (Arc<ImmutableImage<Format>>, CommandBufferExecFuture<NowFuture, AutoCommandBuffer>) {
        let dimensions = Dimensions::Dim2dArray {
            width: self.txtr_width,
            height: self.txtr_height,
            array_layers: self.textures.len() as u32
        };

        let mut all_textures = Vec::new();
        for (dt, id) in self.textures.iter() {
            all_textures.append(&mut dt.clone());
        }

        let (texture, future) = ImmutableImage::from_iter(
            all_textures.into_iter(),
            dimensions,
            Format::R8G8B8A8Unorm,
            self.queue.clone(),
        ).unwrap();

        // (actual texture data, command buffer)
        (texture, future)
    }
}
