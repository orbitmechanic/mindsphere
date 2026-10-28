class_name SphereView
extends Node3D

signal node_selected(node_id: String)
signal node_activated(node_id: String)
signal node_moved(node_id: String)
signal background_clicked
signal camera_changed(camera: Dictionary)

const GLOBE_RADIUS := 2.9
const NODE_RADIUS := 0.155
const BACKGROUND_SHADER := preload("res://shaders/background.gdshader")
const GLOBE_SHADER := preload("res://shaders/globe.gdshader")
const ATMOSPHERE_SHADER := preload("res://shaders/atmosphere.gdshader")

var data: Dictionary = {}
var selected_id := ""

var _pivot: Node3D
var _camera: Camera3D
var _node_layer: Node3D
var _link_layer: Node3D
var _group_layer: Node3D
var _label_canvas: CanvasLayer
var _label_overlay: Control
var _node_visuals: Dictionary = {}

var _target_yaw := 0.0
var _target_pitch := 8.0
var _target_distance := 9.7
var _yaw := 0.0
var _pitch := 8.0
var _distance := 9.7

var _orbiting := false
var _orbit_moved := false
var _dragging_node := ""
var _drag_moved := false
var _last_mouse := Vector2.ZERO
var _mouse_down := Vector2.ZERO
var _hovered_id := ""
var _active := true
var _time := 0.0


func _ready() -> void:
	_build_environment()
	_build_background()
	_build_stars()
	_build_lights()
	_build_globe()
	_pivot = Node3D.new()
	_pivot.name = "OrbitPivot"
	add_child(_pivot)
	_camera = Camera3D.new()
	_camera.name = "Camera"
	_camera.fov = 42.0
	_camera.near = 0.08
	_camera.far = 180.0
	_camera.current = true
	_camera.position = Vector3(0.0, 0.0, _distance)
	_pivot.add_child(_camera)
	_group_layer = Node3D.new()
	_group_layer.name = "GroupRings"
	add_child(_group_layer)
	_link_layer = Node3D.new()
	_link_layer.name = "Links"
	add_child(_link_layer)
	_node_layer = Node3D.new()
	_node_layer.name = "Thoughts"
	add_child(_node_layer)
	_label_canvas = CanvasLayer.new()
	_label_canvas.name = "ThoughtLabels"
	_label_canvas.layer = 1
	add_child(_label_canvas)
	_label_overlay = Control.new()
	_label_overlay.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	_label_overlay.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_label_canvas.add_child(_label_overlay)
	_apply_camera()


func set_data(sphere: Dictionary, restore_camera := true) -> void:
	data = sphere
	selected_id = ""
	_hovered_id = ""
	if restore_camera:
		var camera: Dictionary = data.get("camera", {})
		_target_yaw = float(camera.get("yaw", 0.0))
		_target_pitch = float(camera.get("pitch", 8.0))
		_target_distance = float(camera.get("distance", 9.7))
		_yaw = _target_yaw
		_pitch = _target_pitch
		_distance = _target_distance
		_apply_camera()
	_rebuild_nodes()
	_rebuild_links()
	_rebuild_groups()


func set_active(active: bool) -> void:
	_active = active
	visible = active
	set_process(active)
	if _label_overlay != null:
		_label_overlay.visible = active
	if not active:
		_set_cursor_default()


func set_selected(node_id: String) -> void:
	selected_id = node_id
	_refresh_node_states()


func focus_node(node_id: String) -> void:
	var node := SphereStore.find_node(data, node_id)
	if node.is_empty():
		return
	var unit := SphereStore.node_position(node)
	_target_yaw = rad_to_deg(atan2(unit.x, unit.z))
	_target_pitch = clampf(rad_to_deg(asin(clampf(unit.y, -1.0, 1.0))), -82.0, 82.0)
	_yaw = _target_yaw
	_pitch = _target_pitch
	_camera.position.z = _distance
	_apply_camera()
	_emit_camera()


func suggested_position() -> Vector3:
	var base := _camera.global_position.normalized()
	var right := base.cross(Vector3.UP)
	if right.length_squared() < 0.001:
		right = base.cross(Vector3.RIGHT)
	right = right.normalized()
	var up := right.cross(base).normalized()
	var best := base
	var best_clearance := -1.0
	for i in 90:
		var angle := float(i) * 2.399963
		var radius := 0.18 + 0.058 * sqrt(float(i))
		if radius > 1.05:
			break
		var dir := (base * cos(radius) + (right * cos(angle) + up * sin(angle)) * sin(radius)).normalized()
		var clearance := _angular_clearance(dir)
		if clearance > best_clearance:
			best = dir
			best_clearance = clearance
		if clearance > 0.32:
			return dir
	return best


func update_node(node_id: String) -> void:
	var node := SphereStore.find_node(data, node_id)
	if node.is_empty() or not _node_visuals.has(node_id):
		return
	var info: Dictionary = _node_visuals[node_id]
	var unit := SphereStore.node_position(node)
	info.unit = unit
	var root: Node3D = info.root
	root.position = unit * GLOBE_RADIUS
	var label: Label = info.label
	label.text = _short_title(String(node.title))
	var color := SphereStore.group_color(data, String(node.group_id))
	info.color = color
	var body_material: StandardMaterial3D = info.body.material_override
	body_material.albedo_color = color.darkened(0.16)
	body_material.emission = color
	label.add_theme_color_override("font_color", color.lightened(0.62))
	var halo_material: StandardMaterial3D = info.halo.material_override
	halo_material.albedo_color = Color(color.r, color.g, color.b, 0.15)
	_refresh_node_states()
	_rebuild_links()
	_rebuild_groups()


func rebuild_derived() -> void:
	_rebuild_links()
	_rebuild_groups()


func camera_data() -> Dictionary:
	return {"yaw": _target_yaw, "pitch": _target_pitch, "distance": _target_distance}


func _process(delta: float) -> void:
	_time += delta
	var damp := 1.0 - exp(-11.0 * delta)
	_yaw = lerpf(_yaw, _target_yaw, damp)
	_pitch = lerpf(_pitch, _target_pitch, damp)
	_distance = lerpf(_distance, _target_distance, damp)
	_camera.position.z = _distance
	_pivot.rotation = Vector3(deg_to_rad(-_pitch), deg_to_rad(_yaw), 0.0)
	_animate_nodes(delta)
	_update_screen_labels()
	if not _orbiting and _dragging_node.is_empty():
		var hovered := ""
		if get_viewport().gui_get_hovered_control() == null:
			hovered = _pick_node(get_viewport().get_mouse_position())
		if hovered != _hovered_id:
			_hovered_id = hovered
			_refresh_node_states()
		if hovered.is_empty():
			_set_cursor_default()
		else:
			Input.set_default_cursor_shape(Input.CURSOR_POINTING_HAND)


func _unhandled_input(event: InputEvent) -> void:
	if not _active:
		return
	if event is InputEventMouseButton:
		var mouse_event := event as InputEventMouseButton
		if mouse_event.button_index == MOUSE_BUTTON_WHEEL_UP and mouse_event.pressed:
			_target_distance = clampf(_target_distance - 0.65, 5.4, 15.0)
			_emit_camera()
			get_viewport().set_input_as_handled()
			return
		if mouse_event.button_index == MOUSE_BUTTON_WHEEL_DOWN and mouse_event.pressed:
			_target_distance = clampf(_target_distance + 0.65, 5.4, 15.0)
			_emit_camera()
			get_viewport().set_input_as_handled()
			return
		if mouse_event.button_index == MOUSE_BUTTON_LEFT:
			if mouse_event.pressed:
				_mouse_down = mouse_event.position
				var picked := _pick_node(mouse_event.position)
				if not picked.is_empty():
					_dragging_node = picked
					_drag_moved = false
					selected_id = picked
					_refresh_node_states()
					node_selected.emit(picked)
					if mouse_event.double_click:
						node_activated.emit(picked)
				else:
					_orbiting = true
					_orbit_moved = false
					_last_mouse = mouse_event.position
				get_viewport().set_input_as_handled()
			else:
				if not _dragging_node.is_empty():
					if _drag_moved:
						node_moved.emit(_dragging_node)
					_dragging_node = ""
					_drag_moved = false
				if _orbiting:
					_orbiting = false
					if not _orbit_moved:
						background_clicked.emit()
					else:
						_emit_camera()
				get_viewport().set_input_as_handled()
		elif mouse_event.button_index == MOUSE_BUTTON_RIGHT:
			if mouse_event.pressed:
				_orbiting = true
				_orbit_moved = false
				_last_mouse = mouse_event.position
			else:
				_orbiting = false
				if _orbit_moved:
					_emit_camera()
	elif event is InputEventMouseMotion:
		var motion := event as InputEventMouseMotion
		if not _dragging_node.is_empty():
			var hit: Variant = _ray_sphere(motion.position)
			if hit != null:
				_move_dragged_node(hit.normalized())
				_drag_moved = true
			get_viewport().set_input_as_handled()
		elif _orbiting:
			var delta := motion.position - _last_mouse
			_last_mouse = motion.position
			if delta.length() > 0.8:
				_orbit_moved = true
			_target_yaw -= delta.x * 0.30
			_target_pitch = clampf(_target_pitch - delta.y * 0.25, -82.0, 82.0)
			get_viewport().set_input_as_handled()
	elif event is InputEventPanGesture:
		var pan := event as InputEventPanGesture
		_target_yaw -= pan.delta.x * 3.0
		_target_pitch = clampf(_target_pitch - pan.delta.y * 2.5, -82.0, 82.0)
		get_viewport().set_input_as_handled()


func _build_environment() -> void:
	var world := WorldEnvironment.new()
	var environment := Environment.new()
	environment.background_mode = Environment.BG_COLOR
	environment.background_color = Color("#070a12")
	environment.ambient_light_source = Environment.AMBIENT_SOURCE_COLOR
	environment.ambient_light_color = Color("#7f9cc6")
	environment.ambient_light_energy = 0.62
	environment.tonemap_mode = Environment.TONE_MAPPER_FILMIC
	environment.tonemap_exposure = 1.04
	environment.glow_enabled = true
	environment.glow_intensity = 0.85
	environment.glow_strength = 1.0
	environment.glow_bloom = 0.04
	environment.glow_blend_mode = Environment.GLOW_BLEND_MODE_ADDITIVE
	environment.glow_hdr_threshold = 0.72
	environment.set("glow_levels/2", 0.4)
	environment.set("glow_levels/3", 1.0)
	environment.set("glow_levels/4", 1.0)
	environment.set("glow_levels/5", 0.5)
	world.environment = environment
	add_child(world)


func _build_background() -> void:
	var background := MeshInstance3D.new()
	var sphere := SphereMesh.new()
	sphere.radius = 80.0
	sphere.height = 160.0
	sphere.radial_segments = 48
	sphere.rings = 24
	var material := ShaderMaterial.new()
	material.shader = BACKGROUND_SHADER
	background.mesh = sphere
	background.material_override = material
	background.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	background.extra_cull_margin = 100.0
	add_child(background)


func _build_stars() -> void:
	var quad := QuadMesh.new()
	quad.size = Vector2(0.13, 0.13)
	var material := StandardMaterial3D.new()
	material.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	material.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	material.billboard_mode = BaseMaterial3D.BILLBOARD_ENABLED
	material.albedo_color = Color(0.72, 0.82, 1.0, 0.66)
	material.emission_enabled = true
	material.emission = Color(0.55, 0.72, 1.0)
	material.emission_energy_multiplier = 1.8
	quad.material = material
	var multimesh := MultiMesh.new()
	multimesh.transform_format = MultiMesh.TRANSFORM_3D
	multimesh.use_colors = true
	multimesh.mesh = quad
	multimesh.instance_count = 260
	var rng := RandomNumberGenerator.new()
	rng.seed = 81573
	for i in 260:
		var dir := Vector3(rng.randf_range(-1.0, 1.0), rng.randf_range(-1.0, 1.0), rng.randf_range(-1.0, 1.0))
		if dir.length_squared() < 0.01:
			dir = Vector3.UP
		dir = dir.normalized()
		var scale := rng.randf_range(0.45, 1.65)
		var basis := Basis().scaled(Vector3(scale, scale, scale))
		multimesh.set_instance_transform(i, Transform3D(basis, dir * 58.0))
		var warmth := rng.randf()
		var tint := Color(0.66, 0.80, 1.0).lerp(Color(1.0, 0.84, 0.66), warmth)
		tint.a = rng.randf_range(0.25, 0.82)
		multimesh.set_instance_color(i, tint)
	var stars := MultiMeshInstance3D.new()
	stars.name = "Starfield"
	stars.multimesh = multimesh
	stars.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	add_child(stars)


func _build_lights() -> void:
	var key := DirectionalLight3D.new()
	key.rotation_degrees = Vector3(-32.0, -38.0, 0.0)
	key.light_color = Color("#d7e8ff")
	key.light_energy = 1.15
	key.shadow_enabled = false
	add_child(key)
	var cool := OmniLight3D.new()
	cool.position = Vector3(3.8, 2.2, 4.2)
	cool.light_color = Color("#52bde0")
	cool.light_energy = 2.0
	cool.omni_range = 11.0
	cool.shadow_enabled = false
	add_child(cool)
	var warm := OmniLight3D.new()
	warm.position = Vector3(-4.2, -2.6, 2.4)
	warm.light_color = Color("#df815c")
	warm.light_energy = 1.25
	warm.omni_range = 9.0
	warm.shadow_enabled = false
	add_child(warm)


func _build_globe() -> void:
	var globe := MeshInstance3D.new()
	globe.name = "Globe"
	var sphere := SphereMesh.new()
	sphere.radius = GLOBE_RADIUS
	sphere.height = GLOBE_RADIUS * 2.0
	sphere.radial_segments = 96
	sphere.rings = 48
	var material := ShaderMaterial.new()
	material.shader = GLOBE_SHADER
	globe.mesh = sphere
	globe.material_override = material
	globe.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	add_child(globe)
	var atmosphere := MeshInstance3D.new()
	atmosphere.name = "Atmosphere"
	var atmosphere_sphere := SphereMesh.new()
	atmosphere_sphere.radius = GLOBE_RADIUS * 1.055
	atmosphere_sphere.height = GLOBE_RADIUS * 2.11
	atmosphere_sphere.radial_segments = 96
	atmosphere_sphere.rings = 48
	var atmosphere_material := ShaderMaterial.new()
	atmosphere_material.shader = ATMOSPHERE_SHADER
	atmosphere.mesh = atmosphere_sphere
	atmosphere.material_override = atmosphere_material
	atmosphere.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	add_child(atmosphere)


func _rebuild_nodes() -> void:
	for child in _node_layer.get_children():
		child.queue_free()
	for child in _label_overlay.get_children():
		child.queue_free()
	_node_visuals.clear()
	for node: Dictionary in data.get("nodes", []):
		_create_node(node)


func _create_node(node: Dictionary) -> void:
	var unit := SphereStore.node_position(node)
	var color := SphereStore.group_color(data, String(node.group_id))
	var root := Node3D.new()
	root.name = String(node.id)
	root.position = unit * GLOBE_RADIUS
	_node_layer.add_child(root)

	var body := MeshInstance3D.new()
	var sphere := SphereMesh.new()
	sphere.radius = NODE_RADIUS
	sphere.height = NODE_RADIUS * 2.0
	sphere.radial_segments = 20
	sphere.rings = 10
	var material := StandardMaterial3D.new()
	material.albedo_color = color.darkened(0.28)
	material.roughness = 0.30
	material.metallic = 0.12
	material.emission_enabled = true
	material.emission = color
	material.emission_energy_multiplier = 0.62
	body.mesh = sphere
	body.material_override = material
	body.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	root.add_child(body)

	var halo := MeshInstance3D.new()
	var halo_sphere := SphereMesh.new()
	halo_sphere.radius = NODE_RADIUS * 1.85
	halo_sphere.height = halo_sphere.radius * 2.0
	halo_sphere.radial_segments = 20
	halo_sphere.rings = 10
	var halo_material := StandardMaterial3D.new()
	halo_material.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	halo_material.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	halo_material.blend_mode = BaseMaterial3D.BLEND_MODE_ADD
	halo_material.cull_mode = BaseMaterial3D.CULL_DISABLED
	halo_material.albedo_color = Color(color.r, color.g, color.b, 0.15)
	halo.mesh = halo_sphere
	halo.material_override = halo_material
	halo.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	halo.visible = false
	root.add_child(halo)

	var label_panel := PanelContainer.new()
	label_panel.mouse_filter = Control.MOUSE_FILTER_IGNORE
	label_panel.z_index = 20
	_label_overlay.add_child(label_panel)
	var label := Label.new()
	label.text = _short_title(String(node.title))
	label.add_theme_font_size_override("font_size", 13)
	label.add_theme_color_override("font_color", color.lightened(0.62))
	label.add_theme_color_override("font_outline_color", Color(0.01, 0.02, 0.04, 0.9))
	label.add_theme_constant_override("outline_size", 3)
	label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	label_panel.add_child(label)

	_node_visuals[node.id] = {
		"root": root,
		"body": body,
		"halo": halo,
		"label": label,
		"label_panel": label_panel,
		"unit": unit,
		"color": color,
	}
	_refresh_node_states()


func _refresh_node_states() -> void:
	for id: String in _node_visuals:
		var info: Dictionary = _node_visuals[id]
		var selected := id == selected_id
		var hovered := id == _hovered_id
		var target := 1.32 if selected else (1.16 if hovered else 1.0)
		var body: MeshInstance3D = info.body
		var halo: MeshInstance3D = info.halo
		body.scale = Vector3.ONE * target
		halo.visible = selected or hovered
		var halo_material: StandardMaterial3D = halo.material_override
		halo_material.albedo_color.a = 0.20 if selected else 0.11
		var label: Label = info.label
		label.add_theme_color_override("font_color", info.color.lightened(0.82) if selected else (info.color.lightened(0.70) if hovered else info.color.lightened(0.58)))
		var label_panel: PanelContainer = info.label_panel
		label_panel.add_theme_stylebox_override("panel", _thought_label_style(info.color, selected, hovered))
		label_panel.z_index = 60 if selected else (40 if hovered else 20)


func _update_screen_labels() -> void:
	if _label_overlay == null:
		return
	for id: String in _node_visuals:
		var info: Dictionary = _node_visuals[id]
		var panel: PanelContainer = info.label_panel
		var root: Node3D = info.root
		var camera_position := _camera.global_position
		var to_camera := (camera_position - root.global_position).normalized()
		var front := (info.unit as Vector3).dot(to_camera)
		if front < 0.035 or _camera.is_position_behind(root.global_position):
			panel.visible = false
			continue
		panel.visible = true
		var screen := _camera.unproject_position(root.global_position)
		var min_size := panel.get_combined_minimum_size()
		panel.size = min_size
		var anchor := screen + Vector2(0.0, -26.0)
		var position := anchor - Vector2(min_size.x * 0.5, min_size.y)
		var viewport_size := get_viewport().get_visible_rect().size
		position.x = clampf(position.x, 8.0, viewport_size.x - min_size.x - 8.0)
		position.y = clampf(position.y, 4.0, viewport_size.y - min_size.y - 4.0)
		panel.position = position


func _thought_label_style(color: Color, selected: bool, hovered: bool) -> StyleBoxFlat:
	var alpha := 0.90 if selected else (0.84 if hovered else 0.72)
	var border_alpha := 0.75 if selected else (0.50 if hovered else 0.26)
	var style := UiFactory.panel_style(
		Color(0.026, 0.043, 0.075, alpha),
		Color(color.r, color.g, color.b, border_alpha),
		7,
		1
	)
	style.content_margin_left = 8
	style.content_margin_right = 8
	style.content_margin_top = 3
	style.content_margin_bottom = 3
	style.shadow_color = Color(0, 0, 0, 0.35)
	style.shadow_size = 5
	return style


func _short_title(title: String) -> String:
	var clean := title.strip_edges()
	if clean.length() <= 34:
		return clean
	return clean.substr(0, 33).strip_edges() + "..."


func _animate_nodes(delta: float) -> void:
	if selected_id.is_empty() or not _node_visuals.has(selected_id):
		return
	var info: Dictionary = _node_visuals[selected_id]
	var halo: MeshInstance3D = info.halo
	var pulse := 1.0 + sin(_time * 3.4) * 0.10
	halo.scale = Vector3.ONE * pulse


func _move_dragged_node(unit: Vector3) -> void:
	if not _node_visuals.has(_dragging_node):
		return
	var info: Dictionary = _node_visuals[_dragging_node]
	info.unit = unit
	var root: Node3D = info.root
	root.position = unit * GLOBE_RADIUS
	var node := SphereStore.find_node(data, _dragging_node)
	if not node.is_empty():
		node.pos = [unit.x, unit.y, unit.z]
	_rebuild_links()
	_rebuild_groups()


func _rebuild_links() -> void:
	for child in _link_layer.get_children():
		child.queue_free()
	for link: Dictionary in data.get("links", []):
		var a := SphereStore.find_node(data, String(link.a))
		var b := SphereStore.find_node(data, String(link.b))
		if a.is_empty() or b.is_empty():
			continue
		var start := SphereStore.node_position(a)
		var finish := SphereStore.node_position(b)
		var points := PackedVector3Array()
		var span := start.distance_to(finish)
		var lift := clampf(0.035 + span * 0.035, 0.04, 0.16)
		for i in 29:
			var t := float(i) / 28.0
			var point := start.slerp(finish, t).normalized()
			var arc := sin(PI * t) * lift
			points.append(point * (GLOBE_RADIUS + 0.045 + arc))
		var color_a := SphereStore.group_color(data, String(a.group_id))
		var color_b := SphereStore.group_color(data, String(b.group_id))
		var color := color_a.lerp(color_b, 0.18).darkened(0.10)
		color.a = 0.42
		var mesh := _make_tube(points, 0.0085, false)
		if mesh == null:
			continue
		var instance := MeshInstance3D.new()
		instance.mesh = mesh
		instance.material_override = _glow_material(color, 1.0)
		instance.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
		_link_layer.add_child(instance)


func _rebuild_groups() -> void:
	for child in _group_layer.get_children():
		child.queue_free()
	for group: Dictionary in data.get("groups", []):
		var positions: Array[Vector3] = []
		for node: Dictionary in data.get("nodes", []):
			if node.group_id == group.id:
				positions.append(SphereStore.node_position(node))
		if positions.size() < 2:
			continue
		var center := Vector3.ZERO
		for position in positions:
			center += position
		center = center.normalized()
		var angular_radius := 0.0
		for position in positions:
			angular_radius = maxf(angular_radius, center.angle_to(position))
		angular_radius = minf(angular_radius + 0.115, 1.32)
		var axis := center.cross(Vector3.UP)
		if axis.length_squared() < 0.001:
			axis = center.cross(Vector3.RIGHT)
		axis = axis.normalized()
		var other := center.cross(axis).normalized()
		var points := PackedVector3Array()
		for i in 73:
			var angle := TAU * float(i) / 72.0
			var point := (center * cos(angular_radius) + (axis * cos(angle) + other * sin(angle)) * sin(angular_radius)).normalized()
			points.append(point * (GLOBE_RADIUS + 0.028))
		var color := SphereStore.color_from_html(String(group.color), Color("#8b95b5"))
		color.a = 0.40
		var mesh := _make_tube(points, 0.012, true)
		if mesh == null:
			continue
		var ring := MeshInstance3D.new()
		ring.mesh = mesh
		ring.material_override = _glow_material(color, 0.75)
		ring.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
		_group_layer.add_child(ring)

		var label := Label3D.new()
		label.text = String(group.name).to_upper()
		label.font_size = 30
		label.pixel_size = 0.0032
		label.outline_size = 6
		label.modulate = Color(color.lightened(0.32).r, color.lightened(0.32).g, color.lightened(0.32).b, 0.95)
		label.outline_modulate = Color(0.008, 0.016, 0.035, 0.82)
		label.billboard = BaseMaterial3D.BILLBOARD_ENABLED
		label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
		label.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
		label.position = center * (GLOBE_RADIUS + 0.62)
		label.render_priority = 1
		_group_layer.add_child(label)


func _make_tube(points: PackedVector3Array, radius: float, closed: bool) -> ArrayMesh:
	if points.size() < 2:
		return null
	var st := SurfaceTool.new()
	st.begin(Mesh.PRIMITIVE_TRIANGLES)
	var count := points.size()
	var segment_count := count if closed else count - 1
	var sides := 5
	for i in segment_count:
		var p0 := points[i]
		var p1 := points[(i + 1) % count]
		var tangent := (p1 - p0).normalized()
		if tangent.length_squared() < 0.001:
			continue
		var reference := Vector3.UP if absf(tangent.dot(Vector3.UP)) < 0.94 else Vector3.RIGHT
		var b1 := tangent.cross(reference).normalized()
		var b2 := tangent.cross(b1).normalized()
		for j in sides:
			var angle_a := TAU * float(j) / float(sides)
			var angle_b := TAU * float(j + 1) / float(sides)
			var offset_a := b1 * cos(angle_a) * radius + b2 * sin(angle_a) * radius
			var offset_b := b1 * cos(angle_b) * radius + b2 * sin(angle_b) * radius
			st.add_vertex(p0 + offset_a)
			st.add_vertex(p1 + offset_a)
			st.add_vertex(p1 + offset_b)
			st.add_vertex(p0 + offset_a)
			st.add_vertex(p1 + offset_b)
			st.add_vertex(p0 + offset_b)
	return st.commit()


func _glow_material(color: Color, strength: float) -> StandardMaterial3D:
	var material := StandardMaterial3D.new()
	material.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	material.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	material.blend_mode = BaseMaterial3D.BLEND_MODE_ADD
	material.cull_mode = BaseMaterial3D.CULL_DISABLED
	material.albedo_color = Color(color.r * strength, color.g * strength, color.b * strength, color.a)
	return material


func _angular_clearance(candidate: Vector3) -> float:
	var clearance := PI
	for id: String in _node_visuals:
		var info: Dictionary = _node_visuals[id]
		clearance = minf(clearance, candidate.angle_to(info.unit))
	return clearance


func _pick_node(screen_position: Vector2) -> String:
	var best := ""
	var best_score := INF
	for id: String in _node_visuals:
		var info: Dictionary = _node_visuals[id]
		var root: Node3D = info.root
		if _camera.is_position_behind(root.global_position):
			continue
		var unit: Vector3 = info.unit
		var to_camera := (_camera.global_position - root.global_position).normalized()
		if unit.dot(to_camera) < 0.045:
			continue
		var panel: PanelContainer = info.label_panel
		if panel.visible and panel.get_global_rect().has_point(screen_position):
			var score := 0.0 if id == selected_id else 1.0
			if score < best_score:
				best = id
				best_score = score
			continue
		var screen := _camera.unproject_position(root.global_position)
		var distance := screen.distance_to(screen_position)
		if distance < 27.0 and distance < best_score:
			best = id
			best_score = distance
	return best


func _ray_sphere(screen_position: Vector2) -> Variant:
	var origin := _camera.project_ray_origin(screen_position)
	var direction := _camera.project_ray_normal(screen_position)
	var b := 2.0 * origin.dot(direction)
	var c := origin.dot(origin) - GLOBE_RADIUS * GLOBE_RADIUS
	var discriminant := b * b - 4.0 * c
	if discriminant < 0.0:
		return null
	var t := (-b - sqrt(discriminant)) * 0.5
	if t < 0.0:
		return null
	return origin + direction * t


func _apply_camera() -> void:
	if _pivot == null:
		return
	_pivot.rotation = Vector3(deg_to_rad(-_pitch), deg_to_rad(_yaw), 0.0)
	_camera.position.z = _distance


func _emit_camera() -> void:
	camera_changed.emit(camera_data())


func _set_cursor_default() -> void:
	Input.set_default_cursor_shape(Input.CURSOR_ARROW)
