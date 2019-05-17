use crate::{data::WindowData, rest_api};
use diwata_intel::{window::GroupedWindow, Rows, Window};
use sauron::{
    html::{attributes::*, events::*, *},
    Browser, Cmd, Component, Dispatch, Http, Node,
};
use std::rc::Rc;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::Response;
use window_list_view::WindowListView;
use window_view::WindowView;

mod column_view;
mod detail_view;
mod field_view;
mod row_view;
mod tab_view;
mod table_view;
mod toolbar_view;
mod window_list_view;
mod window_view;

#[derive(Debug, Clone)]
pub enum Msg {
    ActivateWindow(usize),
    WindowMsg(usize, window_view::Msg),
    BrowserResized(i32, i32),
    Tick,
    WindowListMsg(window_list_view::Msg),
    FetchWindowList(Result<Vec<GroupedWindow>, JsValue>),
    ReceivedWindowQueryResult(usize, Result<Rows, JsValue>),
}

pub struct App {
    window_views: Vec<WindowView>,
    active_window: usize,
    browser_height: i32,
    browser_width: i32,
    window_list_view: WindowListView,
}

impl App {
    pub fn new(
        window_list: Vec<GroupedWindow>,
        windows: Vec<Window>,
        browser_width: i32,
        browser_height: i32,
    ) -> App {
        let mut app = App {
            window_views: windows
                .into_iter()
                .map(|window| WindowView::new(window, browser_width, browser_height))
                .collect(),
            window_list_view: WindowListView::new(window_list),
            active_window: 0,
            browser_width,
            browser_height,
        };
        app.update_active_window();
        app.update_size_allocation();
        app
    }

    pub fn set_window_data(&mut self, index: usize, window_data: WindowData) {
        self.window_views[index].set_window_data(window_data);
    }

    fn update_size_allocation(&mut self) {
        let window_list_size = self.calculate_window_list_size();
        self.window_list_view.set_allocated_size(window_list_size);
    }

    fn calculate_window_list_size(&self) -> (i32, i32) {
        (200, self.browser_height - self.logo_height())
    }

    fn logo_height(&self) -> i32 {
        170
    }

    fn update_active_window(&mut self) {
        let active_window = self.active_window;
        self.window_views
            .iter_mut()
            .enumerate()
            .for_each(|(index, window)| {
                if index == active_window {
                    window.show()
                } else {
                    window.hide()
                }
            })
    }

    fn activate_window(&mut self, index: usize) {
        self.active_window = index;
        self.update_active_window();
    }

    fn setup_window_resize_listener(&self) -> Cmd<App, Msg> {
        Browser::onresize(Msg::BrowserResized)
    }
}

impl Component<Msg> for App {
    fn init(&self) -> Cmd<Self, Msg> {
        Cmd::batch(vec![
            rest_api::fetch_window_list(),
            self.setup_window_resize_listener(),
        ])
    }

    fn update(&mut self, msg: Msg) -> Cmd<Self, Msg> {
        match msg {
            Msg::ActivateWindow(index) => {
                self.activate_window(index);
                Cmd::none()
            }
            //FIXME: This is managed here since Mapping in Cmd is not yet solved/supported
            Msg::WindowMsg(index, window_view::Msg::ToolbarMsg(toolbar_view::Msg::RunQuery)) => {
                let sql = self.window_views[index].get_sql_query();
                sauron::log!("In app.rs Run the query: {}", sql);
                rest_api::execute_sql_query(sql, move |rows| {
                    Msg::ReceivedWindowQueryResult(index, rows)
                })
            }
            Msg::WindowMsg(index, window_msg) => {
                self.window_views[index].update(window_msg);
                Cmd::none()
            }
            Msg::BrowserResized(width, height) => {
                sauron::log!("Browser is resized to: {}, {}", width, height);
                self.browser_width = width;
                self.browser_height = height;
                //also notify all opened windows with the resize;
                self.window_views.iter_mut().for_each(|window| {
                    window.update(window_view::Msg::BrowserResized(width, height));
                });
                self.update_size_allocation();
                Cmd::none()
            }
            Msg::Tick => {
                sauron::log("Ticking");
                Cmd::none()
            }
            Msg::WindowListMsg(window_list_msg) => {
                self.window_list_view.update(window_list_msg);
                Cmd::none()
            }
            Msg::FetchWindowList(Ok(window_list)) => {
                self.window_list_view
                    .update(window_list_view::Msg::ReceiveWindowList(window_list));
                Cmd::none()
            }
            Msg::FetchWindowList(Err(js_value)) => {
                sauron::log!("There was an error fetching window list: {:#?}", js_value);
                Cmd::none()
            }

            // FIXME: Also return the window, since the table
            // in the select from can be anything other than
            // the window's current main table.
            Msg::ReceivedWindowQueryResult(index, Ok(rows)) => {
                sauron::log!("Received window query result: {:#?}", rows);
                // FIXME: need to replace the window with a new one
                // with a new set of window data from this result
                let window_data = WindowData::from_rows(rows);
                self.window_views[index].set_window_data(window_data);
                Cmd::none()
            }
            Msg::ReceivedWindowQueryResult(index, Err(err)) => {
                sauron::log!("Error retrieveing records from sql query");
                Cmd::none()
            }
        }
    }

    fn view(&self) -> Node<Msg> {
        main(
            // GRID
            [class("app")],
            [
                section(
                    [class("logo_and_window_list")],
                    [
                        header([class("logo")], []),
                        self.window_list_view.view().map(Msg::WindowListMsg),
                    ],
                ),
                section(
                    [class("window_links_and_window_views")],
                    [
                        header(
                            [class("window_links_and_logout")],
                            [
                                nav(
                                    [class("logout")],
                                    [
                                        button([], [text("logout")]),
                                        button([], [text("Connect to database..")]),
                                    ],
                                ),
                                nav(
                                    [class("window_links")],
                                    self.window_views
                                        .iter()
                                        .enumerate()
                                        .map(|(index, window)| {
                                            a(
                                                [
                                                    class("tab_links"),
                                                    classes_flag([("active", window.is_visible)]),
                                                    onclick(move |_| Msg::ActivateWindow(index)),
                                                ],
                                                [text(&window.name)],
                                            )
                                        })
                                        .collect::<Vec<Node<Msg>>>(),
                                ),
                            ],
                        ),
                        section(
                            [class("window_views")],
                            self.window_views
                                .iter()
                                .enumerate()
                                .map(|(index, window)| {
                                    window
                                        .view()
                                        .map(move |window_msg| Msg::WindowMsg(index, window_msg))
                                })
                                .collect::<Vec<Node<Msg>>>(),
                        ),
                    ],
                ),
            ],
        )
    }
}