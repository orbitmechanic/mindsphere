shader_type spatial;
render_mode unshaded, blend_add, cull_back, depth_draw_never, shadows_disabled, fog_disabled;

uniform vec3 glow_color : source_color = vec3(0.18, 0.62, 0.88);
uniform float intensity = 1.0;

void fragment() {
	float facing = clamp(dot(NORMAL, VIEW), 0.0, 1.0);
	float rim = pow(1.0 - facing, 2.2);
	ALBEDO = glow_color * rim * 2.1 * intensity;
	ALPHA = clamp(rim * 0.78, 0.0, 1.0);
}
