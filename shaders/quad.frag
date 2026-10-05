#version 140
in vec2 v_uv;
out vec4 out_color;

uniform vec3 cam_pos;
uniform vec3 cam_forward;
uniform vec3 cam_right;
uniform vec3 cam_up;
uniform float fov;
uniform float aspect;
uniform float disk_inner;
uniform float rs; 
uniform float disk_outer;
uniform mat4 view_proj;

float hash(vec3 p) {
    p = fract(p * 0.3183099 + 0.1);
    p *= 17.0;
    return fract(p.x * p.y * p.z * (p.x + p.y + p.z));
}

float noise(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    vec2 u = f * f * (3.0 - 2.0 * f);
    float a = hash(vec3(i.x, i.y, 0.0));
    float b = hash(vec3(i.x + 1.0, i.y, 0.0));
    float c = hash(vec3(i.x, i.y + 1.0, 0.0));
    float d = hash(vec3(i.x + 1.0, i.y + 1.0, 0.0));
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

float fbm(vec2 p) {
    float v = 0.0;
    float a = 0.5;
    mat2 rot = mat2(0.87758, 0.47942, -0.47942, 0.87758);
    for(int i = 0; i < 4; i++) {
        v += a * noise(p);
        p = rot * p * 2.0;
        a *= 0.5;
    }
    return v;
}

vec3 disk_color(float r, vec3 world_dir_from_disk) {
    float t = clamp((r - disk_inner) / (disk_outer - disk_inner), 0.0, 1.0);
    
    vec3 core_color = vec3(2.0, 1.6, 0.4);   
    vec3 mid_color  = vec3(1.5, 0.9, 0.2);   
    vec3 edge_color = vec3(0.8, 0.15, 0.2);

    vec3 base = mix(core_color, mid_color, smoothstep(0.0, 0.4, t));
    base = mix(base, edge_color, smoothstep(0.4, 1.0, t));

    float angle = atan(world_dir_from_disk.z, world_dir_from_disk.x);
    vec2 polar = vec2(r * 2.5, angle * 4.0);
    float gas_noise = fbm(polar - vec2(0.0, r * 1.5)); 
    base *= (0.5 + 1.5 * gas_noise);

    vec3 orbital_dir = normalize(vec3(-world_dir_from_disk.z, 0.0, world_dir_from_disk.x));
    float approach = dot(orbital_dir, normalize(-world_dir_from_disk));
    float beaming = pow(max(approach * 0.5 + 0.5, 0.0), 3.0) * 6.0 + 0.3;

    float f = 1.0 - rs / r;
    float redshift = sqrt(max(f, 0.0));

    float edge_fade = smoothstep(disk_inner, disk_inner + (disk_outer - disk_inner) * 0.15, r)
                    * (1.0 - smoothstep(disk_outer - (disk_outer - disk_inner) * 0.25, disk_outer, r));

    return base * beaming * redshift * edge_fade;
}

vec4 geodesic_rhs(vec4 s, float e, float rs) {
    float r = s.x;
    float dr = s.z;
    float dphi = s.w;

    float f = max(1.0 - rs / r, 0.001);
    float dt = e / f;

    float d2r = -(rs / (2.0 * r * r)) * f * dt * dt
              + (rs / (2.0 * r * r * f)) * dr * dr
              + (r - rs) * dphi * dphi;

    float d2phi = -2.0 * dr * dphi / r;

    return vec4(dr, dphi, d2r, d2phi);
}

vec4 rk4_step(vec4 s, float dl, float e, float rs) {
    vec4 k1 = geodesic_rhs(s, e, rs);
    vec4 k2 = geodesic_rhs(s + 0.5 * dl * k1, e, rs);
    vec4 k3 = geodesic_rhs(s + 0.5 * dl * k2, e, rs);
    vec4 k4 = geodesic_rhs(s + dl * k3, e, rs);

    return s + (dl / 6.0) * (k1 + 2.0 * k2 + 2.0 * k3 + k4);
}

void ray_init(vec3 cam_pos, vec3 dir, float rs,
      out vec4 state, out float e,
      out vec3 e_r, out vec3 e_perp) {
    vec3 cam_dir = normalize(cam_pos);
    vec3 cross_result = cross(cam_dir, dir);

    if (length(cross_result) < 0.01) {
        vec3 fallback = abs(dir.y) < 0.99 ? vec3(0.0, 1.0, 0.0) : vec3(1.0, 0.0, 0.0);
        cross_result = cross(cam_dir, fallback);
    }

    vec3 plane_normal = normalize(cross_result);
    e_r = cam_dir;
    e_perp = cross(plane_normal, e_r);

    float r = length(cam_pos);
    float phi = 0.0;

    float dx = dot(dir, e_r);
    float dy = dot(dir, e_perp);

    float dr = dx;
    float dphi = dy / r;

    float f0 = 1.0 - rs / r;
    float dt0 = sqrt((dr*dr)/(f0*f0) + (r*r*dphi*dphi)/f0);
    e = f0 * dt0;

    state = vec4(r, phi, dr, dphi);
}

vec3 starfield(vec3 dir) {
    vec3 p = dir * 300.0;
    vec3 cell = floor(p);
    float h = hash(cell);
    float star = step(0.995, h);
    float brightness = star * hash(cell + 7.0);
    return vec3(brightness);
}

vec3 trace(vec3 cam_pos, vec3 dir, float rs, out vec3 hit_pos, out bool is_far) {
    vec4 state;
    float e;
    vec3 e_r, e_perp;
    ray_init(cam_pos, dir, rs, state, e, e_r, e_perp);

    const int MAX_STEPS = 7200;
    const float R_MAX = 100.0;
    float r_prev = state.x;
    float phi_prev = state.y;
    float capture_radius = rs * 1.01;

    for (int i = 0; i < MAX_STEPS; i++) {
        float r = state.x;

        if (r <= capture_radius) {
            hit_pos = r * (cos(state.y) * e_r + sin(state.y) * e_perp);
            is_far = false;
            return vec3(0.0);
        }
        if (r > R_MAX) {
            is_far = true;
            vec3 world_dir = normalize(cos(state.y) * e_r + sin(state.y) * e_perp);
            return starfield(world_dir);
        }

        float dynamic_dl = mix(0.02, 0.3, clamp((r - rs) / (rs * 5.0), 0.0, 1.0));
        vec4 next = rk4_step(state, dynamic_dl, e, rs);

        if (!(next.x > capture_radius)) {
            hit_pos = state.x * (cos(state.y) * e_r + sin(state.y) * e_perp);
            is_far = false;
            return vec3(0.0);
        }

        vec3 prev_world = r_prev * (cos(phi_prev) * e_r + sin(phi_prev) * e_perp);
        vec3 curr_world = next.x * (cos(next.y) * e_r + sin(next.y) * e_perp);

        if (sign(prev_world.y) != sign(curr_world.y)) {
            float t_intersect = prev_world.y / (prev_world.y - curr_world.y);
            vec3 exact_world = mix(prev_world, curr_world, t_intersect);
            float hit_r = length(exact_world.xz);
            
            if (hit_r > disk_inner && hit_r < disk_outer) {
                hit_pos = exact_world;
                is_far = false;
                return disk_color(hit_r, exact_world);
            }
        }

        r_prev = state.x;
        phi_prev = state.y;
        state = next;
    }

    hit_pos = r_prev * (cos(phi_prev) * e_r + sin(phi_prev) * e_perp);
    is_far = false;
    return vec3(0.0);
}

void main() {
    vec2 ndc = v_uv * 2.0 - 1.0;
    float tan_half_fov = tan(fov * 0.5);

    vec3 dir = normalize(
        cam_forward
        + ndc.x * aspect * tan_half_fov * cam_right
        + ndc.y * tan_half_fov * cam_up
    );

    vec3 hit_pos;
    bool is_far;
    vec3 color = trace(cam_pos, dir, rs, hit_pos, is_far);
    out_color = vec4(color, 1.0);

    if (is_far) {
        gl_FragDepth = 1.0;
    } else {
        vec4 clip = view_proj * vec4(hit_pos, 1.0);
        gl_FragDepth = clip.z / clip.w * 0.5 + 0.5;
    }
}
