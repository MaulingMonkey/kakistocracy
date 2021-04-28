use crate::io::StaticFile;
use crate::utility::StaticBytesRef;
use crate::windows::*;

use winapi::shared::dxgiformat::*;
use winapi::shared::dxgitype::DXGI_SAMPLE_DESC;
use winapi::um::d3d11::*;

use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::*;
use std::convert::*;
use std::io::{self, Read};
use std::ops::*;
use std::path::*;
use std::ptr::*;
use std::time::SystemTime;



pub(crate) struct BasicTextureCache {
    device:                 mcom::Rc<ID3D11Device>,

    placeholder_2d_error:   mcom::Rc<ID3D11Texture2D>,
    #[allow(dead_code)]
    placeholder_2d_missing: mcom::Rc<ID3D11Texture2D>,

    static_files:           RefCell<HashMap<StaticBytesRef,     Entry2D>>,
    #[allow(dead_code)]
    dynamic_files:          RefCell<HashMap<Cow<'static, Path>, Dynamic<Entry2D>>>,
}

impl BasicTextureCache {
    #[allow(dead_code)]
    pub fn get(device: &mcom::Rc<ID3D11Device>) -> impl Deref<Target = Self> {
        d3d11::device_private_data_get_or_insert(device, || BasicTextureCache::new(device.clone()))
    }

    pub fn new(device: mcom::Rc<ID3D11Device>) -> Self {
        let placeholder_2d_error    = create_texture_rgba_1x1(&device, 0xFF00FFFF).unwrap();
        let placeholder_2d_missing  = create_texture_rgba_1x1(&device, 0xFF00FFFF).unwrap();
        Self {
            device,
            placeholder_2d_error,
            placeholder_2d_missing,
            static_files:   Default::default(),
            dynamic_files:  Default::default(),
        }
    }

    pub fn get_texture_2d_static_file(&self, file: &StaticFile) -> mcom::Rc<ID3D11Texture2D> {
        let mut static_files = self.static_files.borrow_mut();
        let entry = static_files.entry(StaticBytesRef(file.data)).or_insert_with(||
            self.create_entry_2d_bytes_debug_name(file.data, file.path).unwrap_or_else(|err| Entry2D {
                texture:    self.placeholder_2d_error.clone(),
                error:      Some(err),
            })
        );
        entry.texture.clone()
    }

    #[allow(dead_code)]
    pub fn get_texture_2d_static_path(&self, path: &'static Path) -> mcom::Rc<ID3D11Texture2D> {
        let mut dynamic_files = self.dynamic_files.borrow_mut();
        let entry = dynamic_files.entry(Cow::Borrowed(path)).or_insert_with(|| {
            let mut last_mod_time = SystemTime::UNIX_EPOCH;
            let bytes = match Self::read_bytes_mod(path, &mut last_mod_time) { Ok(b) => b, Err(err) => { return Dynamic { common: self.entry_io_error(err), last_mod_time } }, };
            let common = self.create_entry_2d_bytes_debug_name(&bytes[..], &path.to_string_lossy()).unwrap_or_else(|err| Entry2D { texture: self.placeholder_2d_error.clone(), error: Some(err) });
            Dynamic { common, last_mod_time }
        });
        entry.common.texture.clone()
    }
}

impl BasicTextureCache {
    #[allow(dead_code)]
    fn read_bytes_mod(path: &Path, st: &mut SystemTime) -> io::Result<Vec<u8>> {
        *st = SystemTime::UNIX_EPOCH;
        let mut file = std::fs::File::open(path)?;
        let meta = file.metadata()?;
        *st = meta.modified()?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        Ok(buf)
    }

    #[allow(dead_code)]
    fn entry_io_error(&self, err: io::Error) -> Entry2D {
        Entry2D {
            texture:    if err.kind() == io::ErrorKind::NotFound { &self.placeholder_2d_missing } else { &self.placeholder_2d_error }.clone(),
            error:      Some(Box::new(err)),
        }
    }

    fn create_entry_2d_bytes_debug_name(&self, bytes: &[u8], _debug_name: &str) -> Result<Entry2D, Box<dyn std::error::Error>> {
        let mut decoder = png::Decoder::new(bytes);
        decoder.set_transformations(
            // png::ColorType::* is in native (little) endian
            // D3DFMT_* is in big endian
            png::Transformations::BGR           |
            png::Transformations::EXPAND        |
            png::Transformations::GRAY_TO_RGB   |
            png::Transformations::PACKING       |
            png::Transformations::IDENTITY
        );
        let (info, mut reader) = decoder.read_info()?;
        let mut buf = vec![0; info.buffer_size()];
        reader.next_frame(&mut buf)?;
        assert!(info.bit_depth == png::BitDepth::Eight);
        let (buf, fmt, line_size) = match info.color_type {
            png::ColorType::RGB => {
                let mut src = &buf[..];
                let mut buf2 = Vec::<u8>::new();
                buf2.reserve(buf.len() * 4/3);
                while let [b, g, r, ref rest @ ..] = *src {
                    buf2.push(b);
                    buf2.push(g);
                    buf2.push(r);
                    buf2.push(0xFF);
                    src = rest;
                }
                (buf2, DXGI_FORMAT_B8G8R8A8_UNORM_SRGB, info.line_size * 4/3)
            },
            png::ColorType::RGBA => (buf, DXGI_FORMAT_B8G8R8A8_UNORM_SRGB, info.line_size),
            _other => panic!("BUG: png::{:?} not supported by BasicTextureCache, was expected to be normalized to RGB or RGBA", _other), // should've been normalized?
        };

        let mut tex = null_mut();
        let desc = D3D11_TEXTURE2D_DESC {
            Width: info.width, Height: info.height, MipLevels: 1, ArraySize: 1,
            Format: fmt, SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
            Usage: D3D11_USAGE_IMMUTABLE, BindFlags: D3D11_BIND_SHADER_RESOURCE, CPUAccessFlags: 0, MiscFlags: 0,
        };
        let initial_data = D3D11_SUBRESOURCE_DATA {
            pSysMem:            buf.as_ptr().cast(),
            SysMemPitch:        line_size.try_into().unwrap(),
            SysMemSlicePitch:   buf.len().try_into().unwrap(),
        };
        let hr = unsafe { self.device.CreateTexture2D(&desc, &initial_data, &mut tex) };
        let err = Error::check_hr("ID3D11Device::CreateTexture2D", hr, "");
        if cfg!(debug_assertions) {
            err.unwrap();
        } else {
            err?;
        }
        let tex = unsafe { mcom::Rc::from_raw(tex) };
        let _ = unsafe { tex.set_debug_name(_debug_name) };

        Ok(Entry2D { texture: tex, error: None })
    }
}

impl From<mcom::Rc<ID3D11Device>> for BasicTextureCache {
    fn from(device: mcom::Rc<ID3D11Device>) -> Self { Self::new(device) }
}



type BoxError = Box<dyn std::error::Error>;

struct Entry2D {
    pub texture:    mcom::Rc<ID3D11Texture2D>,
    pub error:      Option<BoxError>,
}

struct Dynamic<C> {
    common:         C,
    #[allow(dead_code)] // XXX
    last_mod_time:  SystemTime,
}

fn create_texture_rgba_1x1(device: &mcom::Rc<ID3D11Device>, rgba: u32) -> Result<mcom::Rc<ID3D11Texture2D>, Error> {
    let [r,g,b,a] = rgba.to_le_bytes();
    let bgra = [b,g,r,a];

    let mut tex = null_mut();
    let desc = D3D11_TEXTURE2D_DESC {
        Width: 1, Height: 1, MipLevels: 1, ArraySize: 1,
        Format: DXGI_FORMAT_B8G8R8A8_UNORM_SRGB, SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
        Usage: D3D11_USAGE_IMMUTABLE, BindFlags: D3D11_BIND_SHADER_RESOURCE, CPUAccessFlags: 0, MiscFlags: 0,
    };
    let initial_data = D3D11_SUBRESOURCE_DATA { pSysMem: bgra.as_ptr().cast(), SysMemPitch: 4, SysMemSlicePitch: 4 };
    let hr = unsafe { device.CreateTexture2D(&desc, &initial_data, &mut tex) };
    Error::check_hr("ID3D11Device::CreateTexture2D", hr, "")?;
    let tex = unsafe { mcom::Rc::from_raw(tex) };
    let _ = unsafe { tex.set_debug_name(&format!("create_texture_rgba_1x1(device, 0x{:08x})", rgba)) };

    Ok(tex)
}
