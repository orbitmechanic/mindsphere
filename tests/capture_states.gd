extends SceneTree

var _main: Node

func _initialize() -> void:
	_main = load("res://main.tscn").instantiate()
	root.add_child(_main)
	_run()

func _run() -> void:
	for i: int in 35:
		await process_frame
	DirAccess.make_dir_recursive_absolute("res://tests/out")
	var welcome_id: String = _main.current.id
	var blank := SphereStore.create_blank("A blank sphere")
	SphereStore.save_sphere(blank)
	_main._refresh_sphere_list()
	_main._load_sphere(String(blank.id))
	for i: int in 12:
		await process_frame
	root.get_texture().get_image().save_png("res://tests/out/empty.png")
	_main._show_new_group_modal()
	for i: int in 8:
		await process_frame
	root.get_texture().get_image().save_png("res://tests/out/modal.png")
	_main._close_modal()
	_main._load_sphere(welcome_id)
	for i: int in 8:
		await process_frame
	SphereStore.delete_sphere(String(blank.id))
	_main._refresh_sphere_list()
	_main._on_board_node_selected(String(_main.current.nodes[0].id))
	_main._set_mode("board")
	for i: int in 12:
		await process_frame
	root.get_texture().get_image().save_png("res://tests/out/board_selected.png")
	quit(0)
