extends Node3D

const MODE_SPHERE := "sphere"
const MODE_BOARD := "board"

const MENU_RENAME := 1
const MENU_DUPLICATE := 2
const MENU_EXPORT := 3
const MENU_DELETE := 4
const MENU_SHORTCUTS := 5
const MENU_ABOUT := 6

var sphere_view: SphereView
var board_view: BoardView

var _ui_layer: CanvasLayer
var _ui_root: Control
var _left_panel: PanelContainer
var _right_panel: PanelContainer
var _top_bar: PanelContainer
var _board_host: Control
var _empty_overlay: CenterContainer
var _bottom_dock: PanelContainer
var _toast_panel: PanelContainer
var _toast_label: Label
var _toast_dot: Panel
var _toast_timer: Timer
var _toast_tween: Tween
var _modal_layer: Control
var _modal_center: CenterContainer

var _logo: TextureRect
var _sphere_search: LineEdit
var _sphere_list_box: VBoxContainer
var _sphere_title_label: Label
var _sphere_meta_label: Label
var _save_label: Label
var _save_dot: Panel
var _mode_sphere_button: Button
var _mode_board_button: Button
var _hint_label: Label
var _inspector_body: VBoxContainer
var _inspector_kicker: Label
var _inspector_dot: Panel
var _title_edit: LineEdit
var _body_edit: TextEdit
var _group_option: OptionButton
var _link_option: OptionButton

var _save_timer: Timer
var _export_dialog: FileDialog
var _import_dialog: FileDialog
var _web_import_callback
var _empty_panel: PanelContainer
var _empty_button: Button

var _modal_line: LineEdit
var _modal_submit_callback: Callable
var _modal_confirm_callback: Callable
var _modal_fallback := ""

var spheres: Array[Dictionary] = []
var current: Dictionary = {}
var selected_node_id := ""
var view_mode := MODE_SPHERE
var dirty := false
var _list_needs_refresh := false
var _suppress_inspector_edits := false
var _last_sphere_id := ""


func _ready() -> void:
	get_window().title = "Mindsphere"
	get_window().min_size = Vector2i(1280, 800)
	sphere_view = SphereView.new()
	sphere_view.name = "SphereView"
	add_child(sphere_view)
	sphere_view.node_selected.connect(_on_node_selected)
	sphere_view.node_activated.connect(_on_node_activated)
	sphere_view.node_moved.connect(_on_node_moved)
	sphere_view.background_clicked.connect(_clear_selection)
	sphere_view.camera_changed.connect(_on_camera_changed)

	_build_ui()
	_build_file_dialogs()
	_build_timers()
	_setup_web_import()
	get_window().files_dropped.connect(_on_files_dropped)

	SphereStore.ensure_library()
	var settings := SphereStore.load_settings()
	_last_sphere_id = String(settings.get("last_sphere", ""))
	view_mode = String(settings.get("mode", MODE_SPHERE))
	if view_mode != MODE_BOARD:
		view_mode = MODE_SPHERE
	_refresh_sphere_list()
	var initial := _last_sphere_id
	if initial.is_empty() or SphereStore.load_sphere(initial).is_empty():
		initial = String(spheres[0].id) if not spheres.is_empty() else ""
	if initial.is_empty():
		var blank := SphereStore.create_blank("New sphere")
		SphereStore.save_sphere(blank)
		_refresh_sphere_list()
		initial = String(blank.id)
	_load_sphere(initial)
	_set_mode(view_mode)
	_set_save_status("Saved locally", UiFactory.MINT)


func _notification(what: int) -> void:
	if what == NOTIFICATION_WM_CLOSE_REQUEST:
		if not current.is_empty() and dirty:
			_save_current()


func _build_ui() -> void:
	_ui_layer = CanvasLayer.new()
	_ui_layer.name = "Interface"
	_ui_layer.layer = 2
	add_child(_ui_layer)
	_ui_root = Control.new()
	_ui_root.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	_ui_root.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_ui_root.theme = UiFactory.make_theme()
	_ui_layer.add_child(_ui_root)

	_build_board_host()
	_build_empty_overlay()
	_build_left_panel()
	_build_right_panel()
	_build_top_bar()
	_build_bottom_dock()
	_build_toast()
	_build_modal_layer()


func _build_left_panel() -> void:
	_left_panel = PanelContainer.new()
	_left_panel.anchor_top = 0.0
	_left_panel.anchor_bottom = 1.0
	_left_panel.offset_right = 286.0
	_left_panel.mouse_filter = Control.MOUSE_FILTER_STOP
	var style := UiFactory.panel_style(Color(0.031, 0.045, 0.078, 0.96), Color("#1a2538"), 0)
	style.border_width_right = 1
	style.content_margin_left = 20
	style.content_margin_right = 18
	style.content_margin_top = 20
	style.content_margin_bottom = 18
	_left_panel.add_theme_stylebox_override("panel", style)
	_ui_root.add_child(_left_panel)

	var box := VBoxContainer.new()
	box.add_theme_constant_override("separation", 12)
	_left_panel.add_child(box)

	var brand := HBoxContainer.new()
	brand.add_theme_constant_override("separation", 11)
	box.add_child(brand)
	_logo = TextureRect.new()
	_logo.texture = load("res://icon.svg")
	_logo.custom_minimum_size = Vector2(40, 40)
	_logo.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	_logo.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
	_logo.mouse_filter = Control.MOUSE_FILTER_IGNORE
	brand.add_child(_logo)
	var brand_text := VBoxContainer.new()
	brand_text.add_theme_constant_override("separation", -1)
	brand_text.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	brand.add_child(brand_text)
	brand_text.add_child(UiFactory.label("Mindsphere", 21, UiFactory.TEXT))
	brand_text.add_child(UiFactory.label("Think on a sphere", 11, UiFactory.MUTED))

	box.add_child(UiFactory.spacer(5))
	box.add_child(UiFactory.separator(2, 7))

	var spheres_header := HBoxContainer.new()
	spheres_header.add_theme_constant_override("separation", 8)
	box.add_child(spheres_header)
	var spheres_kicker := UiFactory.kicker("YOUR SPHERES", UiFactory.MUTED)
	spheres_kicker.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	spheres_header.add_child(spheres_kicker)
	var add_sphere := UiFactory.button("+", false, true)
	add_sphere.tooltip_text = "Create a new sphere"
	add_sphere.custom_minimum_size = Vector2(30, 26)
	add_sphere.pressed.connect(_show_new_sphere_modal)
	spheres_header.add_child(add_sphere)

	_sphere_search = UiFactory.line_edit("Search spheres", 13)
	_sphere_search.custom_minimum_size.y = 38
	_sphere_search.text_changed.connect(func(_text: String) -> void: _refresh_sphere_list())
	box.add_child(_sphere_search)

	var scroll := ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.vertical_scroll_mode = ScrollContainer.SCROLL_MODE_AUTO
	box.add_child(scroll)
	_sphere_list_box = VBoxContainer.new()
	_sphere_list_box.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_sphere_list_box.add_theme_constant_override("separation", 6)
	scroll.add_child(_sphere_list_box)

	box.add_child(UiFactory.separator(4, 7))

	var local_row := HBoxContainer.new()
	local_row.add_theme_constant_override("separation", 8)
	box.add_child(local_row)
	local_row.add_child(UiFactory.color_dot(UiFactory.MINT, 7))
	var local := UiFactory.label("Stored on this device", 11, UiFactory.MUTED)
	local.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	local_row.add_child(local)
	local_row.add_child(UiFactory.label("No cloud", 10, UiFactory.FAINT))

	var io_row := HBoxContainer.new()
	io_row.add_theme_constant_override("separation", 6)
	box.add_child(io_row)
	var import_button := UiFactory.ghost_button("Import", false, true)
	import_button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	import_button.pressed.connect(_show_import_dialog)
	io_row.add_child(import_button)
	var export_button := UiFactory.ghost_button("Export", false, true)
	export_button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	export_button.pressed.connect(_show_export_dialog)
	io_row.add_child(export_button)


func _build_right_panel() -> void:
	_right_panel = PanelContainer.new()
	_right_panel.anchor_left = 1.0
	_right_panel.anchor_right = 1.0
	_right_panel.anchor_top = 0.0
	_right_panel.anchor_bottom = 1.0
	_right_panel.offset_left = -336.0
	_right_panel.mouse_filter = Control.MOUSE_FILTER_STOP
	var style := UiFactory.panel_style(Color(0.031, 0.045, 0.078, 0.96), Color("#1a2538"), 0)
	style.border_width_left = 1
	style.content_margin_left = 18
	style.content_margin_right = 18
	style.content_margin_top = 20
	style.content_margin_bottom = 18
	_right_panel.add_theme_stylebox_override("panel", style)
	_ui_root.add_child(_right_panel)

	var box := VBoxContainer.new()
	box.add_theme_constant_override("separation", 12)
	_right_panel.add_child(box)

	var header := HBoxContainer.new()
	header.add_theme_constant_override("separation", 8)
	box.add_child(header)
	_inspector_dot = UiFactory.color_dot(UiFactory.ACCENT, 8)
	header.add_child(_inspector_dot)
	_inspector_kicker = UiFactory.kicker("INSPECTOR", UiFactory.MUTED)
	_inspector_kicker.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	header.add_child(_inspector_kicker)
	var help := UiFactory.ghost_button("?", false, true)
	help.tooltip_text = "Keyboard shortcuts"
	help.custom_minimum_size = Vector2(28, 24)
	help.pressed.connect(_show_shortcuts_modal)
	header.add_child(help)

	box.add_child(UiFactory.separator(1, 7))

	var scroll := ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.vertical_scroll_mode = ScrollContainer.SCROLL_MODE_AUTO
	box.add_child(scroll)
	_inspector_body = VBoxContainer.new()
	_inspector_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_inspector_body.add_theme_constant_override("separation", 10)
	scroll.add_child(_inspector_body)


func _build_top_bar() -> void:
	_top_bar = PanelContainer.new()
	_top_bar.anchor_left = 0.0
	_top_bar.anchor_right = 1.0
	_top_bar.offset_left = 286.0
	_top_bar.offset_right = -336.0
	_top_bar.offset_bottom = 72.0
	_top_bar.mouse_filter = Control.MOUSE_FILTER_STOP
	var style := UiFactory.panel_style(Color(0.032, 0.047, 0.082, 0.985), Color(0, 0, 0, 0), 0)
	style.border_width_bottom = 1
	style.border_color = Color("#1c293e")
	style.content_margin_left = 24
	style.content_margin_right = 20
	style.content_margin_top = 12
	style.content_margin_bottom = 12
	_top_bar.add_theme_stylebox_override("panel", style)
	_ui_root.add_child(_top_bar)

	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 12)
	_top_bar.add_child(row)
	var titles := VBoxContainer.new()
	titles.add_theme_constant_override("separation", 0)
	titles.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	titles.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	row.add_child(titles)
	_sphere_title_label = UiFactory.label("Sphere", 21, UiFactory.TEXT)
	titles.add_child(_sphere_title_label)
	_sphere_meta_label = UiFactory.label("", 11, UiFactory.MUTED)
	titles.add_child(_sphere_meta_label)

	var save_chip := HBoxContainer.new()
	save_chip.add_theme_constant_override("separation", 6)
	save_chip.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	row.add_child(save_chip)
	_save_dot = UiFactory.color_dot(UiFactory.MINT, 7)
	save_chip.add_child(_save_dot)
	_save_label = UiFactory.label("Saved locally", 11, UiFactory.MUTED)
	save_chip.add_child(_save_label)

	var new_thought := UiFactory.button("+  New thought", true)
	new_thought.tooltip_text = "Add a thought (N)"
	new_thought.custom_minimum_size.x = 144
	new_thought.pressed.connect(_add_thought)
	row.add_child(new_thought)

	var menu := MenuButton.new()
	menu.text = "More"
	menu.custom_minimum_size = Vector2(68, 40)
	menu.add_theme_font_size_override("font_size", 13)
	menu.mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND
	var popup := menu.get_popup()
	popup.add_item("Rename sphere", MENU_RENAME)
	popup.add_item("Duplicate sphere", MENU_DUPLICATE)
	popup.add_item("Export as JSON", MENU_EXPORT)
	popup.add_separator()
	popup.add_item("Keyboard shortcuts", MENU_SHORTCUTS)
	popup.add_item("About Mindsphere", MENU_ABOUT)
	popup.add_separator()
	popup.add_item("Delete sphere", MENU_DELETE)
	popup.id_pressed.connect(_on_menu_pressed)
	row.add_child(menu)


func _build_board_host() -> void:
	_board_host = Control.new()
	_board_host.anchor_left = 0.0
	_board_host.anchor_right = 1.0
	_board_host.anchor_top = 0.0
	_board_host.anchor_bottom = 1.0
	_board_host.offset_left = 286.0
	_board_host.offset_right = -336.0
	_board_host.offset_top = 72.0
	_board_host.mouse_filter = Control.MOUSE_FILTER_PASS
	_board_host.visible = false
	_ui_root.add_child(_board_host)
	board_view = BoardView.new()
	board_view.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	board_view.node_selected.connect(_on_board_node_selected)
	board_view.node_activated.connect(_on_node_activated)
	board_view.add_requested.connect(_on_board_add_requested)
	board_view.background_clicked.connect(_clear_selection)
	_board_host.add_child(board_view)


func _build_bottom_dock() -> void:
	var holder := CenterContainer.new()
	holder.anchor_left = 0.0
	holder.anchor_right = 1.0
	holder.anchor_top = 1.0
	holder.anchor_bottom = 1.0
	holder.offset_left = 286.0
	holder.offset_right = -336.0
	holder.offset_top = -92.0
	holder.offset_bottom = -18.0
	holder.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_ui_root.add_child(holder)

	_bottom_dock = PanelContainer.new()
	var style := UiFactory.panel_style(Color(0.047, 0.068, 0.112, 0.985), Color("#24344e"), 14)
	style.content_margin_left = 8
	style.content_margin_right = 14
	style.content_margin_top = 7
	style.content_margin_bottom = 7
	style.shadow_color = Color(0, 0, 0, 0.35)
	style.shadow_size = 12
	_bottom_dock.add_theme_stylebox_override("panel", style)
	_bottom_dock.mouse_filter = Control.MOUSE_FILTER_STOP
	holder.add_child(_bottom_dock)
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 10)
	_bottom_dock.add_child(row)

	_mode_sphere_button = UiFactory.button("Sphere  3D", false, true)
	_mode_sphere_button.custom_minimum_size = Vector2(104, 30)
	_mode_sphere_button.pressed.connect(func() -> void: _set_mode(MODE_SPHERE))
	row.add_child(_mode_sphere_button)
	_mode_board_button = UiFactory.button("Board  2D", false, true)
	_mode_board_button.custom_minimum_size = Vector2(96, 30)
	_mode_board_button.pressed.connect(func() -> void: _set_mode(MODE_BOARD))
	row.add_child(_mode_board_button)
	var divider := VSeparator.new()
	divider.add_theme_color_override("separator", Color("#263650"))
	row.add_child(divider)
	_hint_label = UiFactory.label("Drag to orbit  ·  Scroll to zoom  ·  Drag a thought to move it", 11, UiFactory.MUTED)
	row.add_child(_hint_label)


func _build_empty_overlay() -> void:
	_empty_overlay = CenterContainer.new()
	_empty_overlay.anchor_left = 0.0
	_empty_overlay.anchor_right = 1.0
	_empty_overlay.anchor_top = 0.0
	_empty_overlay.anchor_bottom = 1.0
	_empty_overlay.offset_left = 286.0
	_empty_overlay.offset_right = -336.0
	_empty_overlay.offset_top = 72.0
	_empty_overlay.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_empty_overlay.visible = false
	_ui_root.add_child(_empty_overlay)
	_empty_panel = PanelContainer.new()
	_empty_panel.custom_minimum_size = Vector2(420, 230)
	var style := UiFactory.panel_style(Color(0.047, 0.068, 0.112, 0.96), Color("#2a3c59"), 20)
	style.content_margin_left = 30
	style.content_margin_right = 30
	style.content_margin_top = 28
	style.content_margin_bottom = 26
	style.shadow_color = Color(0, 0, 0, 0.4)
	style.shadow_size = 22
	_empty_panel.add_theme_stylebox_override("panel", style)
	_empty_overlay.add_child(_empty_panel)
	var box := VBoxContainer.new()
	box.add_theme_constant_override("separation", 10)
	_empty_panel.add_child(box)
	var kicker := UiFactory.kicker("EMPTY SPHERE", UiFactory.ACCENT)
	kicker.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	box.add_child(kicker)
	var title := UiFactory.label("Begin with one thought", 24, UiFactory.TEXT)
	title.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	box.add_child(title)
	var hint := UiFactory.label("Put it on the sphere first. You can group it and connect it to everything else later.", 13, UiFactory.MUTED)
	hint.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	hint.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	hint.custom_minimum_size.x = 350
	box.add_child(hint)
	_empty_button = UiFactory.button("+  New thought", true)
	_empty_button.custom_minimum_size.x = 160
	_empty_button.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	_empty_button.pressed.connect(_add_thought)
	box.add_child(_empty_button)


func _build_toast() -> void:
	var holder := CenterContainer.new()
	holder.anchor_left = 0.0
	holder.anchor_right = 1.0
	holder.anchor_top = 1.0
	holder.anchor_bottom = 1.0
	holder.offset_left = 286.0
	holder.offset_right = -336.0
	holder.offset_top = -158.0
	holder.offset_bottom = -108.0
	holder.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_ui_root.add_child(holder)
	_toast_panel = PanelContainer.new()
	var style := UiFactory.panel_style(Color(0.055, 0.085, 0.13, 0.98), Color("#314866"), 11)
	style.content_margin_left = 14
	style.content_margin_right = 16
	style.content_margin_top = 8
	style.content_margin_bottom = 8
	style.shadow_color = Color(0, 0, 0, 0.35)
	style.shadow_size = 10
	_toast_panel.add_theme_stylebox_override("panel", style)
	_toast_panel.visible = false
	holder.add_child(_toast_panel)
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 8)
	_toast_panel.add_child(row)
	_toast_dot = UiFactory.color_dot(UiFactory.ACCENT, 7)
	row.add_child(_toast_dot)
	_toast_label = UiFactory.label("", 12, UiFactory.TEXT_SOFT)
	row.add_child(_toast_label)


func _build_modal_layer() -> void:
	_modal_layer = Control.new()
	_modal_layer.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	_modal_layer.mouse_filter = Control.MOUSE_FILTER_STOP
	_modal_layer.visible = false
	_ui_root.add_child(_modal_layer)


func _build_file_dialogs() -> void:
	_export_dialog = FileDialog.new()
	_export_dialog.file_mode = FileDialog.FILE_MODE_SAVE_FILE
	_export_dialog.access = FileDialog.ACCESS_FILESYSTEM
	_export_dialog.filters = PackedStringArray(["*.json ; Mindsphere sphere"])
	_export_dialog.title = "Export sphere as JSON"
	_export_dialog.use_native_dialog = false
	_export_dialog.file_selected.connect(_on_export_selected)
	add_child(_export_dialog)
	_import_dialog = FileDialog.new()
	_import_dialog.file_mode = FileDialog.FILE_MODE_OPEN_FILE
	_import_dialog.access = FileDialog.ACCESS_FILESYSTEM
	_import_dialog.filters = PackedStringArray(["*.json ; Mindsphere sphere"])
	_import_dialog.title = "Import a sphere"
	_import_dialog.use_native_dialog = false
	_import_dialog.file_selected.connect(_on_import_selected)
	add_child(_import_dialog)


func _build_timers() -> void:
	_save_timer = Timer.new()
	_save_timer.one_shot = true
	_save_timer.wait_time = 0.45
	_save_timer.timeout.connect(_save_current)
	add_child(_save_timer)
	_toast_timer = Timer.new()
	_toast_timer.one_shot = true
	_toast_timer.wait_time = 2.4
	_toast_timer.timeout.connect(_hide_toast)
	add_child(_toast_timer)


func _refresh_sphere_list() -> void:
	if _sphere_list_box == null:
		return
	for child in _sphere_list_box.get_children():
		child.queue_free()
	spheres = SphereStore.list_spheres()
	var filter := ""
	if _sphere_search != null:
		filter = _sphere_search.text.strip_edges().to_lower()
	var shown := 0
	for summary: Dictionary in spheres:
		var title := String(summary.title)
		if not filter.is_empty() and not title.to_lower().contains(filter):
			continue
		shown += 1
		_sphere_list_box.add_child(_make_sphere_row(summary))
	if shown == 0:
		var none := UiFactory.label("No spheres match that search.", 12, UiFactory.FAINT)
		none.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		_sphere_list_box.add_child(none)


func _make_sphere_row(summary: Dictionary) -> Control:
	var is_current := String(summary.id) == String(current.get("id", ""))
	var panel := PanelContainer.new()
	panel.mouse_filter = Control.MOUSE_FILTER_STOP
	panel.mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND
	var style := UiFactory.panel_style(
		Color("#102238") if is_current else Color("#0b1424"),
		Color("#3a6786") if is_current else Color("#17243a"),
		10
	)
	style.content_margin_left = 11
	style.content_margin_right = 10
	style.content_margin_top = 9
	style.content_margin_bottom = 9
	panel.add_theme_stylebox_override("panel", style)
	var box := VBoxContainer.new()
	box.add_theme_constant_override("separation", 3)
	panel.add_child(box)
	var top := HBoxContainer.new()
	top.add_theme_constant_override("separation", 7)
	box.add_child(top)
	top.add_child(UiFactory.color_dot(UiFactory.ACCENT if is_current else UiFactory.FAINT, 7))
	var title := UiFactory.label(String(summary.title), 13, UiFactory.TEXT if is_current else UiFactory.TEXT_SOFT)
	title.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	title.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	top.add_child(title)
	var nodes := int(summary.node_count)
	var groups := int(summary.group_count)
	var meta := UiFactory.label("%d thought%s  ·  %d group%s" % [
		nodes, "" if nodes == 1 else "s", groups, "" if groups == 1 else "s"
	], 10, UiFactory.MUTED if is_current else UiFactory.FAINT)
	box.add_child(meta)
	panel.gui_input.connect(_on_sphere_row_input.bind(String(summary.id), panel, is_current))
	panel.mouse_entered.connect(_on_sphere_row_hover.bind(panel, is_current, true))
	panel.mouse_exited.connect(_on_sphere_row_hover.bind(panel, is_current, false))
	return panel


func _on_sphere_row_input(event: InputEvent, sphere_id: String, _panel: PanelContainer, _is_current: bool) -> void:
	if event is InputEventMouseButton:
		var mouse := event as InputEventMouseButton
		if mouse.button_index == MOUSE_BUTTON_LEFT and mouse.pressed:
			_load_sphere(sphere_id)
			get_viewport().set_input_as_handled()


func _on_sphere_row_hover(panel: PanelContainer, is_current: bool, entered: bool) -> void:
	if panel == null or not is_instance_valid(panel) or is_current:
		return
	var style := UiFactory.panel_style(Color("#121e31") if entered else Color("#0b1424"), Color("#283b58") if entered else Color("#17243a"), 10)
	style.content_margin_left = 11
	style.content_margin_right = 10
	style.content_margin_top = 9
	style.content_margin_bottom = 9
	panel.add_theme_stylebox_override("panel", style)


func _load_sphere(sphere_id: String) -> void:
	if sphere_id.is_empty() or sphere_id == String(current.get("id", "")):
		return
	if not current.is_empty() and dirty:
		_save_current()
	var loaded := SphereStore.load_sphere(sphere_id)
	if loaded.is_empty():
		_toast("Could not open that sphere.", UiFactory.CORAL)
		return
	current = loaded
	selected_node_id = ""
	_last_sphere_id = sphere_id
	dirty = false
	_list_needs_refresh = false
	_suppress_inspector_edits = true
	sphere_view.set_data(current, true)
	_suppress_inspector_edits = false
	board_view.set_data(current, "")
	_refresh_top_bar()
	_refresh_inspector()
	_refresh_sphere_list()
	_refresh_empty_state()
	_set_mode(view_mode)
	_set_save_status("Saved locally", UiFactory.MINT)
	_save_app_settings()


func _refresh_top_bar() -> void:
	if current.is_empty():
		return
	var node_count: int = current.get("nodes", []).size()
	var link_count: int = current.get("links", []).size()
	var group_count: int = current.get("groups", []).size()
	_sphere_title_label.text = String(current.title)
	_sphere_meta_label.text = "%d thought%s  ·  %d connection%s  ·  %d group%s" % [
		node_count, "" if node_count == 1 else "s",
		link_count, "" if link_count == 1 else "s",
		group_count, "" if group_count == 1 else "s",
	]


func _refresh_inspector() -> void:
	if _inspector_body == null:
		return
	for child in _inspector_body.get_children():
		child.queue_free()
	_title_edit = null
	_body_edit = null
	_group_option = null
	_link_option = null
	if current.is_empty():
		return
	var node := SphereStore.find_node(current, selected_node_id)
	_suppress_inspector_edits = true
	if node.is_empty():
		selected_node_id = ""
		sphere_view.set_selected("")
		board_view.set_selected("")
		_inspector_kicker.text = "OVERVIEW"
		_inspector_dot.add_theme_stylebox_override("panel", UiFactory.panel_style(UiFactory.ACCENT, Color(0, 0, 0, 0), 4, 0))
		_build_inspector_overview()
	else:
		_inspector_kicker.text = "SELECTED THOUGHT"
		var color := SphereStore.group_color(current, String(node.group_id))
		_inspector_dot.add_theme_stylebox_override("panel", UiFactory.panel_style(color, Color(0, 0, 0, 0), 4, 0))
		_build_inspector_node(node)
	_suppress_inspector_edits = false


func _build_inspector_overview() -> void:
	var heading := UiFactory.label("Nothing selected", 20, UiFactory.TEXT)
	_inspector_body.add_child(heading)
	var intro := UiFactory.label("Pick a thought on the sphere or the board to edit it, move it into a group, or connect it to another thought.", 13, UiFactory.MUTED)
	intro.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_inspector_body.add_child(intro)

	_inspector_body.add_child(UiFactory.spacer(3))
	var stats := HBoxContainer.new()
	stats.add_theme_constant_override("separation", 8)
	_inspector_body.add_child(stats)
	var n: int = current.get("nodes", []).size()
	var l: int = current.get("links", []).size()
	var g: int = current.get("groups", []).size()
	stats.add_child(_stat_card(str(n), "thoughts", UiFactory.ACCENT))
	stats.add_child(_stat_card(str(l), "links", UiFactory.AMBER))
	stats.add_child(_stat_card(str(g), "groups", UiFactory.MINT))

	_inspector_body.add_child(UiFactory.spacer(4))
	var groups_kicker := HBoxContainer.new()
	groups_kicker.add_theme_constant_override("separation", 8)
	_inspector_body.add_child(groups_kicker)
	var gk := UiFactory.kicker("GROUPS", UiFactory.MUTED)
	gk.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	groups_kicker.add_child(gk)
	var add_group := UiFactory.ghost_button("+ New", false, true)
	add_group.pressed.connect(_show_new_group_modal)
	groups_kicker.add_child(add_group)

	if current.get("groups", []).is_empty():
		var no_groups := UiFactory.label("No groups yet. Create one to give a cluster a name and colour.", 12, UiFactory.FAINT)
		no_groups.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		no_groups.custom_minimum_size.x = 0
		_inspector_body.add_child(no_groups)
	else:
		for group: Dictionary in current.get("groups", []):
			var row := HBoxContainer.new()
			row.add_theme_constant_override("separation", 8)
			_inspector_body.add_child(row)
			var color := SphereStore.color_from_html(String(group.color), UiFactory.MUTED)
			row.add_child(UiFactory.color_dot(color, 8))
			var name := UiFactory.label(String(group.name), 13, UiFactory.TEXT_SOFT)
			name.size_flags_horizontal = Control.SIZE_EXPAND_FILL
			name.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
			row.add_child(name)
			var count := 0
			for node: Dictionary in current.get("nodes", []):
				if node.group_id == group.id:
					count += 1
			row.add_child(UiFactory.label(str(count), 11, UiFactory.FAINT))

	_inspector_body.add_child(UiFactory.spacer(6))
	_inspector_body.add_child(UiFactory.separator(2, 7))
	_inspector_body.add_child(UiFactory.kicker("MOVE AROUND", UiFactory.MUTED))
	for line: String in [
		"Drag the globe to turn it.",
		"Drag a thought to move it across the surface.",
		"Scroll to zoom. Double-click a thought to edit.",
	]:
		var tip := UiFactory.label(line, 12, UiFactory.FAINT)
		tip.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		_inspector_body.add_child(tip)

	_inspector_body.add_child(UiFactory.spacer(8))
	var add := UiFactory.button("+  New thought", true)
	add.pressed.connect(_add_thought)
	_inspector_body.add_child(add)


func _stat_card(value: String, label: String, color: Color) -> Control:
	var panel := PanelContainer.new()
	panel.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	var style := UiFactory.panel_style(Color("#0b1423"), Color("#1d2d45"), 10)
	style.content_margin_left = 9
	style.content_margin_right = 9
	style.content_margin_top = 8
	style.content_margin_bottom = 8
	panel.add_theme_stylebox_override("panel", style)
	var box := VBoxContainer.new()
	box.add_theme_constant_override("separation", 0)
	panel.add_child(box)
	box.add_child(UiFactory.label(value, 20, color))
	box.add_child(UiFactory.label(label, 10, UiFactory.MUTED))
	return panel


func _build_inspector_node(node: Dictionary) -> void:
	var color := SphereStore.group_color(current, String(node.group_id))
	var heading := HBoxContainer.new()
	heading.add_theme_constant_override("separation", 8)
	_inspector_body.add_child(heading)
	heading.add_child(UiFactory.color_dot(color, 9))
	var thought_label := UiFactory.kicker("THOUGHT", color.lightened(0.25))
	thought_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	heading.add_child(thought_label)
	_title_edit = UiFactory.line_edit("Name this thought", 17)
	_title_edit.text = String(node.title)
	_title_edit.custom_minimum_size.y = 46
	_title_edit.text_changed.connect(_on_title_changed)
	_title_edit.focus_exited.connect(_on_title_focus_exited)
	_inspector_body.add_child(_title_edit)

	_inspector_body.add_child(UiFactory.kicker("NOTES", UiFactory.MUTED))
	_body_edit = UiFactory.text_edit("What is this thought about?")
	_body_edit.text = String(node.body)
	_body_edit.custom_minimum_size.y = 156
	_body_edit.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_body_edit.text_changed.connect(_on_body_changed)
	_inspector_body.add_child(_body_edit)

	_inspector_body.add_child(UiFactory.spacer(3))
	var group_header := HBoxContainer.new()
	group_header.add_theme_constant_override("separation", 8)
	_inspector_body.add_child(group_header)
	var gk := UiFactory.kicker("GROUP", UiFactory.MUTED)
	gk.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	group_header.add_child(gk)
	var new_group := UiFactory.ghost_button("+ New group", false, true)
	new_group.pressed.connect(_show_new_group_modal)
	group_header.add_child(new_group)

	_group_option = UiFactory.option_button()
	_group_option.add_item("Unfiled")
	_group_option.set_item_metadata(0, "")
	_group_option.select(0)
	var selected_index := 0
	var index := 1
	for group: Dictionary in current.get("groups", []):
		_group_option.add_item(String(group.name))
		_group_option.set_item_metadata(index, String(group.id))
		if String(group.id) == String(node.group_id):
			selected_index = index
		index += 1
	_group_option.select(selected_index)
	_group_option.item_selected.connect(_on_group_selected)
	_inspector_body.add_child(_group_option)

	_inspector_body.add_child(UiFactory.separator(4, 8))
	var links_header := HBoxContainer.new()
	links_header.add_theme_constant_override("separation", 8)
	_inspector_body.add_child(links_header)
	var link_ids := SphereStore.linked_node_ids(current, String(node.id))
	var lk := UiFactory.kicker("CONNECTIONS  ·  %d" % link_ids.size(), UiFactory.MUTED)
	lk.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	links_header.add_child(lk)

	if link_ids.is_empty():
		var empty := UiFactory.label("No connections yet. Link this thought to something it leads into.", 12, UiFactory.FAINT)
		empty.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		_inspector_body.add_child(empty)
	else:
		for other_id in link_ids:
			var other := SphereStore.find_node(current, other_id)
			if other.is_empty():
				continue
			var row := HBoxContainer.new()
			row.add_theme_constant_override("separation", 6)
			_inspector_body.add_child(row)
			row.add_child(UiFactory.color_dot(SphereStore.group_color(current, String(other.group_id)), 7))
			var name := UiFactory.ghost_button(String(other.title), false, true)
			name.alignment = HORIZONTAL_ALIGNMENT_LEFT
			name.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
			name.size_flags_horizontal = Control.SIZE_EXPAND_FILL
			name.tooltip_text = "Select this thought"
			name.pressed.connect(_on_linked_thought_pressed.bind(other_id))
			row.add_child(name)
			var remove := UiFactory.ghost_button("x", true, true)
			remove.custom_minimum_size = Vector2(28, 26)
			remove.tooltip_text = "Remove connection"
			remove.pressed.connect(_remove_link.bind(String(node.id), other_id))
			row.add_child(remove)

	_link_option = UiFactory.option_button()
	_link_option.add_item("Link to another thought...")
	_link_option.select(0)
	var opt_index := 1
	for other: Dictionary in current.get("nodes", []):
		if String(other.id) == String(node.id) or link_ids.has(String(other.id)):
			continue
		_link_option.add_item(String(other.title))
		_link_option.set_item_metadata(opt_index, String(other.id))
		opt_index += 1
	if opt_index > 1:
		_link_option.item_selected.connect(_on_link_choice_selected)
		_inspector_body.add_child(_link_option)

	_inspector_body.add_child(UiFactory.spacer(8))
	var show := UiFactory.ghost_button("Show on sphere", false)
	show.pressed.connect(_show_selected_on_sphere)
	_inspector_body.add_child(show)
	var delete := UiFactory.ghost_button("Delete thought", true)
	delete.pressed.connect(_confirm_delete_thought)
	_inspector_body.add_child(delete)


func _on_title_changed(new_text: String) -> void:
	if _suppress_inspector_edits or selected_node_id.is_empty():
		return
	var node := SphereStore.find_node(current, selected_node_id)
	if node.is_empty():
		return
	node.title = new_text
	sphere_view.update_node(selected_node_id)
	board_view.update_node(selected_node_id)
	_mark_dirty()


func _on_title_focus_exited() -> void:
	if _suppress_inspector_edits or selected_node_id.is_empty():
		return
	var node := SphereStore.find_node(current, selected_node_id)
	if node.is_empty():
		return
	var value := String(node.title).strip_edges()
	if value.is_empty():
		node.title = "Untitled thought"
		if _title_edit != null:
			_title_edit.text = node.title
		sphere_view.update_node(selected_node_id)
		board_view.update_node(selected_node_id)
		_mark_dirty()


func _on_body_changed() -> void:
	if _suppress_inspector_edits or selected_node_id.is_empty() or _body_edit == null:
		return
	var node := SphereStore.find_node(current, selected_node_id)
	if node.is_empty():
		return
	node.body = _body_edit.text
	board_view.update_node(selected_node_id)
	_mark_dirty()


func _on_group_selected(index: int) -> void:
	if _suppress_inspector_edits or _group_option == null or selected_node_id.is_empty():
		return
	var group_id := String(_group_option.get_item_metadata(index))
	var node := SphereStore.find_node(current, selected_node_id)
	if node.is_empty():
		return
	node.group_id = group_id
	sphere_view.update_node(selected_node_id)
	board_view.set_data(current, selected_node_id)
	_refresh_inspector()
	_refresh_top_bar()
	_refresh_empty_state()
	_mark_dirty(true)


func _on_link_choice_selected(index: int) -> void:
	if _suppress_inspector_edits or _link_option == null or selected_node_id.is_empty() or index <= 0:
		return
	var other_id := String(_link_option.get_item_metadata(index))
	if other_id.is_empty():
		return
	if SphereStore.add_link(current, selected_node_id, other_id):
		sphere_view.rebuild_derived()
		board_view.set_data(current, selected_node_id)
		_refresh_inspector()
		_refresh_top_bar()
		_mark_dirty(true)
		_toast("Thoughts connected.", UiFactory.ACCENT)


func _remove_link(first_id: String, second_id: String) -> void:
	SphereStore.remove_link(current, first_id, second_id)
	sphere_view.rebuild_derived()
	board_view.set_data(current, selected_node_id)
	_refresh_inspector()
	_refresh_top_bar()
	_mark_dirty(true)


func _on_linked_thought_pressed(node_id: String) -> void:
	_select_node(node_id)
	if view_mode == MODE_SPHERE:
		sphere_view.focus_node(node_id)


func _show_selected_on_sphere() -> void:
	if selected_node_id.is_empty():
		return
	_set_mode(MODE_SPHERE)
	sphere_view.focus_node(selected_node_id)
	sphere_view.set_selected(selected_node_id)


func _on_node_selected(node_id: String) -> void:
	_select_node(node_id)


func _on_board_node_selected(node_id: String) -> void:
	_select_node(node_id)


func _on_node_activated(node_id: String) -> void:
	_select_node(node_id)
	await get_tree().process_frame
	_focus_inspector_title()


func _on_node_moved(_node_id: String) -> void:
	_mark_dirty()


func _on_camera_changed(camera: Dictionary) -> void:
	if current.is_empty():
		return
	current.camera = camera
	_mark_dirty()


func _select_node(node_id: String) -> void:
	selected_node_id = node_id
	sphere_view.set_selected(node_id)
	board_view.set_selected(node_id)
	_refresh_inspector()


func _clear_selection() -> void:
	selected_node_id = ""
	sphere_view.set_selected("")
	board_view.set_selected("")
	_refresh_inspector()


func _focus_inspector_title() -> void:
	if _title_edit == null or not is_instance_valid(_title_edit):
		return
	_title_edit.grab_focus()
	_title_edit.select_all()


func _add_thought() -> void:
	if current.is_empty():
		return
	var group_id := ""
	var selected := SphereStore.find_node(current, selected_node_id)
	if not selected.is_empty():
		group_id = String(selected.group_id)
	elif not current.get("groups", []).is_empty():
		group_id = String(current.groups[0].id)
	_add_thought_with_group(group_id)


func _on_board_add_requested(group_id: String) -> void:
	_add_thought_with_group(group_id)


func _add_thought_with_group(group_id: String) -> void:
	if current.is_empty():
		return
	var pos := sphere_view.suggested_position()
	var node := SphereStore.add_node(current, "Untitled thought", group_id, pos)
	sphere_view.set_data(current, false)
	selected_node_id = String(node.id)
	sphere_view.set_selected(selected_node_id)
	board_view.set_data(current, selected_node_id)
	_refresh_top_bar()
	_refresh_inspector()
	_refresh_empty_state()
	_refresh_sphere_list()
	_mark_dirty(true)
	_toast("Thought added. Give it a name.", UiFactory.ACCENT)
	await get_tree().process_frame
	_focus_inspector_title()


func _on_menu_pressed(id: int) -> void:
	match id:
		MENU_RENAME:
			_show_rename_sphere_modal()
		MENU_DUPLICATE:
			_duplicate_sphere()
		MENU_EXPORT:
			_show_export_dialog()
		MENU_DELETE:
			_confirm_delete_sphere()
		MENU_SHORTCUTS:
			_show_shortcuts_modal()
		MENU_ABOUT:
			_show_about_modal()


func _set_mode(mode: String) -> void:
	view_mode = MODE_SPHERE if mode == MODE_SPHERE else MODE_BOARD
	var sphere_mode := view_mode == MODE_SPHERE
	sphere_view.set_active(sphere_mode)
	_board_host.visible = not sphere_mode
	if not sphere_mode:
		board_view.set_data(current, selected_node_id)
	_refresh_empty_state()
	_style_mode_button(_mode_sphere_button, sphere_mode)
	_style_mode_button(_mode_board_button, not sphere_mode)
	if _hint_label != null:
		_hint_label.text = "Drag to orbit  ·  Scroll to zoom  ·  Drag a thought to move it" if sphere_mode else "Click a card to inspect  ·  Groups from the sphere appear as columns"
	_save_app_settings()


func _style_mode_button(button: Button, active: bool) -> void:
	if button == null:
		return
	var background := Color("#153449") if active else Color(0, 0, 0, 0)
	var border := Color("#3d7f9c") if active else Color(0, 0, 0, 0)
	button.add_theme_stylebox_override("normal", _mode_style(background, border))
	button.add_theme_stylebox_override("hover", _mode_style(Color("#1a4058") if active else Color("#182337"), border if active else Color("#263750")))
	button.add_theme_stylebox_override("pressed", _mode_style(Color("#153449"), Color("#4e9cbb")))
	button.add_theme_color_override("font_color", UiFactory.ACCENT if active else UiFactory.MUTED)
	button.add_theme_color_override("font_hover_color", UiFactory.ACCENT.lightened(0.1) if active else UiFactory.TEXT)


func _mode_style(background: Color, border: Color) -> StyleBoxFlat:
	var style := UiFactory.panel_style(background, border, 8, 0 if border.a == 0.0 else 1)
	style.content_margin_left = 9
	style.content_margin_right = 9
	style.content_margin_top = 5
	style.content_margin_bottom = 5
	return style


func _refresh_empty_state() -> void:
	if _empty_overlay == null:
		return
	var empty: bool = current.is_empty() or current.get("nodes", []).is_empty()
	_empty_overlay.visible = empty and view_mode == MODE_SPHERE


func _mark_dirty(refresh_list := false) -> void:
	dirty = true
	if refresh_list:
		_list_needs_refresh = true
	_set_save_status("Unsaved changes", UiFactory.AMBER)
	if _save_timer != null:
		_save_timer.start()


func _save_current() -> void:
	if current.is_empty():
		return
	if SphereStore.save_sphere(current):
		dirty = false
		_set_save_status("Saved just now", UiFactory.MINT)
		if _list_needs_refresh:
			_refresh_sphere_list()
			_list_needs_refresh = false
	else:
		_set_save_status("Could not save", UiFactory.CORAL)


func _set_save_status(text: String, color: Color) -> void:
	if _save_label == null:
		return
	_save_label.text = text
	_save_dot.add_theme_stylebox_override("panel", UiFactory.panel_style(color, Color(0, 0, 0, 0), 4, 0))


func _save_app_settings() -> void:
	if current.is_empty():
		return
	var settings := SphereStore.load_settings()
	settings["last_sphere"] = String(current.id)
	settings["mode"] = view_mode
	SphereStore.save_settings(settings)


func _toast(message: String, color := UiFactory.ACCENT) -> void:
	if _toast_panel == null:
		return
	_toast_label.text = message
	_toast_dot.add_theme_stylebox_override("panel", UiFactory.panel_style(color, Color(0, 0, 0, 0), 4, 0))
	if _toast_tween != null and _toast_tween.is_valid():
		_toast_tween.kill()
	_toast_panel.visible = true
	_toast_panel.modulate.a = 0.0
	_toast_tween = create_tween()
	_toast_tween.tween_property(_toast_panel, "modulate:a", 1.0, 0.16)
	_toast_timer.start()


func _hide_toast() -> void:
	if _toast_panel == null:
		return
	_toast_tween = create_tween()
	_toast_tween.tween_property(_toast_panel, "modulate:a", 0.0, 0.22)
	_toast_tween.tween_callback(func() -> void: _toast_panel.visible = false)


func _show_export_dialog() -> void:
	if current.is_empty():
		return
	if dirty:
		_save_current()
	var safe_title := String(current.title).to_lower().replace(" ", "-")
	for character: String in ["/", "\\", ":", "*", "?", "\"", "<", ">", "|"]:
		safe_title = safe_title.replace(character, "-")
	if OS.has_feature("web"):
		var text := SphereStore.export_sphere_text(current)
		JavaScriptBridge.download_buffer(text.to_utf8_buffer(), safe_title + ".json", "application/json")
		_toast("Sphere downloaded.", UiFactory.MINT)
		return
	_export_dialog.current_file = safe_title + ".json"
	_export_dialog.current_dir = OS.get_system_dir(OS.SYSTEM_DIR_DOCUMENTS)
	_export_dialog.popup_centered_ratio(0.68)


func _on_export_selected(path: String) -> void:
	if not path.ends_with(".json"):
		path += ".json"
	if SphereStore.export_sphere(current, path):
		_toast("Sphere exported to %s" % path.get_file(), UiFactory.MINT)
	else:
		_toast("Export failed.", UiFactory.CORAL)


func _show_import_dialog() -> void:
	if OS.has_feature("web"):
		_show_web_import_modal()
		return
	_import_dialog.current_dir = OS.get_system_dir(OS.SYSTEM_DIR_DOCUMENTS)
	_import_dialog.popup_centered_ratio(0.68)


func _on_import_selected(path: String) -> void:
	_finish_import(SphereStore.import_sphere(path))


func _on_files_dropped(files: PackedStringArray) -> void:
	for path in files:
		if path.get_extension().to_lower() == "json":
			_finish_import(SphereStore.import_sphere(path))
			return
	_toast("Drop a .json sphere file to import it.", UiFactory.AMBER)


func _setup_web_import() -> void:
	if not OS.has_feature("web"):
		return
	_web_import_callback = JavaScriptBridge.create_callback(_on_web_import_text)
	var window := JavaScriptBridge.get_interface("window")
	window.mindsphereReceiveImport = _web_import_callback
	JavaScriptBridge.eval(WebImportBridge.JAVASCRIPT, true)


func _on_web_import_text(args: Array) -> void:
	if args.is_empty():
		return
	_finish_import(SphereStore.import_sphere_text(String(args[0])))


func _show_web_import_modal() -> void:
	if JavaScriptBridge.eval("typeof window.mindsphereOpenImport === 'function'", true):
		JavaScriptBridge.eval("window.mindsphereOpenImport()", true)
	else:
		_toast("Import is not available in this browser. Reload and try again.", UiFactory.CORAL)


func _finish_import(imported: Dictionary) -> void:
	if imported.is_empty():
		_toast("That file is not a valid sphere.", UiFactory.CORAL)
		return
	_refresh_sphere_list()
	_load_sphere(String(imported.id))
	_toast("Sphere imported. A new local copy was created.", UiFactory.MINT)


func _show_new_sphere_modal() -> void:
	_show_text_modal(
		"New sphere",
		"Give this sphere a question or a working title. You can rename it at any time.",
		"",
		"e.g. Next year's direction",
		"Create sphere",
		"New sphere",
		func(value: String) -> void:
			var sphere := SphereStore.create_blank(value)
			SphereStore.save_sphere(sphere)
			_refresh_sphere_list()
			_load_sphere(String(sphere.id))
			_toast("New sphere created.", UiFactory.MINT)
	)


func _show_rename_sphere_modal() -> void:
	_show_text_modal(
		"Rename sphere",
		"",
		String(current.get("title", "")),
		"Sphere title",
		"Save name",
		"Untitled sphere",
		func(value: String) -> void:
			current.title = value
			_refresh_top_bar()
			_refresh_sphere_list()
			_mark_dirty(true)
			_toast("Sphere renamed.", UiFactory.MINT)
	)


func _show_new_group_modal() -> void:
	_show_text_modal(
		"New group",
		"Groups give a cluster of thoughts a shared name and colour. They also become columns on the 2D board.",
		"",
		"e.g. Questions to answer",
		"Create group",
		"New group",
		func(value: String) -> void:
			var group := SphereStore.add_group(current, value)
			var node := SphereStore.find_node(current, selected_node_id)
			if not node.is_empty():
				node.group_id = String(group.id)
				sphere_view.update_node(selected_node_id)
			sphere_view.rebuild_derived()
			board_view.set_data(current, selected_node_id)
			_refresh_inspector()
			_refresh_top_bar()
			_mark_dirty(true)
			_toast("Group created.", UiFactory.MINT)
	)


func _duplicate_sphere() -> void:
	if current.is_empty():
		return
	if dirty:
		_save_current()
	var copy := SphereStore.duplicate_sphere(current)
	if copy.is_empty():
		_toast("Could not duplicate this sphere.", UiFactory.CORAL)
		return
	_refresh_sphere_list()
	_load_sphere(String(copy.id))
	_toast("Sphere duplicated.", UiFactory.MINT)


func _confirm_delete_thought() -> void:
	if selected_node_id.is_empty():
		return
	var node := SphereStore.find_node(current, selected_node_id)
	if node.is_empty():
		return
	_show_confirm_modal(
		"Delete thought?",
		"“%s” will be removed from the sphere, along with its connections. This cannot be undone." % String(node.title),
		"Delete thought",
		func() -> void:
			SphereStore.remove_node(current, selected_node_id)
			selected_node_id = ""
			sphere_view.set_data(current, false)
			sphere_view.set_selected("")
			board_view.set_data(current, "")
			_refresh_top_bar()
			_refresh_inspector()
			_refresh_empty_state()
			_refresh_sphere_list()
			_mark_dirty(true)
			_toast("Thought deleted.", UiFactory.CORAL)
	)


func _confirm_delete_sphere() -> void:
	if current.is_empty():
		return
	var title := String(current.title)
	_show_confirm_modal(
		"Delete sphere?",
		"“%s” and its local JSON file will be permanently deleted. Export it first if you want a copy." % title,
		"Delete sphere",
		func() -> void:
			var deleted_id := String(current.id)
			SphereStore.delete_sphere(deleted_id)
			dirty = false
			current = {}
			selected_node_id = ""
			_refresh_sphere_list()
			var next_id := ""
			if not spheres.is_empty():
				next_id = String(spheres[0].id)
			if next_id.is_empty():
				var blank := SphereStore.create_blank("New sphere")
				SphereStore.save_sphere(blank)
				_refresh_sphere_list()
				next_id = String(blank.id)
			_load_sphere(next_id)
			_toast("Sphere deleted.", UiFactory.CORAL)
	)


func _show_shortcuts_modal() -> void:
	_show_info_modal(
		"Keyboard shortcuts",
		"[color=#8fa0bc]WORKING[/color]\n[color=#e8eefb]N[/color]   New thought\n[color=#e8eefb]Delete[/color]   Delete selected thought\n[color=#e8eefb]Esc[/color]   Clear selection\n[color=#e8eefb]Ctrl+S[/color]   Save now\n\n[color=#8fa0bc]NAVIGATION[/color]\n[color=#e8eefb]1[/color]   Sphere mode\n[color=#e8eefb]2[/color]   Board mode\n[color=#e8eefb]Ctrl+F[/color]   Search spheres\n[color=#e8eefb]Ctrl+E[/color]   Export sphere\n\n[color=#8fa0bc]SPHERE[/color]\nDrag background to orbit. Scroll to zoom.\nDrag a thought to move it across the surface.\nDouble-click a thought to edit its title."
	)


func _show_about_modal() -> void:
	var path := ProjectSettings.globalize_path("user://spheres")
	_show_info_modal(
		"Mindsphere",
		"A local-first, open-source sphere mind mapper built with Godot.\n\nEvery thought is a point on a sphere. Turn the sphere, move ideas around, group them, and draw paths between them.\n\n[color=#8fa0bc]Your data[/color]\n%s\n\n[color=#8fa0bc]License[/color]\nMIT. The source lives with the project." % path,
		"Open data folder",
		func() -> void:
			OS.shell_open(path)
	)


func _show_text_modal(title: String, subtitle: String, initial: String, placeholder: String, submit_label: String, fallback: String, callback: Callable) -> void:
	_prepare_modal()
	_modal_fallback = fallback
	_modal_submit_callback = callback
	var panel := _modal_panel(470)
	var box := VBoxContainer.new()
	box.add_theme_constant_override("separation", 12)
	panel.add_child(box)
	box.add_child(UiFactory.label(title, 22, UiFactory.TEXT))
	if not subtitle.is_empty():
		var sub := UiFactory.label(subtitle, 13, UiFactory.MUTED)
		sub.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		box.add_child(sub)
	_modal_line = UiFactory.line_edit(placeholder, 15)
	_modal_line.text = initial
	_modal_line.custom_minimum_size.y = 44
	_modal_line.text_submitted.connect(func(_text: String) -> void: _submit_text_modal())
	_modal_line.gui_input.connect(_on_modal_line_input)
	box.add_child(_modal_line)
	box.add_child(UiFactory.spacer(2))
	var actions := HBoxContainer.new()
	actions.add_theme_constant_override("separation", 8)
	box.add_child(actions)
	var flex := Control.new()
	flex.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	actions.add_child(flex)
	var cancel := UiFactory.ghost_button("Cancel", false)
	cancel.pressed.connect(_close_modal)
	actions.add_child(cancel)
	var submit := UiFactory.button(submit_label, true)
	submit.pressed.connect(_submit_text_modal)
	actions.add_child(submit)
	_modal_line.call_deferred("grab_focus")
	if not initial.is_empty():
		_modal_line.call_deferred("select_all")


func _show_confirm_modal(title: String, message: String, confirm_label: String, callback: Callable) -> void:
	_prepare_modal()
	_modal_confirm_callback = callback
	var panel := _modal_panel(470)
	var box := VBoxContainer.new()
	box.add_theme_constant_override("separation", 12)
	panel.add_child(box)
	box.add_child(UiFactory.kicker("PLEASE CONFIRM", UiFactory.CORAL))
	box.add_child(UiFactory.label(title, 22, UiFactory.TEXT))
	var text := UiFactory.label(message, 13, UiFactory.MUTED)
	text.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	box.add_child(text)
	box.add_child(UiFactory.spacer(4))
	var actions := HBoxContainer.new()
	actions.add_theme_constant_override("separation", 8)
	box.add_child(actions)
	var flex := Control.new()
	flex.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	actions.add_child(flex)
	var cancel := UiFactory.ghost_button("Cancel", false)
	cancel.pressed.connect(_close_modal)
	actions.add_child(cancel)
	var confirm := UiFactory.button(confirm_label, true)
	var normal := UiFactory.panel_style(Color("#e56c6c"), Color("#ff9999"), 10)
	normal.content_margin_left = 16
	normal.content_margin_right = 16
	normal.content_margin_top = 8
	normal.content_margin_bottom = 8
	var hover := normal.duplicate() as StyleBoxFlat
	hover.bg_color = Color("#f27f7f")
	confirm.add_theme_stylebox_override("normal", normal)
	confirm.add_theme_stylebox_override("hover", hover)
	confirm.add_theme_stylebox_override("pressed", normal)
	confirm.add_theme_color_override("font_color", Color("#190607"))
	confirm.add_theme_color_override("font_hover_color", Color("#120304"))
	confirm.add_theme_color_override("font_pressed_color", Color("#190607"))
	confirm.add_theme_color_override("font_focus_color", Color("#190607"))
	confirm.pressed.connect(_submit_confirm_modal)
	actions.add_child(confirm)


func _show_info_modal(title: String, bbcode: String, action_label := "", action := Callable()) -> void:
	_prepare_modal()
	var panel := _modal_panel(520)
	var box := VBoxContainer.new()
	box.add_theme_constant_override("separation", 12)
	panel.add_child(box)
	box.add_child(UiFactory.label(title, 23, UiFactory.TEXT))
	var rich := RichTextLabel.new()
	rich.bbcode_enabled = true
	rich.fit_content = true
	rich.scroll_active = false
	rich.custom_minimum_size.y = 160
	rich.add_theme_font_size_override("normal_font_size", 13)
	rich.add_theme_color_override("default_color", UiFactory.TEXT_SOFT)
	rich.text = bbcode
	box.add_child(rich)
	box.add_child(UiFactory.spacer(2))
	var actions := HBoxContainer.new()
	actions.add_theme_constant_override("separation", 8)
	box.add_child(actions)
	if action.is_valid() and not action_label.is_empty():
		var action_button := UiFactory.ghost_button(action_label, false)
		action_button.pressed.connect(action)
		actions.add_child(action_button)
	var flex := Control.new()
	flex.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	actions.add_child(flex)
	var close := UiFactory.button("Close", true)
	close.pressed.connect(_close_modal)
	actions.add_child(close)


func _prepare_modal() -> void:
	_close_modal()
	_modal_layer.visible = true
	var shade := ColorRect.new()
	shade.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	shade.color = Color(0.005, 0.008, 0.015, 0.68)
	shade.mouse_filter = Control.MOUSE_FILTER_STOP
	_modal_layer.add_child(shade)
	var center := CenterContainer.new()
	center.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	center.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_modal_layer.add_child(center)
	_modal_center = center
	var panel := PanelContainer.new()
	panel.custom_minimum_size = Vector2(470, 0)
	var style := UiFactory.panel_style(Color("#101827"), Color("#304663"), 18)
	style.content_margin_left = 26
	style.content_margin_right = 26
	style.content_margin_top = 24
	style.content_margin_bottom = 22
	style.shadow_color = Color(0, 0, 0, 0.6)
	style.shadow_size = 28
	panel.add_theme_stylebox_override("panel", style)
	center.add_child(panel)


func _modal_panel(width: float) -> VBoxContainer:
	if _modal_center == null or _modal_center.get_child_count() == 0:
		return VBoxContainer.new()
	var panel := _modal_center.get_child(0) as PanelContainer
	panel.custom_minimum_size.x = width
	var box := VBoxContainer.new()
	box.add_theme_constant_override("separation", 12)
	panel.add_child(box)
	return box


func _on_modal_line_input(event: InputEvent) -> void:
	if event is InputEventKey:
		var key := event as InputEventKey
		if key.pressed and key.keycode == KEY_ESCAPE:
			_close_modal()
			if _modal_line != null:
				_modal_line.accept_event()


func _submit_text_modal() -> void:
	if _modal_line == null:
		return
	var value := _modal_line.text.strip_edges()
	if value.is_empty():
		value = _modal_fallback
	var callback := _modal_submit_callback
	_close_modal()
	if callback.is_valid():
		callback.call(value)


func _submit_confirm_modal() -> void:
	var callback := _modal_confirm_callback
	_close_modal()
	if callback.is_valid():
		callback.call()


func _close_modal() -> void:
	if _modal_layer == null:
		return
	_modal_layer.visible = false
	for child in _modal_layer.get_children():
		_modal_layer.remove_child(child)
		child.queue_free()
	_modal_center = null
	_modal_line = null
	_modal_submit_callback = Callable()
	_modal_confirm_callback = Callable()


func _unhandled_key_input(event: InputEvent) -> void:
	if not (event is InputEventKey):
		return
	var key := event as InputEventKey
	if not key.pressed or key.echo:
		return
	if _modal_layer != null and _modal_layer.visible:
		if key.keycode == KEY_ESCAPE:
			_close_modal()
			get_viewport().set_input_as_handled()
		return
	var focus := get_viewport().gui_get_focus_owner()
	var editing := focus is LineEdit or focus is TextEdit
	if key.ctrl_pressed:
		match key.keycode:
			KEY_S:
				_save_current()
				_toast("Saved to local storage.", UiFactory.MINT)
				get_viewport().set_input_as_handled()
			KEY_F:
				_sphere_search.grab_focus()
				_sphere_search.select_all()
				get_viewport().set_input_as_handled()
			KEY_E:
				_show_export_dialog()
				get_viewport().set_input_as_handled()
		return
	if editing:
		return
	match key.keycode:
		KEY_N:
			_add_thought()
			get_viewport().set_input_as_handled()
		KEY_1:
			_set_mode(MODE_SPHERE)
			get_viewport().set_input_as_handled()
		KEY_2:
			_set_mode(MODE_BOARD)
			get_viewport().set_input_as_handled()
		KEY_DELETE, KEY_BACKSPACE:
			if not selected_node_id.is_empty():
				_confirm_delete_thought()
			get_viewport().set_input_as_handled()
		KEY_ESCAPE:
			_clear_selection()
			get_viewport().set_input_as_handled()
