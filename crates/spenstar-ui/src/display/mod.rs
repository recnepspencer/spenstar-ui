//! Semantic labels, avatars, metrics, and structured text.
mod avatar;
mod badge;
mod code;
mod metric;

pub use avatar::{Avatar, AvatarGroup, AvatarStatus};
pub use badge::{Badge, BadgeSize, BadgeVariant, Chip, ChipResponse};
pub use code::{CodeBlock, DescriptionItem, DescriptionList, DiffKind, DiffLine, DiffView};
pub use metric::{KpiCard, KpiDetail};

pub use crate::controls::Tone;
