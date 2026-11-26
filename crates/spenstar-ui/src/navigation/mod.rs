//! Navigation strips, disclosure, and page controls.
mod disclosure;
mod pagination;
mod tabs;
mod trail;

pub use disclosure::{Accordion, AccordionMode, Disclosure, DisclosureVariant};
pub use pagination::{PageSlot, Pagination, page_slots};
pub use tabs::{Tab, TabSize, Tabs, TabsVariant};
pub use trail::{BreadcrumbSeparator, Breadcrumbs};
