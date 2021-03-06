use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use rect::Rect;
use get_error;
use std::ptr;
use libc::c_int;
use num::FromPrimitive;
use pixels;
use render::BlendMode;
use rwops::RWops;

use sys::surface as ll;

pub struct Surface<'a> {
    raw: *mut ll::SDL_Surface,
    _marker: PhantomData<&'a ()>
}

impl<'a> Drop for Surface<'a> {
    #[inline]
    fn drop(&mut self) {
        unsafe { ll::SDL_FreeSurface(self.raw); }
    }
}

/// An unsized Surface reference.
///
/// This type is used whenever Surfaces need to be borrowed from the SDL library, without concern
/// for freeing the Surface.
pub struct SurfaceRef {
    // It's nothing! (it gets transmuted to SDL_Surface later).
    // The empty private field is need to a) make `std::mem::swap()` copy nothing instead of
    // clobbering two surfaces (SDL_Surface's size could change in the future),
    // and b) prevent user initialization of this type.
    _raw: ()
}

#[test]
fn test_surface_ref_size() {
    // `SurfaceRef` must be 0 bytes.
    assert_eq!(::std::mem::size_of::<SurfaceRef>(), 0);
}

impl<'a> Deref for Surface<'a> {
    type Target = SurfaceRef;

    #[inline]
    fn deref(&self) -> &SurfaceRef {
        unsafe { mem::transmute(self.raw) }
    }
}

impl<'a> DerefMut for Surface<'a> {
    #[inline]
    fn deref_mut(&mut self) -> &mut SurfaceRef {
        unsafe { mem::transmute(self.raw) }
    }
}

impl<'a> AsRef<SurfaceRef> for Surface<'a> {
    #[inline]
    fn as_ref(&self) -> &SurfaceRef {
        unsafe { mem::transmute(self.raw) }
    }
}

impl<'a> AsMut<SurfaceRef> for Surface<'a> {
    #[inline]
    fn as_mut(&mut self) -> &mut SurfaceRef {
        unsafe { mem::transmute(self.raw) }
    }
}


impl<'a> Surface<'a> {
    pub unsafe fn from_ll<'b>(raw: *mut ll::SDL_Surface) -> Surface<'b> {
        Surface {
            raw: raw,
            _marker: PhantomData
        }
    }

    /// Creates a new surface using a pixel format.
    ///
    /// # Example
    /// ```no_run
    /// use sdl2::pixels::PixelFormatEnum;
    /// use sdl2::surface::Surface;
    ///
    /// let surface = Surface::new(512, 512, PixelFormatEnum::RGB24).unwrap();
    /// ```
    pub fn new(width: u32, height: u32, format: pixels::PixelFormatEnum) -> Result<Surface<'static>, String> {
        let masks = try!(format.into_masks());
        Surface::from_pixelmasks(width, height, masks)
    }

    /// Creates a new surface using pixel masks.
    ///
    /// # Example
    /// ```no_run
    /// use sdl2::pixels::PixelFormatEnum;
    /// use sdl2::surface::Surface;
    ///
    /// let masks = PixelFormatEnum::RGB24.into_masks().unwrap();
    /// let surface = Surface::from_pixelmasks(512, 512, masks).unwrap();
    /// ```
    pub fn from_pixelmasks(width: u32, height: u32, masks: pixels::PixelMasks) -> Result<Surface<'static>, String> {
        unsafe {
            if width >= (1<<31) || height >= (1<<31) {
                Err("Image is too large.".to_owned())
            } else {
                let raw = ll::SDL_CreateRGBSurface(0, width as c_int, height as c_int,
                    masks.bpp as c_int, masks.rmask, masks.gmask, masks.bmask, masks.amask);

                if raw.is_null() {
                    Err(get_error())
                } else {
                    Ok(Surface {
                        raw: raw,
                        _marker: PhantomData
                    })
                }
            }
        }
    }

    /// Creates a new surface from an existing buffer, using a pixel format.
    pub fn from_data(data: &'a mut [u8], width: u32, height: u32, pitch: u32, format: pixels::PixelFormatEnum) -> Result<Surface<'a>, String> {
        let masks = try!(format.into_masks());
        Surface::from_data_pixelmasks(data, width, height, pitch, masks)
    }

    /// Creates a new surface from an existing buffer, using pixel masks.
    pub fn from_data_pixelmasks(data: &'a mut [u8], width: u32, height: u32, pitch: u32, masks: pixels::PixelMasks) -> Result<Surface<'a>, String> {
        unsafe {
            if width >= (1<<31) || height >= (1<<31) {
                Err("Image is too large.".to_owned())
            } else if pitch >= (1<<31) {
                Err("Pitch is too large.".to_owned())
            } else {
                let raw = ll::SDL_CreateRGBSurfaceFrom(
                    data.as_mut_ptr() as *mut _, width as c_int, height as c_int,
                    masks.bpp as c_int, pitch as c_int, masks.rmask, masks.gmask, masks.bmask, masks.amask);

                if raw.is_null() {
                    Err(get_error())
                } else {
                    Ok(Surface {
                        raw: raw,
                        _marker: PhantomData
                    })
                }
            }
        }
    }

    pub fn load_bmp_rw(rwops: &mut RWops) -> Result<Surface<'static>, String> {
        let raw = unsafe {
            ll::SDL_LoadBMP_RW(rwops.raw(), 0)
        };

        if raw.is_null() {
            Err(get_error())
        } else {
            Ok(Surface {
                raw: raw,
                _marker: PhantomData
            })
        }
    }

    pub fn load_bmp<P: AsRef<Path>>(path: P) -> Result<Surface<'static>, String> {
        let mut file = try!(RWops::from_file(path, "rb"));
        Surface::load_bmp_rw(&mut file)
    }
}

impl SurfaceRef {
    #[inline]
    pub unsafe fn from_ll<'a>(raw: *mut ll::SDL_Surface) -> &'a SurfaceRef {
        mem::transmute(raw)
    }

    #[inline]
    pub unsafe fn from_ll_mut<'a>(raw: *mut ll::SDL_Surface) -> &'a mut SurfaceRef {
        mem::transmute(raw)
    }

    #[inline]
    pub fn raw(&self) -> *mut ll::SDL_Surface {
        unsafe { mem::transmute(self) }
    }

    #[inline]
    fn raw_ref(&self) -> &ll::SDL_Surface {
        unsafe { mem::transmute(self) }
    }

    pub fn width(&self) -> u32 {
        self.raw_ref().w as u32
    }

    pub fn height(&self) -> u32 {
        self.raw_ref().h as u32
    }

    pub fn pitch(&self) -> u32 {
        self.raw_ref().pitch as u32
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width(), self.height())
    }

    pub fn rect(&self) -> Rect {
        Rect::new(0, 0, self.width(), self.height())
    }

    pub fn pixel_format(&self) -> pixels::PixelFormat {
        unsafe {
            pixels::PixelFormat::from_ll(self.raw_ref().format)
        }
    }

    /// Locks a surface so that the pixels can be directly accessed safely.
    pub fn with_lock<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
        unsafe {
            if ll::SDL_LockSurface(self.raw()) != 0 { panic!("could not lock surface"); }

            let raw_pixels = self.raw_ref().pixels as *const _;
            let len = self.raw_ref().pitch as usize * (self.raw_ref().h as usize);
            let pixels = ::std::slice::from_raw_parts(raw_pixels, len);
            let rv = f(pixels);
            ll::SDL_UnlockSurface(self.raw());
            rv
        }
    }

    /// Locks a surface so that the pixels can be directly accessed safely.
    pub fn with_lock_mut<R, F: FnOnce(&mut [u8]) -> R>(&mut self, f: F) -> R {
        unsafe {
            if ll::SDL_LockSurface(self.raw()) != 0 { panic!("could not lock surface"); }

            let raw_pixels = self.raw_ref().pixels as *mut _;
            let len = self.raw_ref().pitch as usize * (self.raw_ref().h as usize);
            let pixels = ::std::slice::from_raw_parts_mut(raw_pixels, len);
            let rv = f(pixels);
            ll::SDL_UnlockSurface(self.raw());
            rv
        }
    }

    /// Returns the Surface's pixel buffer if the Surface doesn't require locking
    /// (e.g. it's a software surface).
    pub fn without_lock(&self) -> Option<&[u8]> {
        if self.must_lock() {
            None
        } else {
            unsafe {
                let raw_pixels = self.raw_ref().pixels as *const _;
                let len = self.raw_ref().pitch as usize * (self.raw_ref().h as usize);

                Some(::std::slice::from_raw_parts(raw_pixels, len))
            }
        }
    }

    /// Returns the Surface's pixel buffer if the Surface doesn't require locking
    /// (e.g. it's a software surface).
    pub fn without_lock_mut(&mut self) -> Option<&mut [u8]> {
        if self.must_lock() {
            None
        } else {
            unsafe {
                let raw_pixels = self.raw_ref().pixels as *mut _;
                let len = self.raw_ref().pitch as usize * (self.raw_ref().h as usize);

                Some(::std::slice::from_raw_parts_mut(raw_pixels, len))
            }
        }
    }

    /// Returns true if the Surface needs to be locked before accessing the Surface pixels.
    pub fn must_lock(&self) -> bool {
        // Implements the SDL_MUSTLOCK macro.
        (self.raw_ref().flags & ll::SDL_RLEACCEL) != 0
    }

    pub fn save_bmp_rw(&self, rwops: &mut RWops) -> Result<(), String> {
        let ret = unsafe {
            ll::SDL_SaveBMP_RW(self.raw(), rwops.raw(), 0)
        };
        if ret == 0 { Ok(()) }
        else { Err(get_error()) }
    }

    pub fn save_bmp<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let mut file = try!(RWops::from_file(path, "wb"));
        self.save_bmp_rw(&mut file)
    }

    pub fn set_palette(&mut self, palette: &pixels::Palette) -> Result<(), String> {
        let result = unsafe { ll::SDL_SetSurfacePalette(self.raw(), palette.raw()) };

        match result {
            0 => Ok(()),
            _ => Err(get_error())
        }
    }

    #[allow(non_snake_case)]
    pub fn enable_RLE(&mut self) {
        let result = unsafe { ll::SDL_SetSurfaceRLE(self.raw(), 1) };

        if result != 0 {
            // Should only panic on a null Surface
            panic!(get_error());
        }
    }

    #[allow(non_snake_case)]
    pub fn disable_RLE(&mut self) {
        let result = unsafe { ll::SDL_SetSurfaceRLE(self.raw(), 0) };

        if result != 0 {
            // Should only panic on a null Surface
            panic!(get_error());
        }
    }

    pub fn set_color_key(&mut self, enable: bool, color: pixels::Color) -> Result<(), String> {
        let key = color.to_u32(&self.pixel_format());
        let result = unsafe {
            ll::SDL_SetColorKey(self.raw(), if enable { 1 } else { 0 }, key)
        };
        if result == 0 {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    /// The function will fail if the surface doesn't have color key enabled.
    pub fn color_key(&self) -> Result<pixels::Color, String> {
        let mut key = 0;

        // SDL_GetColorKey does not mutate, but requires a non-const pointer anyway.

        let result = unsafe {
            ll::SDL_GetColorKey(self.raw(), &mut key)
        };

        if result == 0 {
            Ok(pixels::Color::from_u32(&self.pixel_format(), key))
        } else {
            Err(get_error())
        }
    }

    pub fn set_color_mod(&mut self, color: pixels::Color) {
        let (r, g, b) = match color {
            pixels::Color::RGB(r, g, b) => (r, g, b),
            pixels::Color::RGBA(r, g, b, _) => (r, g, b)
        };

        let result = unsafe { ll::SDL_SetSurfaceColorMod(self.raw(), r, g, b) };

        if result != 0 {
            // Should only fail on a null Surface
            panic!(get_error());
        }
    }

    pub fn color_mod(&self) -> pixels::Color {
        let mut r = 0;
        let mut g = 0;
        let mut b = 0;

        // SDL_GetSurfaceColorMod does not mutate, but requires a non-const pointer anyway.

        let result = unsafe {
            ll::SDL_GetSurfaceColorMod(self.raw(), &mut r, &mut g, &mut b) == 0
        };

        if result {
            pixels::Color::RGB(r, g, b)
        } else {
            // Should only fail on a null Surface
            panic!(get_error())
        }
    }

    pub fn fill_rect(&mut self, rect: Option<Rect>, color: pixels::Color) -> Result<(), String> {
        unsafe {
            let rect_ptr = mem::transmute( rect.as_ref() );
            let format = self.pixel_format();
            let result = ll::SDL_FillRect(self.raw(), rect_ptr, color.to_u32(&format) );
            match result {
                0 => Ok(()),
                _ => Err(get_error())
            }
        }
    }

    pub fn fill_rects(&mut self, rects: &[Option<Rect>], color: pixels::Color) -> Result<(), String> {
        for rect in rects.iter() {
            let result = self.fill_rect(rect.clone(), color);
            match result {
                Err(e) => return Err(e),
                _ => ()
            };
        }

        Ok(())
    }

    pub fn set_alpha_mod(&mut self, alpha: u8) {
        let result = unsafe {
            ll::SDL_SetSurfaceAlphaMod(self.raw(), alpha)
        };

        if result != 0 {
            // Should only fail on a null Surface
            panic!(get_error());
        }
    }

    pub fn alpha_mod(&self) -> u8 {
        let mut alpha = 0;
        let result = unsafe {
            ll::SDL_GetSurfaceAlphaMod(self.raw(), &mut alpha)
        };

        match result {
            0 => alpha,
            // Should only fail on a null Surface
            _ => panic!(get_error())
        }
    }

    /// The function will fail if the blend mode is not supported by SDL.
    pub fn set_blend_mode(&mut self, mode: BlendMode) -> Result<(), String> {
        let result = unsafe {
            ll::SDL_SetSurfaceBlendMode(self.raw(), mode as c_int)
        };

        match result {
            0 => Ok(()),
            _ => Err(get_error())
        }
    }

    pub fn blend_mode(&self) -> BlendMode {
        let mut mode: ll::SDL_BlendMode = 0;
        let result = unsafe {
            ll::SDL_GetSurfaceBlendMode(self.raw(), &mut mode)
        };

        match result {
            0 => FromPrimitive::from_i32(mode as i32).unwrap(),
            // Should only fail on a null Surface
            _ => panic!(get_error())
        }
    }

    /// Sets the clip rectangle for the surface.
    ///
    /// If the rectangle is `None`, clipping will be disabled.
    pub fn set_clip_rect(&mut self, rect: Option<Rect>) -> bool {
        unsafe {
            ll::SDL_SetClipRect(self.raw(), match rect {
                Some(rect) => rect.raw(),
                None => ptr::null()
            }) == 1
        }
    }

    /// Gets the clip rectangle for the surface.
    ///
    /// Returns `None` if clipping is disabled.
    pub fn clip_rect(&self) -> Option<Rect> {
        let mut raw = unsafe { mem::uninitialized() };
        unsafe {
            ll::SDL_GetClipRect(self.raw(), &mut raw)
        };
        if raw.w == 0 || raw.h == 0 {
            None
        } else {
            Some(Rect::from_ll(raw))
        }
    }

    /// Copies the surface into a new one that is optimized for blitting to a surface of a specified pixel format.
    pub fn convert(&self, format: &pixels::PixelFormat) -> Result<Surface<'static>, String> {
        // SDL_ConvertSurface takes a flag as the last parameter, which should be 0 by the docs.
        let surface_ptr = unsafe { ll::SDL_ConvertSurface(self.raw(), format.raw(), 0u32) };

        if surface_ptr.is_null() {
            Err(get_error())
        } else {
            unsafe { Ok(Surface::from_ll(surface_ptr)) }
        }
    }

    // Note: There's no need to implement SDL_ConvertSurfaceFormat, as it 
    // does the same thing as SDL_ConvertSurface but with a slightly different
    // function signature.

    /// Performs surface blitting (surface copying).
    ///
    /// Returns the final blit rectangle, if a `dst_rect` was provided.
    pub fn blit(&self, src_rect: Option<Rect>, 
            dst: &mut SurfaceRef, dst_rect: Option<Rect>) 
            -> Result<Option<Rect>, String> {
        unsafe {
            let src_rect_ptr = src_rect.as_ref().map(|r| r.raw()).unwrap_or(ptr::null());

            // Copy the rect here to make a mutable copy without requiring
            // a mutable argument
            let mut dst_rect = dst_rect;
            let dst_rect_ptr = dst_rect.as_mut().map(|mut r| r.raw_mut())
                .unwrap_or(ptr::null_mut());
            let result = ll::SDL_UpperBlit(
                self.raw(), src_rect_ptr, dst.raw(), dst_rect_ptr
            );

            if result == 0 {
                Ok(dst_rect)
            } else {
                Err(get_error())
            }
        }
    }

    /// Performs low-level surface blitting.
    ///
    /// Unless you know what you're doing, use `blit()` instead, which will clip the input rectangles.
    /// This function could crash if the rectangles aren't pre-clipped to the surface, and is therefore unsafe.
    pub unsafe fn lower_blit(&self, src_rect: Option<Rect>,
                      dst: &mut SurfaceRef, dst_rect: Option<Rect>) -> Result<(), String> {

        match {
            // The rectangles don't change, but the function requires mutable pointers.
            let src_rect_ptr = src_rect.as_ref().map(|r| r.raw())
                .unwrap_or(ptr::null()) as *mut _;
            let dst_rect_ptr = dst_rect.as_ref().map(|r| r.raw())
                .unwrap_or(ptr::null()) as *mut _;
            ll::SDL_LowerBlit(self.raw(), src_rect_ptr, dst.raw(), dst_rect_ptr)
        } {
            0 => Ok(()),
            _ => Err(get_error())
        }
    }

    /// Performs scaled surface bliting (surface copying).
    ///
    /// Returns the final blit rectangle, if a `dst_rect` was provided.
    pub fn blit_scaled(&self, src_rect: Option<Rect>,
                             dst: &mut SurfaceRef, dst_rect: Option<Rect>) -> Result<Option<Rect>, String> {

        match unsafe {
            let src_rect_ptr = src_rect.as_ref().map(|r| r.raw()).unwrap_or(ptr::null());

            // Copy the rect here to make a mutable copy without requiring
            // a mutable argument
            let mut dst_rect = dst_rect;
            let dst_rect_ptr = dst_rect.as_mut().map(|mut r| r.raw_mut())
                .unwrap_or(ptr::null_mut());
            ll::SDL_UpperBlitScaled(self.raw(), src_rect_ptr, dst.raw(), dst_rect_ptr)
        } {
            0 => Ok(dst_rect),
            _ => Err(get_error())
        }
    }

    /// Performs low-level scaled surface blitting.
    ///
    /// Unless you know what you're doing, use `blit_scaled()` instead, which will clip the input rectangles.
    /// This function could crash if the rectangles aren't pre-clipped to the surface, and is therefore unsafe.
    pub unsafe fn lower_blit_scaled(&self, src_rect: Option<Rect>,
                             dst: &mut SurfaceRef, dst_rect: Option<Rect>) -> Result<(), String> {

        match {
            // The rectangles don't change, but the function requires mutable pointers.
            let src_rect_ptr = src_rect.as_ref().map(|r| r.raw())
                .unwrap_or(ptr::null()) as *mut _;
            let dst_rect_ptr = dst_rect.as_ref().map(|r| r.raw())
                .unwrap_or(ptr::null()) as *mut _;
            ll::SDL_LowerBlitScaled(self.raw(), src_rect_ptr, dst.raw(), dst_rect_ptr)
        } {
            0 => Ok(()),
            _ => Err(get_error())
        }
    }

    /*
    pub fn SDL_ConvertPixels(width: c_int, height: c_int, src_format: uint32_t, src: *c_void, src_pitch: c_int, dst_format: uint32_t, dst: *c_void, dst_pitch: c_int) -> c_int;
    */
}
