#version 140
        in vec2 v_uv;
        out vec4 out_color;

        uniform vec3 cam_pos;
        uniform vec3 cam_forward;
        uniform vec3 cam_right;
        uniform vec3 cam_up;
        uniform float fov;
        uniform float aspect;

        vec4 geodesic_rhs(vec4 s, float e, float rs) {
            float r = s.x;
            float dr = s.z;
            float dphi = s.w;

            float f = 1.0 - rs / r;
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
            vec3 plane_normal = normalize(cross(cam_pos, dir));
            e_r = normalize(cam_pos);
            e_perp = cross(plane_normal, e_r);

            float r = length(cam_pos);
            float phi = 0.0;

            float dx = dot(dir, e_r);
            float dy = dot(dir, e_perp);

            float dr = dx;         // simplified from dir·cos(phi)+dir·sin(phi) since phi=0
            float dphi = dy / r;   // simplified likewise

            float f0 = 1.0 - rs / r;
            float dt0 = sqrt((dr*dr)/(f0*f0) + (r*r*dphi*dphi)/f0);
            e = f0 * dt0;

            state = vec4(r, phi, dr, dphi);
        }


        float hash(vec3 p) {
            p = fract(p * 0.3183099 + 0.1);
            p *= 17.0;
            return fract(p.x * p.y * p.z * (p.x + p.y + p.z));
        }

        vec3 starfield(vec3 dir) {
            vec3 p = dir * 300.0;
            vec3 cell = floor(p);
            float h = hash(cell);
            float star = step(0.995, h);
            float brightness = star * hash(cell + 7.0);
            return vec3(brightness);
        }

        vec3 trace(vec3 cam_pos, vec3 dir, float rs) {
            vec4 state;
            float e;
            vec3 e_r, e_perp;
            ray_init(cam_pos, dir, rs, state, e, e_r, e_perp);

            const int MAX_STEPS = 200;
            const float DL = 0.1;
            const float R_MAX = 100.0;

            for (int i = 0; i < MAX_STEPS; i++) {
                float r = state.x;

                if (r <= rs) {
                    return vec3(0.0); // captured — event horizon
                }
                if (r > R_MAX) {
                    float phi = state.y;
                    vec3 world_dir = normalize(cos(phi) * e_r + sin(phi) * e_perp);
                    return starfield(world_dir);
                }

                state = rk4_step(state, DL, e, rs);
            }

            // ran out of steps without resolving — treat as escaped
            float phi = state.y;
            vec3 world_dir = normalize(cos(phi) * e_r + sin(phi) * e_perp);
            return starfield(world_dir);
        }

        uniform float rs; 

        void main() {
            vec2 ndc = v_uv * 2.0 - 1.0;
            float tan_half_fov = tan(fov * 0.5);

            vec3 dir = normalize(
                cam_forward
                + ndc.x * aspect * tan_half_fov * cam_right
                + ndc.y * tan_half_fov * cam_up
            );

            out_color = vec4(trace(cam_pos, dir, rs), 1.0);
        }
