shader_type spatial;
render_mode blend_mix, depth_draw_opaque, cull_back, diffuse_burley, specular_schlick_ggx;

uniform vec3 base_color : source_color = vec3(0.035, 0.06, 0.115);
uniform vec3 deep_color : source_color = vec3(0.012, 0.02, 0.045);
uniform vec3 grid_color : source_color = vec3(0.24, 0.64, 0.82);
uniform vec3 rim_color : source_color = vec3(0.15, 0.48, 0.68);
uniform float grid_strength = 0.17;
uniform float rim_strength = 0.55;

varying vec3 local_dir;

void vertex() {
	local_dir = normalize(VERTEX);
}

void fragment() {
	float lat_cell = fract(UV.y * 18.0);
	float lon_cell = fract(UV.x * 36.0);
	float lat_line = 1.0 - smoothstep(0.0, 0.055, abs(lat_cell - 0.5));
	float lon_line = 1.0 - smoothstep(0.0, 0.040, abs(lon_cell - 0.5));
	float grid = max(lat_line, lon_line);
	grid *= smoothstep(0.0, 0.10, UV.y) * smoothstep(1.0, 0.90, UV.y);

	float depth_tint = smoothstep(-0.6, 0.65, local_dir.z);
	vec3 color = mix(deep_color, base_color, depth_tint);
	color += grid_color * grid * grid_strength;

	float fresnel = pow(1.0 - clamp(dot(NORMAL, VIEW), 0.0, 1.0), 2.8);
	color += rim_color * fresnel * rim_strength;

	ALBEDO = color;
	ROUGHNESS = 0.66;
	METALLIC = 0.12;
	EMISSION = rim_color * fresnel * 0.22;
}
