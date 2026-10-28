class_name BoardView
extends Control

signal node_selected(node_id: String)
signal node_activated(node_id: String)
signal add_requested(group_id: String)
signal background_clicked

var data: Dictionary = {}
var selected_id := ""

var _scroll: ScrollContainer
var _columns: HBoxContainer
var _count_label: Label
var _cards: Dictionary = {}
var _scroll_x := 0
var _scroll_y := 0


func _ready() -> void:
	mouse_filter = Control.MOUSE_FILTER_PASS
	resized.connect(_redraw)
	_build_shell()


func set_data(sphere: Dictionary, selection := "") -> void:
	data = sphere
	selected_id = selection
	if _scroll == null:
		return
	_scroll_x = _scroll.scroll_horizontal
	_scroll_y = _scroll.scroll_vertical
	_rebuild()
	set_selected(selected_id)
	await get_tree().process_frame
	if is_instance_valid(_scroll):
		_scroll.scroll_horizontal = _scroll_x
		_scroll.scroll_vertical = _scroll_y


func set_selected(node_id: String) -> void:
	selected_id = node_id
	for id: String in _cards:
		var card: PanelContainer = _cards[id].panel
		card.add_theme_stylebox_override("panel", _card_style(id == selected_id, false))


func update_node(node_id: String) -> void:
	if not _cards.has(node_id):
		return
	var node := SphereStore.find_node(data, node_id)
	if node.is_empty():
		return
	var entry: Dictionary = _cards[node_id]
	var title: Label = entry.title
	title.text = String(node.title)
	var body: Label = entry.body
	body.text = _excerpt(String(node.body))
	body.visible = not String(node.body).strip_edges().is_empty()


func _build_shell() -> void:
	var root := VBoxContainer.new()
	root.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	root.add_theme_constant_override("separation", 16)
	root.mouse_filter = Control.MOUSE_FILTER_PASS
	var pad := UiFactory.margin(root, 28, 24, 28, 92)
	pad.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	add_child(pad)

	var header := HBoxContainer.new()
	header.add_theme_constant_override("separation", 12)
	root.add_child(header)
	var titles := VBoxContainer.new()
	titles.add_theme_constant_override("separation", 2)
	titles.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	header.add_child(titles)
	titles.add_child(UiFactory.kicker("2D BOARD", UiFactory.ACCENT))
	var heading := UiFactory.label("The same thoughts, laid flat", 23, UiFactory.TEXT)
	titles.add_child(heading)
	_count_label = UiFactory.label("", 13, UiFactory.MUTED)
	_count_label.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	header.add_child(_count_label)

	var scroll := ScrollContainer.new()
	scroll.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_AUTO
	scroll.vertical_scroll_mode = ScrollContainer.SCROLL_MODE_AUTO
	scroll.add_theme_constant_override("scrollbar_margin_bottom", 8)
	root.add_child(scroll)
	_scroll = scroll

	_columns = HBoxContainer.new()
	_columns.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_columns.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_columns.add_theme_constant_override("separation", 16)
	scroll.add_child(_columns)


func _rebuild() -> void:
	if _columns == null:
		return
	for child in _columns.get_children():
		child.queue_free()
	_cards.clear()
	var nodes: Array = data.get("nodes", [])
	if _count_label != null:
		_count_label.text = "%d thoughts  ·  %d groups" % [nodes.size(), data.get("groups", []).size()]
	if nodes.is_empty():
		_add_empty_column()
		return
	for group: Dictionary in data.get("groups", []):
		_add_group_column(String(group.id), String(group.name), SphereStore.color_from_html(String(group.color), UiFactory.MUTED))
	var unfiled: Array[Dictionary] = []
	for node: Dictionary in nodes:
		if not _group_exists(String(node.group_id)):
			unfiled.append(node)
	if not unfiled.is_empty():
		_add_group_column("", "Unfiled", UiFactory.MUTED, unfiled)


func _add_group_column(group_id: String, group_name: String, color: Color, explicit_nodes: Array = []) -> void:
	var panel := PanelContainer.new()
	panel.custom_minimum_size = Vector2(302, 0)
	panel.size_flags_vertical = Control.SIZE_EXPAND_FILL
	var style := UiFactory.panel_style(Color("#0c1322"), Color(color.r, color.g, color.b, 0.22), 16, 1)
	style.content_margin_left = 12
	style.content_margin_right = 12
	style.content_margin_top = 12
	style.content_margin_bottom = 12
	panel.add_theme_stylebox_override("panel", style)
	_columns.add_child(panel)

	var box := VBoxContainer.new()
	box.add_theme_constant_override("separation", 10)
	panel.add_child(box)

	var header := HBoxContainer.new()
	header.add_theme_constant_override("separation", 8)
	box.add_child(header)
	header.add_child(UiFactory.color_dot(color, 9))
	var name_label := UiFactory.label(group_name, 15, UiFactory.TEXT)
	name_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	header.add_child(name_label)
	var nodes_in_group: Array = []
	if explicit_nodes.is_empty() and not group_id.is_empty():
		for node: Dictionary in data.get("nodes", []):
			if String(node.group_id) == group_id:
				nodes_in_group.append(node)
	else:
		nodes_in_group = explicit_nodes
	header.add_child(UiFactory.label(str(nodes_in_group.size()), 12, UiFactory.FAINT))

	var accent := ColorRect.new()
	accent.color = Color(color.r, color.g, color.b, 0.55)
	accent.custom_minimum_size = Vector2(0, 2)
	accent.mouse_filter = Control.MOUSE_FILTER_IGNORE
	box.add_child(accent)

	if nodes_in_group.is_empty():
		var empty := UiFactory.label("Nothing here yet.", 13, UiFactory.FAINT)
		empty.add_theme_constant_override("line_spacing", 2)
		box.add_child(UiFactory.margin(empty, 4, 8, 4, 2))
	for node: Dictionary in nodes_in_group:
		_add_card(box, node, color)

	var add := UiFactory.ghost_button("+  Add thought", false, true)
	add.alignment = HORIZONTAL_ALIGNMENT_LEFT
	add.pressed.connect(func() -> void: add_requested.emit(group_id))
	box.add_child(add)
	var tail := Control.new()
	tail.size_flags_vertical = Control.SIZE_EXPAND_FILL
	tail.mouse_filter = Control.MOUSE_FILTER_IGNORE
	box.add_child(tail)


func _add_empty_column() -> void:
	var panel := PanelContainer.new()
	panel.custom_minimum_size = Vector2(440, 260)
	var style := UiFactory.panel_style(Color("#0c1322"), UiFactory.BORDER, 18, 1)
	panel.add_theme_stylebox_override("panel", style)
	_columns.add_child(panel)
	var center := CenterContainer.new()
	panel.add_child(center)
	var box := VBoxContainer.new()
	box.add_theme_constant_override("separation", 10)
	box.alignment = BoxContainer.ALIGNMENT_CENTER
	center.add_child(box)
	var title := UiFactory.label("No thoughts yet", 20, UiFactory.TEXT)
	title.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	box.add_child(title)
	var hint := UiFactory.label("Add a thought to begin. It will appear here and on the sphere.", 13, UiFactory.MUTED)
	hint.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	hint.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	hint.custom_minimum_size.x = 320
	box.add_child(hint)
	var add := UiFactory.button("New thought", true)
	add.pressed.connect(func() -> void: add_requested.emit(""))
	box.add_child(add)


func _add_card(parent: VBoxContainer, node: Dictionary, color: Color) -> void:
	var card := PanelContainer.new()
	card.custom_minimum_size = Vector2(0, 84)
	card.mouse_filter = Control.MOUSE_FILTER_STOP
	card.mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND
	card.add_theme_stylebox_override("panel", _card_style(false, false))
	var content := MarginContainer.new()
	content.add_theme_constant_override("margin_left", 12)
	content.add_theme_constant_override("margin_right", 12)
	content.add_theme_constant_override("margin_top", 11)
	content.add_theme_constant_override("margin_bottom", 10)
	content.mouse_filter = Control.MOUSE_FILTER_IGNORE
	card.add_child(content)
	var box := VBoxContainer.new()
	box.add_theme_constant_override("separation", 5)
	box.mouse_filter = Control.MOUSE_FILTER_IGNORE
	content.add_child(box)

	var title := UiFactory.label(String(node.title), 15, UiFactory.TEXT)
	title.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	title.mouse_filter = Control.MOUSE_FILTER_IGNORE
	box.add_child(title)

	var body := UiFactory.label(_excerpt(String(node.body)), 12, UiFactory.MUTED)
	body.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	body.max_lines_visible = 2
	body.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	body.mouse_filter = Control.MOUSE_FILTER_IGNORE
	body.visible = not String(node.body).strip_edges().is_empty()
	box.add_child(body)

	var footer := HBoxContainer.new()
	footer.add_theme_constant_override("separation", 6)
	footer.mouse_filter = Control.MOUSE_FILTER_IGNORE
	box.add_child(footer)
	var links := SphereStore.linked_node_ids(data, String(node.id)).size()
	if links > 0:
		var linked := UiFactory.label("%d link%s" % [links, "" if links == 1 else "s"], 11, UiFactory.FAINT)
		linked.mouse_filter = Control.MOUSE_FILTER_IGNORE
		footer.add_child(linked)
	var flex := Control.new()
	flex.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	flex.mouse_filter = Control.MOUSE_FILTER_IGNORE
	footer.add_child(flex)
	var dot := UiFactory.color_dot(color, 6)
	footer.add_child(dot)

	parent.add_child(card)
	card.gui_input.connect(_on_card_gui_input.bind(String(node.id), card))
	card.mouse_entered.connect(_on_card_hover.bind(card, true))
	card.mouse_exited.connect(_on_card_hover.bind(card, false))
	_cards[String(node.id)] = {"panel": card, "title": title, "body": body}


func _on_card_gui_input(event: InputEvent, node_id: String, card: PanelContainer) -> void:
	if event is InputEventMouseButton:
		var mouse := event as InputEventMouseButton
		if mouse.button_index == MOUSE_BUTTON_LEFT and mouse.pressed:
			selected_id = node_id
			set_selected(node_id)
			node_selected.emit(node_id)
			if mouse.double_click:
				node_activated.emit(node_id)
			accept_event()


func _on_card_hover(card: PanelContainer, entered: bool) -> void:
	if card == null or not is_instance_valid(card):
		return
	var id := ""
	for key: String in _cards:
		if _cards[key].panel == card:
			id = key
			break
	card.add_theme_stylebox_override("panel", _card_style(id == selected_id, entered))


func _card_style(selected: bool, hovered: bool) -> StyleBoxFlat:
	var bg := Color("#111a2b")
	var border := Color("#22314b")
	if hovered:
		bg = Color("#17243a")
		border = Color("#355074")
	if selected:
		bg = Color("#112b38")
		border = Color("#5ad2ff")
	var style := UiFactory.panel_style(bg, border, 11, 1)
	style.content_margin_left = 0
	style.content_margin_right = 0
	style.content_margin_top = 0
	style.content_margin_bottom = 0
	return style


func _group_exists(group_id: String) -> bool:
	if group_id.is_empty():
		return false
	for group: Dictionary in data.get("groups", []):
		if String(group.id) == group_id:
			return true
	return false


func _excerpt(text: String) -> String:
	var clean := text.strip_edges().replace("\n", "  ")
	if clean.length() <= 96:
		return clean
	return clean.substr(0, 93).strip_edges() + "..."


func _draw() -> void:
	draw_rect(Rect2(Vector2.ZERO, size), Color("#080d18"))
	var grid_color := Color(0.30, 0.46, 0.72, 0.035)
	var step := 56
	var x := 0.0
	while x < size.x:
		draw_line(Vector2(x, 0), Vector2(x, size.y), grid_color)
		x += step
	var y := 0.0
	while y < size.y:
		draw_line(Vector2(0, y), Vector2(size.x, y), grid_color)
		y += step


func _redraw() -> void:
	queue_redraw()
