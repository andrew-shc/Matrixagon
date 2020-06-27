use vulkano::device::Queue;
use vulkano::image::{ImmutableImage, Dimensions};
use vulkano::format::Format;
use vulkano::command_buffer::{CommandBufferExecFuture, AutoCommandBuffer};
use vulkano::sync::NowFuture;

use std::sync::Arc;
use std::io::Cursor;
use std::collections::BTreeMap;
use std::mem;

use png;


#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct TextureID(pub u32, pub &'static str);

pub struct Texture {
    queue: Arc<Queue>,

    txtr_cnt: u32,  // texture ID counter
    textures: BTreeMap<TextureID, TextureComponent>,
    futures: Vec<CommandBufferExecFuture<NowFuture, AutoCommandBuffer>>,
}

impl Texture {
    pub fn new(queue: Arc<Queue>) -> Self {
        Self {
            queue: queue.clone(),

            txtr_cnt: 0,
            textures: BTreeMap::new(),
            futures: Vec::new(),
        }
    }

    pub fn add(&mut self, txtr_bytes: Vec<u8>, name: &'static str)
        -> TextureID {
        let (texture, future) = TextureComponent::new(self.queue.clone(), txtr_bytes, name);
        let id = TextureID(self.txtr_cnt, name);

        self.textures.insert(id.clone(), texture);
        self.futures.push(future);
        self.txtr_cnt += 1;

        id
    }

    pub fn futures(&mut self) -> Vec<CommandBufferExecFuture<NowFuture, AutoCommandBuffer>> {
        let fut = mem::replace(&mut self.futures, Vec::new());
        fut
    }

    pub fn id_name(&self, name: String) -> Option<TextureID> {
        let mut texture_id = None;
        for (id, _) in self.textures.iter() {
            if id.1 == name {
                texture_id = Some(id.clone());
            }
        };
        texture_id  // clones the ID, then unwraps it to deref the internal data and then wrap it again with Some
    }

    pub fn texture_id(&self, id: &TextureID) -> &TextureComponent {
        self.textures.get(id).expect(format!("Invalid Texture ID: {:?}", id).as_str())
    }

    pub fn texture_name(&self, name: String) -> &TextureComponent {
        let mut texture = None;
        for id in self.textures.keys() {
            if id.1 == name {
                if let Some(t) = self.textures.get(id) {
                    texture = Some(t);
                }
            }
        }
        texture.expect(format!("Expected textures or inavlid texture: {}", name).as_str())
    }


    // TODO: might need to turn it into a static array
    pub fn texture_array(&self) -> Vec<Arc<ImmutableImage<Format>>> {
        let mut textures = Vec::new();
        for (_id, txtr) in self.textures.iter() {
            textures.push(txtr.texture.clone());
        }
        textures
    }

    pub fn texture_len(&self) -> usize {
        self.textures.len()
    }
}

pub struct TextureComponent {
    texture: Arc<ImmutableImage<Format>>,
    name: &'static str,  // internal texture name used internally in game
}

impl TextureComponent {
    pub fn new(queue: Arc<Queue>, txtr_bytes: Vec<u8>, name: &'static str)
        -> (Self, CommandBufferExecFuture<NowFuture, AutoCommandBuffer>) {
        let cursor = Cursor::new(txtr_bytes);
        let decoder = png::Decoder::new(cursor);
        let (info, mut reader) = decoder.read_info().unwrap();
        let dimensions = Dimensions::Dim2d { width: info.width, height: info.height };
        let mut txtr_data = Vec::new();
        txtr_data.resize((info.width * info.height * 4) as usize, 0);
        reader.next_frame(&mut txtr_data).unwrap();

        let (texture, future) = ImmutableImage::from_iter(
            txtr_data.into_iter(),
            dimensions,
            Format::R8G8B8A8Unorm,
            queue.clone(),
        ).unwrap();

        (
            Self {
                texture: texture,
                name: name,
            },
            future,
        )
    }
}
