use egui::{Color32, Response, Ui};

use super::{IconSource, SvgIcon};

macro_rules! lucide {
    ($($variant:ident => $name:literal),+ $(,)?) => {
        /// Curated Lucide icon set.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum Icon {
            $(
                #[doc = concat!("Lucide `", $name, "`.")]
                $variant,
            )+
        }

        impl Icon {
            /// All icons, in alphabetical order by SVG name.
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];

            /// Upstream icon name.
            pub const fn name(self) -> &'static str {
                match self { $(Self::$variant => $name),+ }
            }

            /// Borrow the original, unmodified SVG.
            pub const fn svg(self) -> SvgIcon {
                match self {
                    $(Self::$variant => SvgIcon::from_static(
                        $name, include_str!(concat!("../../assets/icons/", $name, ".svg")),
                    )),+
                }
            }

            /// Display the icon at the supplied logical size.
            pub fn show(self, ui: &mut Ui, size: f32, tint: Color32) -> Response {
                IconSource::from(self).show(ui, size, tint)
            }
        }
    };
}

/// License and attribution for the embedded Lucide assets.
pub const LUCIDE_LICENSE: &str = include_str!("../../assets/icons/LICENSE-LUCIDE");

lucide! {
    Activity => "activity",
    AlertCircle => "alert-circle",
    AlertTriangle => "alert-triangle",
    ArrowDown => "arrow-down",
    ArrowLeft => "arrow-left",
    ArrowRight => "arrow-right",
    ArrowUpDown => "arrow-up-down",
    ArrowUpRight => "arrow-up-right",
    ArrowUp => "arrow-up",
    BarChart3 => "bar-chart-3",
    Bell => "bell",
    Bookmark => "bookmark",
    Boxes => "boxes",
    Building2 => "building-2",
    Calendar => "calendar",
    CheckCircle2 => "check-circle-2",
    Check => "check",
    ChevronDown => "chevron-down",
    ChevronLeft => "chevron-left",
    ChevronRight => "chevron-right",
    ChevronUp => "chevron-up",
    ChevronsUpDown => "chevrons-up-down",
    ClipboardList => "clipboard-list",
    Clock => "clock",
    Code2 => "code-2",
    Command => "command",
    Compass => "compass",
    Copy => "copy",
    Cpu => "cpu",
    Database => "database",
    Download => "download",
    Edit => "edit",
    ExternalLink => "external-link",
    EyeOff => "eye-off",
    Eye => "eye",
    FileText => "file-text",
    File => "file",
    FilterX => "filter-x",
    Filter => "filter",
    Flame => "flame",
    FolderOpen => "folder-open",
    Folder => "folder",
    GitBranch => "git-branch",
    GitCommit => "git-commit",
    GitMerge => "git-merge",
    GitPullRequest => "git-pull-request",
    Globe => "globe",
    Grid => "grid",
    Heart => "heart",
    HelpCircle => "help-circle",
    Home => "home",
    Image => "image",
    Inbox => "inbox",
    Info => "info",
    Kanban => "kanban",
    Layers => "layers",
    LayoutDashboard => "layout-dashboard",
    LineChart => "line-chart",
    Link => "link",
    List => "list",
    LoaderCircle => "loader-circle",
    Lock => "lock",
    LogOut => "log-out",
    Mail => "mail",
    Menu => "menu",
    Minus => "minus",
    Moon => "moon",
    MoreHorizontal => "more-horizontal",
    MoreVertical => "more-vertical",
    Package => "package",
    Palette => "palette",
    PanelLeftClose => "panel-left-close",
    PanelLeftOpen => "panel-left-open",
    PanelLeft => "panel-left",
    Pencil => "pencil",
    PieChart => "pie-chart",
    Plus => "plus",
    Redo2 => "redo-2",
    RefreshCw => "refresh-cw",
    RotateCcw => "rotate-ccw",
    RotateCw => "rotate-cw",
    Save => "save",
    Search => "search",
    Send => "send",
    Settings => "settings",
    Share2 => "share-2",
    ShieldAlert => "shield-alert",
    ShieldCheck => "shield-check",
    Shield => "shield",
    Sidebar => "sidebar",
    SlidersHorizontal => "sliders-horizontal",
    Sparkles => "sparkles",
    Star => "star",
    Sun => "sun",
    Table => "table",
    Tag => "tag",
    Terminal => "terminal",
    Trash2 => "trash-2",
    Truck => "truck",
    Undo2 => "undo-2",
    Unlock => "unlock",
    Upload => "upload",
    UserCheck => "user-check",
    User => "user",
    Users => "users",
    Warehouse => "warehouse",
    XCircle => "x-circle",
    X => "x",
    Zap => "zap",
}
