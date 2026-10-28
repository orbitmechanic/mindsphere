class_name UiFactory
extends RefCounted

const BG := Color("#070a12")
const PANEL := Color("#0c1220")
const PANEL_RAISED := Color("#111a2b")
const PANEL_HOVER := Color("#172237")
const BORDER := Color("#202d45")
const BORDER_BRIGHT := Color("#2d405f")
const TEXT := Color("#e8eefb")
const TEXT_SOFT := Color("#c3cee2")
const MUTED := Color("#8491aa")
const FAINT := Color("#5c6982")
const ACCENT := Color("#5ad2ff")
const ACCENT_DIM := Color("#1c5c73")
const CORAL := Color("#ff7a7a")
const AMBER := Color("#ffb454")
const MINT := Color("#6ee7b7")

const GROUP_COLORS: Array[Color] = [
	Color("#5ad2ff"), Color("#ffb454"), Color("#ff7a7a"),
	Color("#6ee7b7"), Color("#b69cff"), Color("#f38bd0"),
]


static func make_theme() -> Theme:
	var theme := Theme.new()
	theme.default_font_size = 15

	theme.set_color("font_color", "Label", TEXT_SOFT)
	theme.set_color("font_color", "RichTextLabel", TEXT_SOFT)
	theme.set_color("default_color", "RichTextLabel", TEXT_SOFT)
	theme.set_constant("line_separation", "RichTextLabel", 4)

	theme.set_stylebox("panel", "PanelContainer", panel_style(PANEL, Color(0, 0, 0, 0), 16))
	theme.set_stylebox("panel", "Panel", panel_style(PANEL, Color(0, 0, 0, 0), 16))

	var normal := panel_style(PANEL_RAISED, BORDER, 10)
	normal.content_margin_left = 14
	normal.content_margin_right = 14
	normal.content_margin_top = 8
	normal.content_margin_bottom = 8
	var hover := normal.duplicate() as StyleBoxFlat
	hover.bg_color = PANEL_HOVER
	hover.border_color = BORDER_BRIGHT
	var pressed := normal.duplicate() as StyleBoxFlat
	pressed.bg_color = Color("#0e2b3a")
	pressed.border_color = ACCENT_DIM
	var focus := StyleBoxFlat.new()
	focus.bg_color = Color(0, 0, 0, 0)
	focus.border_color = Color(ACCENT.r, ACCENT.g, ACCENT.b, 0.5)
	focus.set_border_width_all(1)
	focus.set_corner_radius_all(10)
	focus.set_expand_margin_all(2)
	var disabled := normal.duplicate() as StyleBoxFlat
	disabled.bg_color = Color("#0b101b")
	disabled.border_color = Color("#161f31")

	for type in ["Button", "OptionButton", "MenuButton", "CheckButton"]:
		theme.set_stylebox("normal", type, normal)
		theme.set_stylebox("hover", type, hover)
		theme.set_stylebox("pressed", type, pressed)
		theme.set_stylebox("focus", type, focus)
		theme.set_stylebox("disabled", type, disabled)
		theme.set_color("font_color", type, TEXT_SOFT)
		theme.set_color("font_hover_color", type, TEXT)
		theme.set_color("font_pressed_color", type, Color("#e9fbff"))
		theme.set_color("font_focus_color", type, TEXT)
		theme.set_color("font_disabled_color", type, FAINT)

	var field := panel_style(Color("#090f1a"), Color("#1d293f"), 10)
	field.content_margin_left = 12
	field.content_margin_right = 12
	field.content_margin_top = 8
	field.content_margin_bottom = 8
	var field_focus := field.duplicate() as StyleBoxFlat
	field_focus.border_color = Color(ACCENT.r, ACCENT.g, ACCENT.b, 0.7)
	theme.set_stylebox("normal", "LineEdit", field)
	theme.set_stylebox("focus", "LineEdit", field_focus)
	theme.set_stylebox("read_only", "LineEdit", field)
	theme.set_color("font_color", "LineEdit", TEXT)
	theme.set_color("font_placeholder_color", "LineEdit", FAINT)
	theme.set_color("caret_color", "LineEdit", ACCENT)
	theme.set_color("selection_color", "LineEdit", Color(ACCENT.r, ACCENT.g, ACCENT.b, 0.26))

	theme.set_stylebox("normal", "TextEdit", field)
	theme.set_stylebox("focus", "TextEdit", field_focus)
	theme.set_stylebox("read_only", "TextEdit", field)
	theme.set_color("font_color", "TextEdit", TEXT_SOFT)
	theme.set_color("font_placeholder_color", "TextEdit", FAINT)
	theme.set_color("caret_color", "TextEdit", ACCENT)
	theme.set_color("selection_color", "TextEdit", Color(ACCENT.r, ACCENT.g, ACCENT.b, 0.26))
	theme.set_stylebox("panel", "TextEdit", field)

	var popup := panel_style(Color("#101827"), BORDER_BRIGHT, 10)
	popup.content_margin_left = 6
	popup.content_margin_right = 6
	popup.content_margin_top = 6
	popup.content_margin_bottom = 6
	theme.set_stylebox("panel", "PopupMenu", popup)
	theme.set_color("font_color", "PopupMenu", TEXT_SOFT)
	theme.set_color("font_hover_color", "PopupMenu", TEXT)
	theme.set_color("font_accelerator_color", "PopupMenu", FAINT)
	theme.set_stylebox("hover", "PopupMenu", panel_style(Color("#172237"), Color(0, 0, 0, 0), 7))
	theme.set_constant("item_start_padding", "PopupMenu", 10)
	theme.set_constant("item_end_padding", "PopupMenu", 10)

	theme.set_stylebox("scroll", "VScrollBar", panel_style(Color(0, 0, 0, 0), Color(0, 0, 0, 0), 4))
	theme.set_stylebox("grabber", "VScrollBar", panel_style(Color("#2a3854"), Color(0, 0, 0, 0), 4))
	theme.set_stylebox("grabber_highlight", "VScrollBar", panel_style(Color("#3b5076"), Color(0, 0, 0, 0), 4))
	theme.set_stylebox("grabber_pressed", "VScrollBar", panel_style(ACCENT_DIM, Color(0, 0, 0, 0), 4))

	theme.set_stylebox("panel", "TooltipPanel", panel_style(Color("#111a2b"), BORDER_BRIGHT, 8))
	theme.set_color("font_color", "TooltipLabel", TEXT_SOFT)

	theme.set_stylebox("separator", "HSeparator", line_style(Color("#1c273d")))
	theme.set_constant("separation", "HSeparator", 12)
	return theme


static func panel_style(bg: Color, border := BORDER, radius := 14, border_width := 1) -> StyleBoxFlat:
	var style := StyleBoxFlat.new()
	style.bg_color = bg
	style.border_color = border
	style.set_border_width_all(border_width)
	style.set_corner_radius_all(radius)
	style.anti_aliasing = true
	return style


static func line_style(color: Color, height := 1) -> StyleBoxLine:
	var style := StyleBoxLine.new()
	style.color = color
	style.thickness = height
	return style


static func label(text: String, size := 15, color := TEXT_SOFT) -> Label:
	var node := Label.new()
	node.text = text
	node.add_theme_font_size_override("font_size", size)
	node.add_theme_color_override("font_color", color)
	return node


static func kicker(text: String, color := MUTED) -> Label:
	var node := label(text.to_upper(), 11, color)
	node.add_theme_constant_override("outline_size", 0)
	node.add_theme_font_size_override("font_size", 11)
	return node


static func button(text: String, primary := false, compact := false) -> Button:
	var node := Button.new()
	node.text = text
	node.focus_mode = Control.FOCUS_ALL
	node.mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND
	node.add_theme_font_size_override("font_size", 13 if compact else 14)
	node.custom_minimum_size.y = 32.0 if compact else 40.0
	if compact:
		node.add_theme_stylebox_override("normal", _compact_style(PANEL_RAISED, BORDER, 9))
		node.add_theme_stylebox_override("hover", _compact_style(PANEL_HOVER, BORDER_BRIGHT, 9))
		node.add_theme_stylebox_override("pressed", _compact_style(Color("#0e2b3a"), ACCENT_DIM, 9))
	if primary:
		node.add_theme_stylebox_override("normal", _primary_style(false))
		node.add_theme_stylebox_override("hover", _primary_style(true))
		node.add_theme_stylebox_override("pressed", _primary_style(false))
		node.add_theme_color_override("font_color", Color("#061016"))
		node.add_theme_color_override("font_hover_color", Color("#02080b"))
		node.add_theme_color_override("font_pressed_color", Color("#061016"))
		node.add_theme_color_override("font_focus_color", Color("#061016"))
	return node


static func ghost_button(text: String, danger := false, compact := false) -> Button:
	var node := button(text, false, compact)
	node.add_theme_stylebox_override("normal", _compact_style(Color(0, 0, 0, 0), Color(0, 0, 0, 0), 9))
	node.add_theme_stylebox_override("hover", _compact_style(Color(CORAL.r, CORAL.g, CORAL.b, 0.10) if danger else PANEL_HOVER, Color(CORAL.r, CORAL.g, CORAL.b, 0.25) if danger else BORDER, 9))
	node.add_theme_color_override("font_color", CORAL if danger else MUTED)
	node.add_theme_color_override("font_hover_color", CORAL.lightened(0.15) if danger else TEXT)
	return node


static func line_edit(placeholder := "", size := 14) -> LineEdit:
	var node := LineEdit.new()
	node.placeholder_text = placeholder
	node.add_theme_font_size_override("font_size", size)
	node.caret_blink = true
	return node


static func text_edit(placeholder := "") -> TextEdit:
	var node := TextEdit.new()
	node.placeholder_text = placeholder
	node.wrap_mode = TextEdit.LINE_WRAPPING_BOUNDARY
	node.scroll_fit_content_height = false
	node.add_theme_font_size_override("font_size", 14)
	return node


static func option_button() -> OptionButton:
	var node := OptionButton.new()
	node.add_theme_font_size_override("font_size", 13)
	node.fit_to_longest_item = false
	node.custom_minimum_size.y = 38
	return node


static func separator(margin_top := 0, margin_bottom := 0) -> Control:
	var wrapper := MarginContainer.new()
	wrapper.add_theme_constant_override("margin_top", margin_top)
	wrapper.add_theme_constant_override("margin_bottom", margin_bottom)
	var sep := HSeparator.new()
	wrapper.add_child(sep)
	return wrapper


static func spacer(height: float) -> Control:
	var spacer := Control.new()
	spacer.custom_minimum_size.y = height
	spacer.mouse_filter = Control.MOUSE_FILTER_IGNORE
	return spacer


static func margin(child: Control, left: int, top: int, right: int, bottom: int) -> MarginContainer:
	var wrapper := MarginContainer.new()
	wrapper.add_theme_constant_override("margin_left", left)
	wrapper.add_theme_constant_override("margin_top", top)
	wrapper.add_theme_constant_override("margin_right", right)
	wrapper.add_theme_constant_override("margin_bottom", bottom)
	wrapper.add_child(child)
	return wrapper


static func color_dot(color: Color, size := 9) -> Panel:
	var dot := Panel.new()
	dot.custom_minimum_size = Vector2(size, size)
	dot.mouse_filter = Control.MOUSE_FILTER_IGNORE
	var style := panel_style(color, Color(0, 0, 0, 0), int(size * 0.5), 0)
	dot.add_theme_stylebox_override("panel", style)
	return dot


static func _primary_style(hover: bool) -> StyleBoxFlat:
	var style := panel_style(Color("#7adfff") if hover else Color("#59cbe9"), Color("#b7f3ff") if hover else Color("#8eeaff"), 10)
	style.content_margin_left = 16
	style.content_margin_right = 16
	style.content_margin_top = 8
	style.content_margin_bottom = 8
	style.shadow_color = Color(0.20, 0.75, 0.95, 0.25)
	style.shadow_size = 10
	return style


static func _compact_style(bg: Color, border: Color, radius: int) -> StyleBoxFlat:
	var style := panel_style(bg, border, radius)
	style.content_margin_left = 10
	style.content_margin_right = 10
	style.content_margin_top = 5
	style.content_margin_bottom = 5
	return style
