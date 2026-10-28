class_name SphereStore
extends RefCounted

const DATA_DIR := "user://spheres"
const SETTINGS_PATH := "user://settings.json"
const FORMAT := "mindsphere.sphere"
const VERSION := 1
const WEB_LIBRARY_KEY := "mindsphere.library.v1"
const WEB_SETTINGS_KEY := "mindsphere.settings.v1"

const GROUP_COLORS: Array[String] = [
	"#5ad2ff", "#ffb454", "#ff7a7a", "#6ee7b7", "#b69cff", "#f38bd0", "#8ed16f", "#e6d36b"
]


static func ensure_library() -> void:
	if _is_web():
		if _web_library().is_empty():
			save_sphere(build_welcome_sphere())
		return
	DirAccess.make_dir_recursive_absolute(DATA_DIR)
	if _list_files().is_empty():
		save_sphere(build_welcome_sphere())


static func list_spheres() -> Array[Dictionary]:
	ensure_library()
	var result: Array[Dictionary] = []
	if _is_web():
		for value: Variant in _web_library().values():
			if not value is Dictionary:
				continue
			var sphere: Dictionary = value
			_normalize(sphere)
			result.append(_summary(sphere))
	else:
		for file_name in _list_files():
			var sphere := _load_path(DATA_DIR.path_join(file_name))
			if not sphere.is_empty():
				result.append(_summary(sphere))
	result.sort_custom(func(a: Dictionary, b: Dictionary) -> bool:
		return int(a.updated_at) > int(b.updated_at)
	)
	return result


static func load_sphere(id: String) -> Dictionary:
	if id.is_empty() or id.contains("/") or id.contains(".."):
		return {}
	if _is_web():
		var library := _web_library()
		if not library.has(id) or not library[id] is Dictionary:
			return {}
		var sphere: Dictionary = library[id]
		_normalize(sphere)
		return sphere
	return _load_path(_path_for(id))


static func save_sphere(sphere: Dictionary) -> bool:
	_normalize(sphere)
	sphere.updated_at = _now()
	if _is_web():
		var library := _web_library()
		library[String(sphere.id)] = sphere.duplicate(true)
		_web_set(WEB_LIBRARY_KEY, JSON.stringify(library))
		return true
	DirAccess.make_dir_recursive_absolute(DATA_DIR)
	var file := FileAccess.open(_path_for(String(sphere.id)), FileAccess.WRITE)
	if file == null:
		push_error("Mindsphere: could not write %s (%s)" % [_path_for(String(sphere.id)), error_string(FileAccess.get_open_error())])
		return false
	file.store_string(JSON.stringify(sphere, "\t"))
	file.close()
	return true


static func delete_sphere(id: String) -> bool:
	if id.is_empty() or id.contains("/") or id.contains(".."):
		return false
	if _is_web():
		var library := _web_library()
		if not library.erase(id):
			return false
		_web_set(WEB_LIBRARY_KEY, JSON.stringify(library))
		return true
	var path := _path_for(id)
	if not FileAccess.file_exists(path):
		return false
	return DirAccess.remove_absolute(path) == OK


static func import_sphere(path: String) -> Dictionary:
	if not FileAccess.file_exists(path):
		return {}
	var file := FileAccess.open(path, FileAccess.READ)
	if file == null:
		return {}
	return import_sphere_text(file.get_as_text())


static func import_sphere_text(text: String) -> Dictionary:
	var parsed: Variant = JSON.parse_string(text)
	if not parsed is Dictionary:
		return {}
	_normalize(parsed)
	parsed.id = _make_id("sphere")
	parsed.created_at = _now()
	parsed.updated_at = _now()
	if not save_sphere(parsed):
		return {}
	return parsed


static func export_sphere(sphere: Dictionary, path: String) -> bool:
	_normalize(sphere)
	var file := FileAccess.open(path, FileAccess.WRITE)
	if file == null:
		return false
	file.store_string(JSON.stringify(sphere, "\t"))
	file.close()
	return true


static func export_sphere_text(sphere: Dictionary) -> String:
	_normalize(sphere)
	return JSON.stringify(sphere, "\t")


static func duplicate_sphere(sphere: Dictionary) -> Dictionary:
	var copy := create_blank(String(sphere.get("title", "Untitled sphere")) + " copy")
	copy.groups = sphere.get("groups", []).duplicate(true)
	copy.nodes = sphere.get("nodes", []).duplicate(true)
	copy.links = sphere.get("links", []).duplicate(true)
	copy.camera = sphere.get("camera", {}).duplicate(true)
	_normalize(copy)
	if save_sphere(copy):
		return copy
	return {}


static func create_blank(title: String) -> Dictionary:
	var now := _now()
	var sphere := {
		"format": FORMAT,
		"version": VERSION,
		"id": _make_id("sphere"),
		"title": title.strip_edges() if not title.strip_edges().is_empty() else "Untitled sphere",
		"created_at": now,
		"updated_at": now,
		"camera": {"yaw": 0.0, "pitch": 10.0, "distance": 9.7},
		"groups": [],
		"nodes": [],
		"links": [],
	}
	_normalize(sphere)
	return sphere


static func build_welcome_sphere() -> Dictionary:
	var sphere := create_blank("Thinking it through")
	sphere.camera = {"yaw": 0.0, "pitch": 8.0, "distance": 9.7}
	var g_start := add_group(sphere, "Start here", "#5ad2ff")
	var g_ideas := add_group(sphere, "Ways in", "#ffb454")
	var g_questions := add_group(sphere, "Open questions", "#b69cff")
	var g_local := add_group(sphere, "Your space", "#6ee7b7")

	var a := add_node(sphere, "No centre, no edge", g_start.id, spherical(5.0, -5.0), "A sphere has no fixed middle and no right way up. Any thought can be brought to the centre just by turning the map.")
	var b := add_node(sphere, "Drag a thought", g_start.id, spherical(24.0, -22.0), "Drag any orb across the surface. Ideas are allowed to move when your thinking changes.")
	var c := add_node(sphere, "Group related ideas", g_ideas.id, spherical(-5.0, 20.0), "Give a cluster a name and a colour. Groups become columns in the 2D board.")
	var d := add_node(sphere, "Connect the dots", g_ideas.id, spherical(22.0, 29.0), "Draw a path between thoughts that lead into one another. Connections work across groups.")
	var e := add_node(sphere, "Every angle is valid", g_questions.id, spherical(-24.0, 2.0), "Turn the sphere to see the same set of thoughts from another side. Perspective is part of the map.")
	var f := add_node(sphere, "What will you think through?", g_questions.id, spherical(-30.0, -18.0), "Start with a question that matters. Add everything that comes up before you begin to sort it.")
	var g := add_node(sphere, "Your data stays local", g_local.id, spherical(35.0, -4.0), "Spheres are plain JSON files in your Godot user directory. No account, no cloud, no telemetry.")
	var h := add_node(sphere, "Export a snapshot", g_local.id, spherical(39.0, 16.0), "Use Export to keep a portable copy of this sphere, or Import to bring one in.")

	add_link(sphere, a.id, b.id)
	add_link(sphere, a.id, c.id)
	add_link(sphere, a.id, d.id)
	add_link(sphere, c.id, e.id)
	add_link(sphere, e.id, f.id)
	add_link(sphere, d.id, g.id)
	add_link(sphere, g.id, h.id)
	add_link(sphere, f.id, h.id)
	return sphere


static func add_group(sphere: Dictionary, name: String, color := "") -> Dictionary:
	if not sphere.has("groups"):
		sphere.groups = []
	var palette_index := int(sphere.groups.size()) % GROUP_COLORS.size()
	var group := {
		"id": _make_id("group"),
		"name": name.strip_edges() if not name.strip_edges().is_empty() else "New group",
		"color": color if not color.is_empty() else GROUP_COLORS[palette_index],
	}
	sphere.groups.append(group)
	return group


static func add_node(sphere: Dictionary, title: String, group_id: String, pos: Vector3, body := "") -> Dictionary:
	if not sphere.has("nodes"):
		sphere.nodes = []
	var node := {
		"id": _make_id("thought"),
		"title": title.strip_edges() if not title.strip_edges().is_empty() else "Untitled thought",
		"body": body,
		"group_id": group_id,
		"pos": _vector_to_array(pos.normalized()),
		"created_at": _now(),
	}
	sphere.nodes.append(node)
	return node


static func add_link(sphere: Dictionary, a_id: String, b_id: String) -> bool:
	if a_id == b_id or a_id.is_empty() or b_id.is_empty():
		return false
	if not sphere.has("links"):
		sphere.links = []
	for link: Dictionary in sphere.links:
		if (link.a == a_id and link.b == b_id) or (link.a == b_id and link.b == a_id):
			return false
	sphere.links.append({"a": a_id, "b": b_id})
	return true


static func remove_link(sphere: Dictionary, a_id: String, b_id: String) -> void:
	if not sphere.has("links"):
		return
	sphere.links = sphere.links.filter(func(link: Dictionary) -> bool:
		return not ((link.a == a_id and link.b == b_id) or (link.a == b_id and link.b == a_id))
	)


static func remove_node(sphere: Dictionary, node_id: String) -> void:
	sphere.nodes = sphere.nodes.filter(func(node: Dictionary) -> bool:
		return node.id != node_id
	)
	sphere.links = sphere.links.filter(func(link: Dictionary) -> bool:
		return link.a != node_id and link.b != node_id
	)


static func find_node(sphere: Dictionary, node_id: String) -> Dictionary:
	for node: Dictionary in sphere.get("nodes", []):
		if node.id == node_id:
			return node
	return {}


static func find_group(sphere: Dictionary, group_id: String) -> Dictionary:
	for group: Dictionary in sphere.get("groups", []):
		if group.id == group_id:
			return group
	return {}


static func group_name(sphere: Dictionary, group_id: String) -> String:
	var group := find_group(sphere, group_id)
	return String(group.get("name", "Unfiled"))


static func group_color(sphere: Dictionary, group_id: String) -> Color:
	var group := find_group(sphere, group_id)
	return color_from_html(String(group.get("color", "#8b95b5")), Color("#8b95b5"))


static func color_from_html(html: String, fallback := Color.WHITE) -> Color:
	if html.is_empty():
		return fallback
	return Color.from_string(html, fallback)


static func linked_node_ids(sphere: Dictionary, node_id: String) -> Array[String]:
	var result: Array[String] = []
	for link: Dictionary in sphere.get("links", []):
		if link.a == node_id:
			result.append(String(link.b))
		elif link.b == node_id:
			result.append(String(link.a))
	return result


static func node_position(node: Dictionary) -> Vector3:
	var raw: Array = node.get("pos", [0.0, 0.0, 1.0])
	if raw.size() < 3:
		return Vector3.BACK
	return Vector3(float(raw[0]), float(raw[1]), float(raw[2])).normalized()


static func spherical(lat_deg: float, lon_deg: float) -> Vector3:
	var lat := deg_to_rad(lat_deg)
	var lon := deg_to_rad(lon_deg)
	return Vector3(cos(lat) * sin(lon), sin(lat), cos(lat) * cos(lon)).normalized()


static func load_settings() -> Dictionary:
	if _is_web():
		var raw_web := _web_get(WEB_SETTINGS_KEY)
		if raw_web.is_empty():
			return {}
		var parsed_web: Variant = JSON.parse_string(raw_web)
		return parsed_web if parsed_web is Dictionary else {}
	if not FileAccess.file_exists(SETTINGS_PATH):
		return {}
	var file := FileAccess.open(SETTINGS_PATH, FileAccess.READ)
	if file == null:
		return {}
	var parsed: Variant = JSON.parse_string(file.get_as_text())
	return parsed if parsed is Dictionary else {}


static func save_settings(settings: Dictionary) -> void:
	if _is_web():
		_web_set(WEB_SETTINGS_KEY, JSON.stringify(settings))
		return
	var file := FileAccess.open(SETTINGS_PATH, FileAccess.WRITE)
	if file != null:
		file.store_string(JSON.stringify(settings, "\t"))
		file.close()


static func _summary(sphere: Dictionary) -> Dictionary:
	return {
		"id": sphere.id,
		"title": sphere.title,
		"node_count": sphere.nodes.size(),
		"group_count": sphere.groups.size(),
		"updated_at": sphere.updated_at,
	}


static func _is_web() -> bool:
	return OS.has_feature("web")


static func _web_library() -> Dictionary:
	var raw := _web_get(WEB_LIBRARY_KEY)
	if raw.is_empty():
		return {}
	var parsed: Variant = JSON.parse_string(raw)
	return parsed if parsed is Dictionary else {}


static func _web_get(key: String) -> String:
	if not _is_web():
		return ""
	var encoded := String(JavaScriptBridge.eval("window.localStorage.getItem(%s) || ''" % JSON.stringify(key), true))
	if encoded.is_empty():
		return ""
	return Marshalls.base64_to_utf8(encoded)


static func _web_set(key: String, value: String) -> void:
	if not _is_web():
		return
	var encoded := Marshalls.utf8_to_base64(value)
	JavaScriptBridge.eval("window.localStorage.setItem(%s, %s)" % [JSON.stringify(key), JSON.stringify(encoded)], true)


static func _load_path(path: String) -> Dictionary:
	if not FileAccess.file_exists(path):
		return {}
	var file := FileAccess.open(path, FileAccess.READ)
	if file == null:
		return {}
	var parsed: Variant = JSON.parse_string(file.get_as_text())
	if not parsed is Dictionary:
		push_warning("Mindsphere: ignoring invalid sphere file %s" % path)
		return {}
	_normalize(parsed)
	return parsed


static func _normalize(sphere: Dictionary) -> void:
	sphere["format"] = FORMAT
	sphere["version"] = VERSION
	sphere["id"] = String(sphere.get("id", _make_id("sphere")))
	sphere["title"] = String(sphere.get("title", "Untitled sphere"))
	sphere["created_at"] = int(sphere.get("created_at", _now()))
	sphere["updated_at"] = int(sphere.get("updated_at", _now()))
	var camera: Variant = sphere.get("camera", {})
	if not camera is Dictionary:
		camera = {}
	camera["yaw"] = float(camera.get("yaw", 0.0))
	camera["pitch"] = float(camera.get("pitch", 10.0))
	camera["distance"] = clampf(float(camera.get("distance", 9.0)), 5.2, 16.0)
	sphere["camera"] = camera
	if not sphere.get("groups") is Array:
		sphere["groups"] = []
	if not sphere.get("nodes") is Array:
		sphere["nodes"] = []
	if not sphere.get("links") is Array:
		sphere["links"] = []
	var valid_groups: Array[String] = []
	for group: Dictionary in sphere.groups:
		group["id"] = String(group.get("id", _make_id("group")))
		group["name"] = String(group.get("name", "New group"))
		group["color"] = String(group.get("color", "#8b95b5"))
		valid_groups.append(group.id)
	var valid_nodes: Array[String] = []
	for node: Dictionary in sphere.nodes:
		node["id"] = String(node.get("id", _make_id("thought")))
		node["title"] = String(node.get("title", "Untitled thought"))
		node["body"] = String(node.get("body", ""))
		node["group_id"] = String(node.get("group_id", ""))
		if not valid_groups.has(node.group_id):
			node["group_id"] = ""
		node["created_at"] = int(node.get("created_at", _now()))
		var raw: Variant = node.get("pos", [0.0, 0.0, 1.0])
		var pos := Vector3.BACK
		if raw is Array and raw.size() >= 3:
			pos = Vector3(float(raw[0]), float(raw[1]), float(raw[2]))
		if pos.length_squared() < 0.0001:
			pos = Vector3.BACK
		node["pos"] = _vector_to_array(pos.normalized())
		valid_nodes.append(node.id)
	sphere["links"] = sphere.links.filter(func(link: Dictionary) -> bool:
		return valid_nodes.has(String(link.get("a", ""))) and valid_nodes.has(String(link.get("b", "")))
	)


static func _path_for(id: String) -> String:
	return DATA_DIR.path_join(id + ".json")


static func _list_files() -> Array[String]:
	var result: Array[String] = []
	var dir := DirAccess.open(DATA_DIR)
	if dir == null:
		return result
	for file_name in dir.get_files():
		if file_name.ends_with(".json"):
			result.append(file_name)
	return result


static func _make_id(prefix: String) -> String:
	var stamp := int(Time.get_unix_time_from_system() * 1000.0)
	return "%s_%d_%d" % [prefix, stamp, randi_range(1000, 999999)]


static func _now() -> int:
	return int(Time.get_unix_time_from_system())


static func _vector_to_array(value: Vector3) -> Array:
	return [value.x, value.y, value.z]
