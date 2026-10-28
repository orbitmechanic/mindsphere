extends SceneTree

var failures := 0

func _initialize() -> void:
	print("PROBE=store")
	SphereStore.ensure_library()
	var sphere := SphereStore.create_blank("Probe sphere")
	_check(sphere.id.begins_with("sphere_"), "blank sphere id")
	var group := SphereStore.add_group(sphere, "Cluster")
	var a := SphereStore.add_node(sphere, "Alpha", String(group.id), SphereStore.spherical(10.0, 20.0), "first")
	var b := SphereStore.add_node(sphere, "Beta", String(group.id), SphereStore.spherical(-10.0, 30.0), "second")
	_check(SphereStore.add_link(sphere, String(a.id), String(b.id)), "link added")
	_check(not SphereStore.add_link(sphere, String(a.id), String(b.id)), "duplicate link rejected")
	_check(SphereStore.save_sphere(sphere), "sphere saved")
	var loaded := SphereStore.load_sphere(String(sphere.id))
	_check(int(loaded.nodes.size()) == 2, "nodes round trip")
	_check(int(loaded.links.size()) == 1, "links round trip")
	_check(absf(SphereStore.node_position(loaded.nodes[0]).length() - 1.0) < 0.001, "position normalized")
	var export_path := "user://probe_export.json"
	_check(SphereStore.export_sphere(loaded, export_path), "export written")
	var imported := SphereStore.import_sphere(export_path)
	_check(not imported.is_empty(), "import read")
	_check(String(imported.id) != String(loaded.id), "import gets new id")
	_check(int(imported.nodes.size()) == 2, "import nodes preserved")
	var imported_text := SphereStore.import_sphere_text(SphereStore.export_sphere_text(loaded))
	_check(not imported_text.is_empty(), "text export/import read")
	_check(String(imported_text.id) != String(loaded.id), "text import gets new id")
	_check(int(imported_text.nodes.size()) == 2, "text import nodes preserved")
	var duplicate := SphereStore.duplicate_sphere(loaded)
	_check(not duplicate.is_empty() and String(duplicate.id) != String(loaded.id), "duplicate gets new id")
	SphereStore.remove_node(loaded, String(a.id))
	_check(int(loaded.nodes.size()) == 1, "node removed")
	_check(int(loaded.links.size()) == 0, "links cascade on node removal")
	_check(SphereStore.delete_sphere(String(sphere.id)), "test sphere deleted")
	_check(SphereStore.delete_sphere(String(imported.id)), "imported sphere deleted")
	_check(SphereStore.delete_sphere(String(imported_text.id)), "text-imported sphere deleted")
	_check(SphereStore.delete_sphere(String(duplicate.id)), "duplicate sphere deleted")
	DirAccess.remove_absolute(ProjectSettings.globalize_path(export_path))
	_check(not FileAccess.file_exists(export_path), "export cleaned")
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
