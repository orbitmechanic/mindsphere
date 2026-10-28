shader_type spatial;
render_mode unshaded, cull_front, depth_draw_never, shadows_disabled, fog_disabled;

uniform vec3 top_color : source_color = vec3(0.024, 0.035, 0.075);
uniform vec3 mid_color : source_color = vec3(0.045, 0.064, 0.118);
uniform vec3 bottom_color : source_color = vec3(0.012, 0.014, 0.027);
uniform vec3 haze_color : source_color = vec3(0.09, 0.32, 0.42);

varying vec3 local_dir;

void vertex() {
	local_dir = normalize(VERTEX);
}

void fragment() {
	float height = clamp(local_dir.y * 0.5 + 0.5, 0.0, 1.0);
	vec3 color = mix(bottom_color, mid_color, smoothstep(0.0, 0.58, height));
	color = mix(color, top_color, smoothstep(0.56, 1.0, height));

	vec3 haze_dir = normalize(vec3(-0.48, 0.36, 0.80));
	float haze = pow(max(dot(local_dir, haze_dir), 0.0), 5.0);
	color += haze_color * haze * 0.16;

	vec3 warm_dir = normalize(vec3(0.72, -0.42, 0.55));
	float warm = pow(max(dot(local_dir, warm_dir), 0.0), 9.0);
	color += vec3(0.20, 0.08, 0.035) * warm * 0.12;

	float vignette = smoothstep(1.2, 0.18, length(local_dir.xy));
	color *= mix(0.72, 1.0, vignette);

	ALBEDO = color;
}
