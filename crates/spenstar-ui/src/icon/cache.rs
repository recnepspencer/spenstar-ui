use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use egui::{Color32, Context, Id, TextureHandle, TextureOptions};

use super::{SvgIcon, SvgIconError};

/// Texture cache residency for an egui context.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IconCacheStats {
    /// Resident texture entries.
    pub textures: usize,
    /// Resident RGBA texture bytes.
    pub bytes: usize,
}

#[derive(Default)]
struct Cache {
    // Least recently used at the front, most recently used at the back.
    entries: VecDeque<Entry>,
    bytes: usize,
}

struct Entry {
    svg: SvgIcon,
    px: u32,
    tint: Color32,
    texture: TextureHandle,
}

impl Entry {
    fn bytes(&self) -> usize {
        (self.px as usize).pow(2) * 4
    }
}

type SharedCache = Arc<Mutex<Cache>>;

/// Context-local texture lookup and rasterization.
pub struct SvgIconEngine;

impl SvgIconEngine {
    /// Maximum resident texture count.
    pub const MAX_TEXTURES: usize = 192;
    /// Maximum resident RGBA texture bytes.
    pub const MAX_TEXTURE_BYTES: usize = 8 * 1024 * 1024;

    /// Get a cached texture at the current viewport's physical scale.
    ///
    /// The cache bounds apply to its own texture handles. Callers can keep
    /// returned handles alive independently.
    pub fn texture(
        ctx: &Context,
        icon: &SvgIcon,
        size: f32,
        tint: Color32,
    ) -> Result<TextureHandle, SvgIconError> {
        let px = (size * ctx.pixels_per_point()).ceil();
        let limit = ctx
            .input(|input| input.max_texture_side)
            .min(SvgIcon::MAX_PHYSICAL_PX as usize);
        if !px.is_finite() || px < 1. || px > limit as f32 {
            return Err(SvgIconError::InvalidSize);
        }
        let px = px as u32;
        let cache = Self::cache(ctx);
        let mut cache = cache
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(index) = cache
            .entries
            .iter()
            .position(|entry| entry.px == px && entry.tint == tint && entry.svg == *icon)
            && let Some(entry) = cache.entries.remove(index)
        {
            let texture = entry.texture.clone();
            cache.entries.push_back(entry);
            return Ok(texture);
        }

        let image = icon.rasterize(px, tint)?;
        let bytes = (px as usize).pow(2) * 4;
        while cache.entries.len() >= Self::MAX_TEXTURES
            || cache.bytes + bytes > Self::MAX_TEXTURE_BYTES
        {
            let Some(entry) = cache.entries.pop_front() else {
                break;
            };
            cache.bytes -= entry.bytes();
        }
        let texture = ctx.load_texture(icon.id(), image, TextureOptions::LINEAR);
        cache.bytes += bytes;
        cache.entries.push_back(Entry {
            svg: icon.clone(),
            px,
            tint,
            texture: texture.clone(),
        });
        Ok(texture)
    }

    /// Inspect current cache residency.
    pub fn stats(ctx: &Context) -> IconCacheStats {
        let cache = Self::cache(ctx);
        let cache = cache
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        IconCacheStats {
            textures: cache.entries.len(),
            bytes: cache.bytes,
        }
    }

    fn cache(ctx: &Context) -> SharedCache {
        ctx.data_mut(|data| {
            data.get_temp_mut_or_default::<SharedCache>(Id::unique("spenstar-ui/svg-cache"))
                .clone()
        })
    }
}
