extends SceneTree
func _initialize() -> void:
	var main: Node = load("res://main.tscn").instantiate()
	root.add_child(main)
	_run(main)
func _run(main: Node) -> void:
	for i in 20:
		await process_frame
	for node in main.find_children("*", "HSeparator", true, false):
		print("HSEP global=", node.get_global_rect(), " visible=", node.is_visible_in_tree())
	for node in main.find_children("*", "VSeparator", true, false):
		print("VSEP global=", node.get_global_rect(), " visible=", node.is_visible_in_tree())
	for node in main.find_children("*", "ScrollContainer", true, false):
		print("SCROLL global=", node.get_global_rect(), " visible=", node.is_visible_in_tree(), " h=", node.scroll_horizontal, " v=", node.scroll_vertical)
	quit()
