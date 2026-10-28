extends SceneTree

var _main: Node

func _initialize() -> void:
	_main = load("res://main.tscn").instantiate()
	root.add_child(_main)
	_run()

func _run() -> void:
	for i in 45:
		await process_frame
	DirAccess.make_dir_recursive_absolute("res://tests/out")
	print("VIEWPORT=", root.size)
	root.get_texture().get_image().save_png("res://tests/out/sphere.png")
	_main._set_mode("board")
	for i in 15:
		await process_frame
	root.get_texture().get_image().save_png("res://tests/out/board.png")
	_main._set_mode("sphere")
	var node_id := ""
	for node in _main.current.get("nodes", []):
		if String(node.title) == "No centre, no edge":
			node_id = String(node.id)
			break
	if not node_id.is_empty():
		_main._select_node(node_id)
	for i in 12:
		await process_frame
	root.get_texture().get_image().save_png("res://tests/out/selected.png")
	quit(0)
