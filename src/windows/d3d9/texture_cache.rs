use crate::utility::StaticBytesRef;
use crate::windows::*;

use winapi::shared::d3d9::*;
use winapi::shared::d3d9types::*;
use winapi::shared::winerror::HRESULT;
use winapi::um::unknwnbase::IUnknown;

use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::*;
use std::io::{self, Read};
use std::ops::*;
use std::path::*;
use std::ptr::*;
use std::time::SystemTime;



pub struct BasicTextureCache {
    device:                 mcom::Rc<IDirect3DDevice9>,

    placeholder_2d_error:   mcom::Rc<IDirect3DTexture9>,
    placeholder_2d_missing: mcom::Rc<IDirect3DTexture9>,

    static_files:           RefCell<HashMap<StaticBytesRef,     Entry2D>>,
    dynamic_files:          RefCell<HashMap<Cow<'static, Path>, Dynamic<Entry2D>>>,
}

impl BasicTextureCache {
    pub fn get(device: &mcom::Rc<IDirect3DDevice9>) -> impl Deref<Target = Self> {
        let pdguid = type_guid::<Self>();
        let bb = device.get_back_buffer(0, 0).unwrap();
        match unsafe { bb.get_private_data_com::<IUnknown>(&pdguid) } {
            Ok(btc) => UnkWrapRc::from_com_unknown(&btc).unwrap(),
            Err(err) if err.hresult() == D3DERR_NOTFOUND => {
                let btc = UnkWrapRc::new(Self::new(device.clone()));
                bb.set_private_data_com(&pdguid, &btc.to_com_unknown()).unwrap();
                btc
            },
            Err(err) => panic!("{}", err),
        }
    }

    pub fn new(device: mcom::Rc<IDirect3DDevice9>) -> Self {
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

    pub fn get_texture_2d_static_file(&self, bytes: &'static [u8]) -> mcom::Rc<IDirect3DTexture9> {
        let mut static_files = self.static_files.borrow_mut();
        let entry = static_files.entry(StaticBytesRef(bytes)).or_insert_with(||
            self.entry_2d_file_bytes(bytes).unwrap_or_else(|err| Entry2D {
                texture:    self.placeholder_2d_error.clone(),
                error:      Some(err),
            })
        );
        entry.texture.clone()
    }

    pub fn get_texture_2d_static_path(&self, path: &'static Path) -> mcom::Rc<IDirect3DTexture9> {
        let mut dynamic_files = self.dynamic_files.borrow_mut();
        let entry = dynamic_files.entry(Cow::Borrowed(path)).or_insert_with(|| {
            let mut last_mod_time = SystemTime::UNIX_EPOCH;
            let bytes = match Self::read_bytes_mod(path, &mut last_mod_time) { Ok(b) => b, Err(err) => { return Dynamic { common: self.entry_io_error(err), last_mod_time } }, };
            let common = self.entry_2d_file_bytes(&bytes[..]).unwrap_or_else(|err| Entry2D { texture: self.placeholder_2d_error.clone(), error: Some(err) });
            Dynamic { common, last_mod_time }
        });
        entry.common.texture.clone()
    }
}

impl BasicTextureCache {
    fn read_bytes_mod(path: &Path, st: &mut SystemTime) -> io::Result<Vec<u8>> {
        *st = SystemTime::UNIX_EPOCH;
        let mut file = std::fs::File::open(path)?;
        let meta = file.metadata()?;
        *st = meta.modified()?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        Ok(buf)
    }

    fn entry_io_error(&self, err: io::Error) -> Entry2D {
        Entry2D {
            texture:    if err.kind() == io::ErrorKind::NotFound { &self.placeholder_2d_missing } else { &self.placeholder_2d_error }.clone(),
            error:      Some(Box::new(err)),
        }
    }

    fn entry_2d_file_bytes(&self, bytes: &[u8]) -> Result<Entry2D, Box<dyn std::error::Error>> {
        let decoder = png::Decoder::new(bytes);
        let (info, mut reader) = decoder.read_info()?;
        let mut buf = vec![0; info.buffer_size()];
        reader.next_frame(&mut buf)?;
        assert!(info.bit_depth == png::BitDepth::Eight);
        let mut swap = &mut buf[..];
        let fmt = match info.color_type {
            // png::ColorType::* is in native (little) endian
            // D3DFMT_* is in big endian

            png::ColorType::RGB => {
                while let [r, _g, b, rest @ ..] = swap {
                    std::mem::swap(r, b); // RGB => BGR
                    swap = rest;
                }
                D3DFMT_R8G8B8
            },
            png::ColorType::RGBA => {
                while let [r, _g, b, _a, rest @ ..] = swap {
                    std::mem::swap(r, b); // RGBA => BGRA
                    swap = rest;
                }
                D3DFMT_A8R8G8B8
            },
            _other => panic!("BUG: png::{:?} not supported by BasicTextureCache, was expected to be normalized to RGB or RGBA", _other), // should've been normalized?
        };

        let mut tex = null_mut();
        let hr = unsafe { self.device.CreateTexture(info.width, info.height, 1, 0, fmt, D3DPOOL_DEFAULT, &mut tex, null_mut()) };
        Error::check_hr("IDirect3DDevice9::CreateTexture", hr, "")?;
        let tex = unsafe { mcom::Rc::from_raw(tex) };

        let mut lock = unsafe { std::mem::zeroed() };
        let hr = unsafe { tex.LockRect(0, &mut lock, null(), 0) };
        Error::check_hr("IDirect3DTexture9::LockRect", hr, "")?;

        let dst_pitch = lock.Pitch as usize;
        let dst_scan0 : *mut u8 = lock.pBits.cast();
        debug_assert!(dst_pitch >= info.line_size);
        for y in 0 .. info.height as usize {
            let dst_scany = unsafe { dst_scan0.add(lock.Pitch as usize * y) };
            let src_start = y * info.line_size;
            let src_end = src_start + info.line_size;
            let src = &buf[src_start .. src_end];
            unsafe { std::ptr::copy_nonoverlapping(src.as_ptr(), dst_scany, src.len()) };
        }

        let hr = unsafe { tex.UnlockRect(0) };
        Error::check_hr("IDirect3DTexture9::UnlockRect", hr, "")?;

        Ok(Entry2D { texture: tex, error: None })
    }
}

impl From<mcom::Rc<IDirect3DDevice9>> for BasicTextureCache {
    fn from(device: mcom::Rc<IDirect3DDevice9>) -> Self { Self::new(device) }
}



type BoxError = Box<dyn std::error::Error>;

struct Entry2D {
    pub texture:    mcom::Rc<IDirect3DTexture9>,
    pub error:      Option<BoxError>,
}

struct Dynamic<C> {
    common:         C,
    #[allow(dead_code)] // XXX
    last_mod_time:  SystemTime,
}

fn create_texture_rgba_1x1(device: &mcom::Rc<IDirect3DDevice9>, rgba: u32) -> Result<mcom::Rc<IDirect3DTexture9>, Error> {
    let [r, g, b, a] = rgba.to_le_bytes();
    let argb = u32::from_le_bytes([a, r, g, b]);

    let mut tex = null_mut();
    let hr = unsafe { device.CreateTexture(1, 1, 1, D3DUSAGE_DYNAMIC, D3DFMT_A8R8G8B8, D3DPOOL_DEFAULT, &mut tex, null_mut()) };
    Error::check_hr("IDirect3DDevice9::CreateTexture", hr, "")?;
    let tex = unsafe { mcom::Rc::from_raw(tex) };

    let mut lock = unsafe { std::mem::zeroed() };
    let hr = unsafe { tex.LockRect(0, &mut lock, null(), D3DLOCK_DISCARD) };
    Error::check_hr("IDirect3DTexture9::LockRect", hr, "")?;

    unsafe { std::ptr::write_unaligned(lock.pBits.cast(), argb) };

    let hr = unsafe { tex.UnlockRect(0) };
    Error::check_hr("IDirect3DTexture9::UnlockRect", hr, "")?;

    Ok(tex)
}


const D3DERR_NOTFOUND : HRESULT = MAKE_D3DHRESULT(2150);

const _FACD3D : u16 = 0x876;
#[allow(non_snake_case)] const fn MAKE_D3DHRESULT(code: u16) -> HRESULT { error::MAKE_HRESULT(1, _FACD3D, code) }
//#[allow(non_snake_case)] const fn MAKE_D3DSTATUS (code: u16) -> HRESULT { error::MAKE_HRESULT(0, _FACD3D, code) }
