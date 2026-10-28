extends SceneTree

var failures := 0
var _main: Node

func _initialize() -> void:
	print("PROBE=scene")
	_main = load("res://main.tscn").instantiate()
	root.add_child(_main)
	_run()

func _run() -> void:
	for i in 20:
		await process_frame
	_check(not _main.current.is_empty(), "initial sphere loaded")
	var welcome_id := String(_main.current.id)
	var test := SphereStore.create_blank("Scene probe")
	SphereStore.save_sphere(test)
	_main._refresh_sphere_list()
	_main._load_sphere(String(test.id))
	_main._set_mode("sphere")
	for i in 8:
		await process_frame
	_check(_main.current.get("nodes", []).is_empty(), "blank sphere has no nodes")
	_check(_main._empty_overlay.visible, "empty state visible")
	_main._add_thought()
	for i in 12:
		await process_frame
	_check(int(_main.current.nodes.size()) == 1, "thought added")
	_check(not _main.selected_node_id.is_empty(), "new thought selected")
	_check(_main._title_edit != null, "inspector title field present")
	var node_id: String = _main.selected_node_id
	_main._on_title_changed("A tested thought")
	_main._body_edit.text = "Stored locally, then read back."
	_main._on_body_changed()
	for i in 4:
		await process_frame
	_check(String(SphereStore.find_node(_main.current, node_id).title) == "A tested thought", "title updated in memory")
	_main._save_current()
	var reloaded := SphereStore.load_sphere(String(test.id))
	_check(String(reloaded.nodes[0].title) == "A tested thought", "title persisted")
	_check(String(reloaded.nodes[0].body).begins_with("Stored locally"), "notes persisted")
	_main._set_mode("board")
	for i: int in 10:
		await process_frame
	_check(_main._board_host.visible, "board visible")
	_check(_main.board_view._cards.has(node_id), "board card present")
	_main._on_board_node_selected(node_id)
	for i: int in 3:
		await process_frame
	_check(_main.selected_node_id == node_id, "board selection works")
	_main._set_mode("sphere")
	for i in 10:
		await process_frame
	var info: Dictionary = _main.sphere_view._node_visuals[node_id]
	var screen: Vector2 = _main.sphere_view._camera.unproject_position((info.root as Node3D).global_position)
	_check(_main.sphere_view._pick_node(screen) == node_id, "screen-space picking finds thought")
	_check(_main.sphere_view.suggested_position().length() > 0.99, "suggested position is on sphere")
	_main._show_shortcuts_modal()
	for i: int in 2:
		await process_frame
	var nodes_before: int = _main.current.nodes.size()
	var shortcut := InputEventKey.new()
	shortcut.pressed = true
	shortcut.keycode = KEY_N
	_main._unhandled_key_input(shortcut)
	for i: int in 2:
		await process_frame
	_check(_main.current.nodes.size() == nodes_before, "modal blocks global shortcuts")
	_main._close_modal()
	_main._show_about_modal()
	for i: int in 2:
		await process_frame
	_check(_main._modal_layer.visible, "modal reopens immediately")
	_check(_main._modal_center != null and _main._modal_center.get_child_count() == 1, "one modal panel is active")
	_check(_main._modal_layer.get_child_count() == 2, "stale modal layers are detached")
	_main._close_modal()
	_main._load_sphere(welcome_id)
	for i: int in 8:
		await process_frame
	SphereStore.delete_sphere(String(test.id))
	_main._refresh_sphere_list()
	_main._set_mode("sphere")
	_check(String(_main.current.id) == welcome_id, "returned to welcome sphere")
	_main.queue_free()
	await process_frame
	await process_frame
	if failures == 0:
		print("PROBE_RESULT=PASS")
		quit(0)
	else:
		print("PROBE_RESULT=FAIL count=", failures)
		quit(1)


func _check(condition: bool, label: String) -> void:
	if condition:
		print("PASS ", label)
	else:
		failures += 1
		print("FAIL ", label)
