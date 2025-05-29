use std::cell::RefCell;
use std::cell::RefMut;
use std::collections::hash_map::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;

use fltk::app::App;
use fltk::browser::*;
use fltk::button::*;
use fltk::group::*;
use fltk::image::IcoImage;
use fltk::input::*;
use fltk::menu::*;
use fltk::prelude::*;
use fltk::table::*;
use fltk::terminal::*;
use fltk::text::*;
use fltk::window::Window;

use fltk_theme::ColorTheme;
use fltk_theme::WidgetScheme;

use crate::constants::ColorScheme;
use crate::constants::Style;
use crate::error::GuiError;

#[macro_export]
/// Generates the `WidgetName` enum and `From<WidgetName> for String` impl
macro_rules! impl_widget_name_enum {
    ($($name_variant:ident,)+) => {
        #[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
        pub enum WidgetName {
            $($name_variant),+
        }

        impl From<WidgetName> for String {
            fn from(value: WidgetName) -> Self {
                match value {
                    $(WidgetName::$name_variant => stringify!($name_variant).to_string()),+
                }
            }
        }
    };
}

/// Generates the `From<Rc<RefCell<$ty>>> for AnyWidget` impl
macro_rules! impl_from_widget {
    ($ty:ty => $variant:ident) => {
        impl From<Rc<RefCell<$ty>>> for AnyWidget {
            fn from(x: Rc<RefCell<$ty>>) -> Self {
                AnyWidget::$variant(x)
            }
        }
    };
}

pub trait GothicOrganizerWindow {
    type Message: Clone + Send + Sync + 'static;
    type Task: Clone + Send + Sync + 'static;
    type WidgetName: Clone + Send + Sync + Eq + Into<String> + std::hash::Hash + 'static;

    /// Should return a reference to the `ApplicationSettings` used
    /// to build this window.
    fn settings(&self) -> ApplicationSettings;

    /// Should return a reference to the container used to store widgets used by this window.
    fn widgets(&self) -> &HashMap<Self::WidgetName, AnyWidget>;

    /// Should return a mutable reference to the container used to store widgets used by this window.
    fn widgets_mut(&mut self) -> &mut HashMap<Self::WidgetName, AnyWidget>;

    /// Constructs the `Window` (with size, pos, icon, etc).
    fn window(settings: &ApplicationSettings) -> Window {
        let mut wnd = Window::default()
            .with_size(settings.width, settings.height)
            .with_pos(settings.x, settings.y)
            .with_label(&settings.title);

        if settings.centered {
            wnd = wnd.center_screen();
        }

        if settings.resizable {
            wnd.make_resizable(true);
        }

        if let Some(icon) = &settings.icon {
            wnd.set_icon(IcoImage::load(icon).ok())
        }

        wnd
    }

    /// Construct the `App` and apply theming.
    fn app(settings: &ApplicationSettings) -> App {
        let app = App::default();
        WidgetScheme::new(settings.style.into()).apply();
        ColorTheme::new(settings.colors.into()).apply();
        app
    }

    /// Constructs the UI for this window.
    /// All widgets need to be defined in this method. Dedicated `add_widget` method can be used to
    /// do this.
    fn populate_ui(&mut self, sender: fltk::app::Sender<Self::Message>, layout: &mut AnyGroup) -> Result<(), GuiError>;

    /// Here the concrete window should match `msg` and mutate itself,
    /// potentially sending back a `GuiError` or a `Task` to be handled by `event_loop`.
    fn handle_message(&mut self, msg: Self::Message) -> Result<Self::Task, GuiError>;

    /// This method defines how `Task` returned by `handle_message` is handled during the event
    /// loop. Example implementation:
    /// ```ignore
    /// while app.wait() {
    ///     if let Some(msg) = r.recv() {
    ///         match self.handle_message(msg)? {
    ///             _ => (),
    ///         }
    ///     }
    /// }
    /// ```
    ///
    fn event_loop(
        &mut self,
        app: &mut App,
        window: &mut Window,
        receiver: fltk::app::Receiver<<Self as GothicOrganizerWindow>::Message>,
    ) -> Result<(), GuiError>;

    /// Main entry point. Populates the UI and starts the event loop. Overwrite for custom widget
    /// layout and window handling. By default utilizes `fltk::group::Grid` for widget layout.
    fn run(&mut self) -> Result<(), GuiError> {
        let settings = self.settings();
        let mut wnd = Self::window(&settings);
        let mut app = Self::app(&settings);
        let (s, r) = fltk::app::channel::<Self::Message>();

        wnd.begin();
        let mut grid = fltk::group::Grid::default_fill();
        grid.set_layout(10, 1);
        grid.set_margin(10, 20, 10, 10);
        grid.set_gap(20, 10);

        self.populate_ui(s, &mut Rc::new(RefCell::new(grid)).into())?;

        wnd.end();
        wnd.show();

        self.event_loop(&mut app, &mut wnd, r)?;

        Ok(())
    }

    /// Adds a widget to the dedicated container under `name` and returns a reference to it.
    fn add_widget<T>(&mut self, name: Self::WidgetName, widget: T) -> Rc<RefCell<T>>
    where
        T: WidgetExt + 'static,
        Rc<RefCell<T>>: Into<AnyWidget>,
    {
        let rc = Rc::new(RefCell::new(widget));
        self.widgets_mut().insert(name, rc.clone().into());
        rc
    }

    fn widget(&self, name: &Self::WidgetName) -> Result<AnyWidget, GuiError> {
        self.widgets()
            .get(name)
            .cloned()
            .ok_or(GuiError::WidgetNotFound(name.clone().into()))
    }

    /// Activates the widget under `name`.
    fn activate_widget(&mut self, name: &Self::WidgetName) -> Result<(), GuiError> {
        let query = self
            .widgets_mut()
            .get(name)
            .ok_or(GuiError::WidgetNotFound(name.clone().into()))?;

        let mut w = query.as_widget_ext_mut();
        w.activate();
        Ok(())
    }

    /// Deactivates the widget under `name`.
    fn deactivate_widget(&mut self, name: &Self::WidgetName) -> Result<(), GuiError> {
        let query = self
            .widgets_mut()
            .get(name)
            .ok_or(GuiError::WidgetNotFound(name.clone().into()))?;

        let mut w = query.as_widget_ext_mut();
        w.deactivate();
        Ok(())
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum AnyGroup {
    Flex(Rc<RefCell<Flex>>),
    Grid(Rc<RefCell<Grid>>),
}

impl From<Rc<RefCell<Flex>>> for AnyGroup {
    fn from(x: Rc<RefCell<Flex>>) -> Self {
        AnyGroup::Flex(x)
    }
}
impl From<Rc<RefCell<Grid>>> for AnyGroup {
    fn from(x: Rc<RefCell<Grid>>) -> Self {
        AnyGroup::Grid(x)
    }
}

impl AnyGroup {
    pub fn as_group_mut(&self) -> RefMut<dyn GroupExt> {
        match self {
            AnyGroup::Flex(x) => x.borrow_mut(),
            AnyGroup::Grid(x) => x.borrow_mut(),
        }
    }

    pub fn as_grid_mut(&self) -> Option<RefMut<Grid>> {
        if let AnyGroup::Grid(x) = self {
            Some(x.borrow_mut())
        } else {
            None
        }
    }

    pub fn as_flex_mut(&self) -> Option<RefMut<Flex>> {
        if let AnyGroup::Flex(x) = self {
            Some(x.borrow_mut())
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub enum AnyWidget {
    TextDisplay(Rc<RefCell<TextDisplay>>),
    TextEditor(Rc<RefCell<TextEditor>>),
    Terminal(Rc<RefCell<Terminal>>),
    Browser(Rc<RefCell<Browser>>),
    HoldBrowser(Rc<RefCell<HoldBrowser>>),
    FileBrowser(Rc<RefCell<FileBrowser>>),
    CheckBrowser(Rc<RefCell<CheckBrowser>>),
    MultiBrowser(Rc<RefCell<MultiBrowser>>),
    SelectBrowser(Rc<RefCell<SelectBrowser>>),
    Table(Rc<RefCell<Table>>),
    TableRow(Rc<RefCell<TableRow>>),
    Basic(Rc<RefCell<dyn WidgetExt>>),
    Valuator(Rc<RefCell<dyn ValuatorExt>>),
    Menu(Rc<RefCell<dyn MenuExt>>),
    Input(Rc<RefCell<dyn InputExt>>),
    Button(Rc<RefCell<dyn ButtonExt>>),
    Group(Rc<RefCell<dyn GroupExt>>),
    Window(Rc<RefCell<dyn WindowExt>>),
}

impl From<Rc<RefCell<dyn WidgetExt>>> for AnyWidget {
    fn from(x: Rc<RefCell<dyn WidgetExt>>) -> Self {
        AnyWidget::Basic(x)
    }
}

impl From<Rc<RefCell<dyn ValuatorExt>>> for AnyWidget {
    fn from(x: Rc<RefCell<dyn ValuatorExt>>) -> Self {
        AnyWidget::Valuator(x)
    }
}

impl From<Rc<RefCell<dyn MenuExt>>> for AnyWidget {
    fn from(x: Rc<RefCell<dyn MenuExt>>) -> Self {
        AnyWidget::Menu(x)
    }
}

impl From<Rc<RefCell<dyn InputExt>>> for AnyWidget {
    fn from(x: Rc<RefCell<dyn InputExt>>) -> Self {
        AnyWidget::Input(x)
    }
}

impl From<Rc<RefCell<dyn ButtonExt>>> for AnyWidget {
    fn from(x: Rc<RefCell<dyn ButtonExt>>) -> Self {
        AnyWidget::Button(x)
    }
}

impl From<Rc<RefCell<dyn GroupExt>>> for AnyWidget {
    fn from(x: Rc<RefCell<dyn GroupExt>>) -> Self {
        AnyWidget::Group(x)
    }
}

impl From<Rc<RefCell<dyn WindowExt>>> for AnyWidget {
    fn from(x: Rc<RefCell<dyn WindowExt>>) -> Self {
        AnyWidget::Window(x)
    }
}

impl_from_widget!(Browser      => Browser);
impl_from_widget!(HoldBrowser  => HoldBrowser);
impl_from_widget!(FileBrowser  => FileBrowser);
impl_from_widget!(CheckBrowser => CheckBrowser);
impl_from_widget!(MultiBrowser => MultiBrowser);
impl_from_widget!(SelectBrowser=> SelectBrowser);
impl_from_widget!(Table        => Table);
impl_from_widget!(TableRow     => TableRow);
impl_from_widget!(TextDisplay  => TextDisplay);
impl_from_widget!(TextEditor   => TextEditor);
impl_from_widget!(Terminal     => Terminal);
impl_from_widget!(Choice       => Menu);
impl_from_widget!(Button       => Button);
impl_from_widget!(Input        => Input);

impl AnyWidget {
    /// Get a `RefMut<dyn WidgetExt>` no matter which variant we are.
    pub fn as_widget_ext_mut(&self) -> RefMut<dyn WidgetExt> {
        match self {
            AnyWidget::Basic(w) => w.borrow_mut(),
            AnyWidget::Valuator(w) => RefMut::map(w.borrow_mut(), |v| v as &mut dyn WidgetExt),
            AnyWidget::Menu(w) => RefMut::map(w.borrow_mut(), |m| m as &mut dyn WidgetExt),
            AnyWidget::Input(w) => RefMut::map(w.borrow_mut(), |i| i as &mut dyn WidgetExt),
            AnyWidget::Button(w) => RefMut::map(w.borrow_mut(), |b| b as &mut dyn WidgetExt),
            AnyWidget::Group(w) => RefMut::map(w.borrow_mut(), |g| g as &mut dyn WidgetExt),
            AnyWidget::Window(w) => RefMut::map(w.borrow_mut(), |w| w as &mut dyn WidgetExt),

            // for all the concrete types that also impl WidgetExt:
            AnyWidget::Browser(w) => RefMut::map(w.borrow_mut(), |b| b as &mut dyn WidgetExt),
            AnyWidget::HoldBrowser(w) => RefMut::map(w.borrow_mut(), |b| b as &mut dyn WidgetExt),
            AnyWidget::FileBrowser(w) => RefMut::map(w.borrow_mut(), |b| b as &mut dyn WidgetExt),
            AnyWidget::CheckBrowser(w) => RefMut::map(w.borrow_mut(), |b| b as &mut dyn WidgetExt),
            AnyWidget::MultiBrowser(w) => RefMut::map(w.borrow_mut(), |b| b as &mut dyn WidgetExt),
            AnyWidget::SelectBrowser(w) => RefMut::map(w.borrow_mut(), |b| b as &mut dyn WidgetExt),

            AnyWidget::Table(w) => RefMut::map(w.borrow_mut(), |t| t as &mut dyn WidgetExt),
            AnyWidget::TableRow(w) => RefMut::map(w.borrow_mut(), |t| t as &mut dyn WidgetExt),
            AnyWidget::TextDisplay(w) => RefMut::map(w.borrow_mut(), |t| t as &mut dyn WidgetExt),
            AnyWidget::TextEditor(w) => RefMut::map(w.borrow_mut(), |t| t as &mut dyn WidgetExt),
            AnyWidget::Terminal(w) => RefMut::map(w.borrow_mut(), |t| t as &mut dyn WidgetExt),
        }
    }
}

#[derive(Debug, Default)]
pub struct ApplicationSettings {
    pub icon: Option<PathBuf>,
    pub title: String,
    pub width: i32,
    pub height: i32,
    pub centered: bool,
    pub x: i32,
    pub y: i32,
    pub resizable: bool,
    pub style: Style,
    pub colors: ColorScheme,
}

#[allow(dead_code)]
impl ApplicationSettings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_icon(mut self, icon: impl AsRef<Path>) -> Self {
        let path = icon.as_ref();

        self.icon = Some(path.into());
        self
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn with_width(mut self, width: i32) -> Self {
        self.width = width;
        self
    }

    pub fn with_height(mut self, height: i32) -> Self {
        self.height = height;
        self
    }

    pub fn with_x(mut self, x: i32) -> Self {
        self.x = x;
        self
    }

    pub fn with_y(mut self, y: i32) -> Self {
        self.y = y;
        self
    }

    pub fn with_position(mut self, x: i32, y: i32) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    pub fn centered(mut self) -> Self {
        self.centered = true;
        self
    }

    pub fn resizable(mut self) -> Self {
        self.resizable = true;
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn with_colors(mut self, colors: ColorScheme) -> Self {
        self.colors = colors;
        self
    }
}
