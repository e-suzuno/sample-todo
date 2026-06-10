//! 最小の in-memory TODO アプリ（Rust + GPUI）。
//!
//! 構成:
//! - ドメイン層: `Task` / `TaskList`（GPUI 非依存・純Rust。`#[cfg(test)]` で検証）
//! - View 層: `TextInput`（自前テキスト入力）/ `TodoApp`（一覧・追加・トグル・削除・残数）
//!
//! テキスト入力部は gpui の examples/input.rs（Apache-2.0）の利用パターンを参考に
//! 最小限へ縮小した自前実装。

use std::ops::Range;

use gpui::{
    App, Bounds, Context, ElementId, ElementInputHandler, Entity, EntityInputHandler, FocusHandle,
    Focusable, GlobalElementId, KeyBinding, LayoutId, Pixels, ShapedLine,
    SharedString, Style, TextRun, UTF16Selection, Window, WindowBounds, WindowOptions, actions,
    div, fill, hsla, point, prelude::*, px, relative, rgb, size, white,
};
use gpui_platform::application;
use unicode_segmentation::UnicodeSegmentation;

// ============================================================================
// ドメイン層（GPUI 非依存・純Rust）
// ============================================================================

/// 1件のタスク。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    /// 一意なID。
    pub id: u64,
    /// 表示文字列。
    pub title: String,
    /// 完了状態。
    pub completed: bool,
}

/// タスク一覧（in-memory）。
#[derive(Debug, Default)]
pub struct TaskList {
    tasks: Vec<Task>,
    next_id: u64,
}

impl TaskList {
    /// 空の一覧を生成する。
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            next_id: 1,
        }
    }

    /// タスク参照のスライスを返す。
    pub fn tasks(&self) -> &[Task] {
        &self.tasks
    }

    /// タスクを末尾に追加する。
    /// 空文字・空白のみは無視し、追加した場合のみ採番した ID を返す。
    /// 前後の空白はトリムして格納する。
    pub fn add(&mut self, title: &str) -> Option<u64> {
        let trimmed = title.trim();
        if trimmed.is_empty() {
            return None;
        }
        let id = self.next_id;
        self.next_id += 1;
        self.tasks.push(Task {
            id,
            title: trimmed.to_string(),
            completed: false,
        });
        Some(id)
    }

    /// 指定IDの完了状態を反転する。該当が無ければ何もしない。
    pub fn toggle(&mut self, id: u64) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
            task.completed = !task.completed;
        }
    }

    /// 指定IDのタスクを削除する。
    pub fn remove(&mut self, id: u64) {
        self.tasks.retain(|t| t.id != id);
    }

    /// 未完了タスクの件数を返す。
    pub fn remaining_count(&self) -> usize {
        self.tasks.iter().filter(|t| !t.completed).count()
    }
}

// ============================================================================
// View 層: テキスト入力（自前実装）
// ============================================================================

actions!(
    todo_input,
    [Backspace, Delete, Left, Right, Home, End, SubmitTask, Quit]
);

/// 単一行テキスト入力。フォーカス・IME・キャレットを自前で扱う。
struct TextInput {
    focus_handle: FocusHandle,
    content: SharedString,
    placeholder: SharedString,
    selected_range: Range<usize>,
    selection_reversed: bool,
    marked_range: Option<Range<usize>>,
    last_layout: Option<ShapedLine>,
    last_bounds: Option<Bounds<Pixels>>,
}

impl TextInput {
    fn new(cx: &mut Context<Self>, placeholder: impl Into<SharedString>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            content: "".into(),
            placeholder: placeholder.into(),
            selected_range: 0..0,
            selection_reversed: false,
            marked_range: None,
            last_layout: None,
            last_bounds: None,
        }
    }

    fn left(&mut self, _: &Left, _: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.move_to(self.previous_boundary(self.cursor_offset()), cx);
        } else {
            self.move_to(self.selected_range.start, cx);
        }
    }

    fn right(&mut self, _: &Right, _: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.move_to(self.next_boundary(self.cursor_offset()), cx);
        } else {
            self.move_to(self.selected_range.end, cx);
        }
    }

    fn home(&mut self, _: &Home, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0, cx);
    }

    fn end(&mut self, _: &End, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(self.content.len(), cx);
    }

    fn backspace(&mut self, _: &Backspace, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            let prev = self.previous_boundary(self.cursor_offset());
            if self.cursor_offset() == prev {
                return;
            }
            self.select_to(prev, cx);
        }
        self.replace_text_in_range(None, "", window, cx);
    }

    fn delete(&mut self, _: &Delete, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            let next = self.next_boundary(self.cursor_offset());
            if self.cursor_offset() == next {
                return;
            }
            self.select_to(next, cx);
        }
        self.replace_text_in_range(None, "", window, cx);
    }

    fn move_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        self.selected_range = offset..offset;
        cx.notify();
    }

    fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    fn select_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        if self.selection_reversed {
            self.selected_range.start = offset;
        } else {
            self.selected_range.end = offset;
        }
        if self.selected_range.end < self.selected_range.start {
            self.selection_reversed = !self.selection_reversed;
            self.selected_range = self.selected_range.end..self.selected_range.start;
        }
        cx.notify();
    }

    /// 現在の入力内容を取り出して空にする（タスク追加時に使用）。
    fn take_content(&mut self, cx: &mut Context<Self>) -> String {
        let value = self.content.to_string();
        self.content = "".into();
        self.selected_range = 0..0;
        self.selection_reversed = false;
        self.marked_range = None;
        cx.notify();
        value
    }

    fn offset_from_utf16(&self, offset: usize) -> usize {
        let mut utf8_offset = 0;
        let mut utf16_count = 0;
        for ch in self.content.chars() {
            if utf16_count >= offset {
                break;
            }
            utf16_count += ch.len_utf16();
            utf8_offset += ch.len_utf8();
        }
        utf8_offset
    }

    fn offset_to_utf16(&self, offset: usize) -> usize {
        let mut utf16_offset = 0;
        let mut utf8_count = 0;
        for ch in self.content.chars() {
            if utf8_count >= offset {
                break;
            }
            utf8_count += ch.len_utf8();
            utf16_offset += ch.len_utf16();
        }
        utf16_offset
    }

    fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    fn range_from_utf16(&self, range_utf16: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range_utf16.start)..self.offset_from_utf16(range_utf16.end)
    }

    fn previous_boundary(&self, offset: usize) -> usize {
        self.content
            .grapheme_indices(true)
            .rev()
            .find_map(|(idx, _)| (idx < offset).then_some(idx))
            .unwrap_or(0)
    }

    fn next_boundary(&self, offset: usize) -> usize {
        self.content
            .grapheme_indices(true)
            .find_map(|(idx, _)| (idx > offset).then_some(idx))
            .unwrap_or(self.content.len())
    }

    /// range を content の長さ・文字境界内へクランプする（スライス前の防御）。
    fn clamp_range(&self, range: Range<usize>) -> Range<usize> {
        let len = self.content.len();
        let mut start = range.start.min(len);
        let mut end = range.end.min(len);
        if start > end {
            std::mem::swap(&mut start, &mut end);
        }
        while !self.content.is_char_boundary(start) {
            start -= 1;
        }
        while !self.content.is_char_boundary(end) {
            end -= 1;
        }
        start..end
    }
}

impl EntityInputHandler for TextInput {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.range_from_utf16(&range_utf16);
        actual_range.replace(self.range_to_utf16(&range));
        Some(self.content[range].to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.range_to_utf16(&self.selected_range),
            reversed: self.selection_reversed,
        })
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        self.marked_range
            .as_ref()
            .map(|range| self.range_to_utf16(range))
    }

    fn unmark_text(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        self.marked_range = None;
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());

        // 想定外の range でパニックしないよう content の長さ・文字境界にクランプする。
        let range = self.clamp_range(range);

        self.content =
            (self.content[0..range.start].to_owned() + new_text + &self.content[range.end..])
                .into();
        self.selected_range = range.start + new_text.len()..range.start + new_text.len();
        self.marked_range.take();
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());

        // 想定外の range でパニックしないよう content の長さ・文字境界にクランプする。
        let range = self.clamp_range(range);

        self.content =
            (self.content[0..range.start].to_owned() + new_text + &self.content[range.end..])
                .into();
        if !new_text.is_empty() {
            self.marked_range = Some(range.start..range.start + new_text.len());
        } else {
            self.marked_range = None;
        }
        self.selected_range = new_selected_range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .map(|new_range| new_range.start + range.start..new_range.end + range.start)
            .unwrap_or_else(|| range.start + new_text.len()..range.start + new_text.len());

        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let last_layout = self.last_layout.as_ref()?;
        let range = self.range_from_utf16(&range_utf16);
        Some(Bounds::from_corners(
            point(
                bounds.left() + last_layout.x_for_index(range.start),
                bounds.top(),
            ),
            point(
                bounds.left() + last_layout.x_for_index(range.end),
                bounds.bottom(),
            ),
        ))
    }

    fn character_index_for_point(
        &mut self,
        point: gpui::Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        let line_point = self.last_bounds?.localize(&point)?;
        let last_layout = self.last_layout.as_ref()?;
        let utf8_index = last_layout.index_for_x(point.x - line_point.x)?;
        Some(self.offset_to_utf16(utf8_index))
    }
}

/// テキスト本体を描画する低レベル Element（キャレット・プレースホルダ含む）。
struct TextElement {
    input: Entity<TextInput>,
}

struct PrepaintState {
    line: Option<ShapedLine>,
    cursor: Option<gpui::PaintQuad>,
}

impl IntoElement for TextElement {
    type Element = Self;
    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for TextElement {
    type RequestLayoutState = ();
    type PrepaintState = PrepaintState;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = window.line_height().into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let input = self.input.read(cx);
        let content = input.content.clone();
        let cursor = input.cursor_offset();
        let style = window.text_style();

        let (display_text, text_color) = if content.is_empty() {
            (input.placeholder.clone(), hsla(0., 0., 0., 0.4))
        } else {
            (content, style.color)
        };

        let run = TextRun {
            len: display_text.len(),
            font: style.font(),
            color: text_color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let runs = vec![run];

        let font_size = style.font_size.to_pixels(window.rem_size());
        let line = window
            .text_system()
            .shape_line(display_text, font_size, &runs, None);

        let cursor_pos = line.x_for_index(cursor);
        let cursor = Some(fill(
            Bounds::new(
                point(bounds.left() + cursor_pos, bounds.top()),
                size(px(2.), bounds.bottom() - bounds.top()),
            ),
            gpui::blue(),
        ));

        PrepaintState {
            line: Some(line),
            cursor,
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus_handle = self.input.read(cx).focus_handle.clone();
        window.handle_input(
            &focus_handle,
            ElementInputHandler::new(bounds, self.input.clone()),
            cx,
        );
        let line = prepaint.line.take().unwrap();
        line.paint(
            bounds.origin,
            window.line_height(),
            gpui::TextAlign::Left,
            None,
            window,
            cx,
        )
        .unwrap();

        if focus_handle.is_focused(window)
            && let Some(cursor) = prepaint.cursor.take()
        {
            window.paint_quad(cursor);
        }

        self.input.update(cx, |input, _cx| {
            input.last_layout = Some(line);
            input.last_bounds = Some(bounds);
        });
    }
}

impl Render for TextInput {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .key_context("TodoInput")
            .track_focus(&self.focus_handle(cx))
            .cursor(gpui::CursorStyle::IBeam)
            .on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::delete))
            .on_action(cx.listener(Self::left))
            .on_action(cx.listener(Self::right))
            .on_action(cx.listener(Self::home))
            .on_action(cx.listener(Self::end))
            .bg(white())
            .line_height(px(30.))
            .text_size(px(18.))
            .child(
                div()
                    .h(px(30. + 4. * 2.))
                    .w_full()
                    .p(px(4.))
                    .child(TextElement { input: cx.entity() }),
            )
    }
}

impl Focusable for TextInput {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

// ============================================================================
// View 層: TODO アプリ本体
// ============================================================================

struct TodoApp {
    list: TaskList,
    input: Entity<TextInput>,
    focus_handle: FocusHandle,
}

impl TodoApp {
    fn new(cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| TextInput::new(cx, "新しいタスクを入力..."));
        Self {
            list: TaskList::new(),
            input,
            focus_handle: cx.focus_handle(),
        }
    }

    /// 入力欄の内容をタスクとして追加し、入力欄を空にする（F3）。
    fn add_task(&mut self, cx: &mut Context<Self>) {
        let title = self.input.update(cx, |input, cx| input.take_content(cx));
        self.list.add(&title);
        cx.notify();
    }

    /// Enter キー（SubmitTask アクション）で追加（F3）。
    fn on_submit(&mut self, _: &SubmitTask, _: &mut Window, cx: &mut Context<Self>) {
        self.add_task(cx);
    }

    /// 完了状態を切替える（F4）。
    fn toggle_task(&mut self, id: u64, cx: &mut Context<Self>) {
        self.list.toggle(id);
        cx.notify();
    }

    /// タスクを削除する（F5）。
    fn remove_task(&mut self, id: u64, cx: &mut Context<Self>) {
        self.list.remove(id);
        cx.notify();
    }
}

impl Focusable for TodoApp {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TodoApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 一覧行を構築（F4 完了表示・F5 削除ボタン）。
        let rows: Vec<_> = self
            .list
            .tasks()
            .iter()
            .map(|task| {
                let id = task.id;
                let completed = task.completed;

                // タイトル: 完了なら打消し線＋グレー表示（F4）。
                let mut title = div()
                    .flex_grow(1.0)
                    .child(SharedString::from(task.title.clone()));
                if completed {
                    title = title.line_through().text_color(rgb(0x999999));
                } else {
                    title = title.text_color(rgb(0x111111));
                }

                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .px_2()
                    .py_1()
                    .border_b_1()
                    .border_color(rgb(0xdddddd))
                    // チェックボックス兼トグル（F4）。
                    .child(
                        div()
                            .id(("toggle", id as usize))
                            .w(px(20.))
                            .h(px(20.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .border_1()
                            .border_color(rgb(0x888888))
                            .rounded_sm()
                            .cursor_pointer()
                            .when(completed, |s| s.bg(rgb(0x4caf50)))
                            .child(if completed { "x" } else { "" })
                            .on_click(cx.listener(move |this, _, _window, cx| {
                                this.toggle_task(id, cx);
                            })),
                    )
                    .child(title)
                    // 削除ボタン（F5）。
                    .child(
                        div()
                            .id(("remove", id as usize))
                            .px_2()
                            .text_color(white())
                            .bg(rgb(0xe53935))
                            .rounded_sm()
                            .cursor_pointer()
                            .child("削除")
                            .hover(|s| s.bg(rgb(0xc62828)))
                            .on_click(cx.listener(move |this, _, _window, cx| {
                                this.remove_task(id, cx);
                            })),
                    )
            })
            .collect();

        div()
            .key_context("TodoApp")
            .track_focus(&self.focus_handle(cx))
            // Enter キーで追加（F3）。
            .on_action(cx.listener(Self::on_submit))
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0xf5f5f5))
            .text_color(rgb(0x111111))
            .text_size(px(16.))
            .p_4()
            .gap_3()
            // タイトル。
            .child(div().text_size(px(24.)).child("TODO リスト"))
            // 入力欄＋追加ボタン（F2 / F3）。
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_2()
                    .items_center()
                    .child(
                        div()
                            .flex_grow(1.0)
                            .border_1()
                            .border_color(rgb(0xbbbbbb))
                            .rounded_sm()
                            .child(self.input.clone()),
                    )
                    .child(
                        div()
                            .id("add")
                            .px_3()
                            .py_2()
                            .bg(rgb(0x1976d2))
                            .text_color(white())
                            .rounded_sm()
                            .cursor_pointer()
                            .child("追加")
                            .hover(|s| s.bg(rgb(0x1565c0)))
                            .on_click(cx.listener(|this, _, _window, cx| {
                                this.add_task(cx);
                            })),
                    ),
            )
            // 残数表示（F6）。
            .child(
                div()
                    .text_color(rgb(0x555555))
                    .child(SharedString::from(format!(
                        "未完了 {}件",
                        self.list.remaining_count()
                    ))),
            )
            // タスク一覧。
            .child(
                div()
                    .flex()
                    .flex_col()
                    .bg(white())
                    .rounded_md()
                    .children(rows),
            )
    }
}

// ============================================================================
// エントリポイント（F1: ウィンドウ表示）
// ============================================================================

fn main() {
    application().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(480.0), px(600.0)), cx);

        // キーバインド登録。Enter で追加（F3）、編集系キー、Quit。
        // Quit は mac の cmd-q に加え、Linux/Windows 向けに ctrl-q も登録する。
        cx.on_action(|_: &Quit, cx| cx.quit());
        cx.bind_keys([
            KeyBinding::new("enter", SubmitTask, Some("TodoApp")),
            KeyBinding::new("backspace", Backspace, Some("TodoInput")),
            KeyBinding::new("delete", Delete, Some("TodoInput")),
            KeyBinding::new("left", Left, Some("TodoInput")),
            KeyBinding::new("right", Right, Some("TodoInput")),
            KeyBinding::new("home", Home, Some("TodoInput")),
            KeyBinding::new("end", End, Some("TodoInput")),
            KeyBinding::new("cmd-q", Quit, None),
            KeyBinding::new("ctrl-q", Quit, None),
        ]);

        let window = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(gpui::TitlebarOptions {
                        title: Some(SharedString::from("Sample TODO")),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |_, cx| cx.new(TodoApp::new),
            )
            .unwrap();

        // 起動時に入力欄へフォーカス（F2）。
        window
            .update(cx, |app, window, cx| {
                window.focus(&app.input.focus_handle(cx), cx);
                cx.activate(true);
            })
            .unwrap();
    });
}

// ============================================================================
// テスト（ドメイン層: GPUI 非依存で完結）
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_appends_non_empty_task() {
        let mut list = TaskList::new();
        let id = list.add("買い物").expect("空でないので追加される");
        assert_eq!(list.tasks().len(), 1);
        assert_eq!(list.tasks()[0].title, "買い物");
        assert_eq!(list.tasks()[0].id, id);
        assert!(!list.tasks()[0].completed);
    }

    #[test]
    fn add_ignores_empty_and_whitespace() {
        let mut list = TaskList::new();
        assert_eq!(list.add(""), None);
        assert_eq!(list.add("   "), None);
        assert_eq!(list.add("\t\n"), None);
        assert_eq!(list.tasks().len(), 0);
    }

    #[test]
    fn add_trims_and_preserves_order() {
        let mut list = TaskList::new();
        list.add("  一番目  ");
        list.add("二番目");
        assert_eq!(list.tasks()[0].title, "一番目");
        assert_eq!(list.tasks()[1].title, "二番目");
    }

    #[test]
    fn ids_are_unique_and_incrementing() {
        let mut list = TaskList::new();
        let a = list.add("a").unwrap();
        let b = list.add("b").unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn toggle_flips_completed() {
        let mut list = TaskList::new();
        let id = list.add("タスク").unwrap();
        assert!(!list.tasks()[0].completed);
        list.toggle(id);
        assert!(list.tasks()[0].completed);
        list.toggle(id);
        assert!(!list.tasks()[0].completed);
    }

    #[test]
    fn toggle_unknown_id_is_noop() {
        let mut list = TaskList::new();
        list.add("タスク").unwrap();
        list.toggle(9999); // 存在しないID
        assert!(!list.tasks()[0].completed);
    }

    #[test]
    fn remove_unknown_id_is_noop() {
        let mut list = TaskList::new();
        list.add("タスク").unwrap();
        list.remove(9999); // 存在しないID
        assert_eq!(list.tasks().len(), 1);
    }

    #[test]
    fn remove_deletes_target_only() {
        let mut list = TaskList::new();
        let a = list.add("a").unwrap();
        let b = list.add("b").unwrap();
        list.remove(a);
        assert_eq!(list.tasks().len(), 1);
        assert_eq!(list.tasks()[0].id, b);
    }

    #[test]
    fn remaining_count_tracks_completion() {
        let mut list = TaskList::new();
        let a = list.add("a").unwrap();
        let _b = list.add("b").unwrap();
        let c = list.add("c").unwrap();
        assert_eq!(list.remaining_count(), 3);
        list.toggle(a);
        assert_eq!(list.remaining_count(), 2);
        list.toggle(c);
        assert_eq!(list.remaining_count(), 1);
        list.remove(a); // 完了済みを削除しても未完了数は変わらない
        assert_eq!(list.remaining_count(), 1);
    }
}
