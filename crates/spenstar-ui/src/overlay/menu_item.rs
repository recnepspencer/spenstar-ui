use crate::icon::IconSource;

/// An entry in a dropdown or context menu.
#[derive(Clone)]
pub enum MenuItem<'a> {
    /// Command, toggle, or parent of a submenu.
    Action {
        /// Value returned on selection.
        id: &'a str,
        /// Visible label.
        label: &'a str,
        /// Optional leading built-in or custom SVG icon.
        icon: Option<IconSource>,
        /// Optional trailing shortcut hint.
        shortcut: Option<&'a str>,
        /// Checked state for toggles.
        checked: Option<bool>,
        /// Whether the action can be activated.
        enabled: bool,
        /// Whether to show the destructive treatment.
        danger: bool,
        /// Child entries. A non-empty slice makes this action a submenu trigger.
        children: &'a [MenuItem<'a>],
    },
    /// Muted section label.
    Heading(&'a str),
    /// Hairline rule.
    Separator,
}

impl<'a> MenuItem<'a> {
    /// Build an enabled command.
    pub fn action(id: &'a str, label: &'a str) -> Self {
        Self::Action {
            id,
            label,
            icon: None,
            shortcut: None,
            checked: None,
            enabled: true,
            danger: false,
            children: &[],
        }
    }

    /// Build a nested menu trigger.
    pub fn submenu(label: &'a str, children: &'a [Self]) -> Self {
        Self::Action {
            id: label,
            label,
            icon: None,
            shortcut: None,
            checked: None,
            enabled: true,
            danger: false,
            children,
        }
    }

    /// Show an icon before the label.
    pub fn leading(mut self, leading: impl Into<IconSource>) -> Self {
        if let Self::Action { icon, .. } = &mut self {
            *icon = Some(leading.into());
        }
        self
    }

    /// Show a shortcut hint. Shortcut registration remains caller-owned.
    pub fn shortcut(mut self, text: &'a str) -> Self {
        if let Self::Action { shortcut, .. } = &mut self {
            *shortcut = Some(text);
        }
        self
    }

    /// Render a check marker for a selected toggle.
    pub fn checked(mut self, value: bool) -> Self {
        if let Self::Action { checked, .. } = &mut self {
            *checked = Some(value);
        }
        self
    }

    /// Use the danger treatment.
    pub fn destructive(mut self) -> Self {
        if let Self::Action { danger, .. } = &mut self {
            *danger = true;
        }
        self
    }

    /// Set whether keyboard and pointer activation are available.
    pub fn enabled(mut self, value: bool) -> Self {
        if let Self::Action { enabled, .. } = &mut self {
            *enabled = value;
        }
        self
    }

    pub(super) fn selectable(&self) -> bool {
        matches!(self, Self::Action { enabled: true, .. })
    }
}
