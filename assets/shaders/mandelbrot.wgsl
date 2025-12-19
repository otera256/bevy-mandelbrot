#import bevy_sprite::mesh2d_vertex_output::VertexOutput

const PI: f32 = 3.141592653589793;

struct MaterialData {
    num_iterations: u32,
    range: f32,
    aspect_ratio: f32,
    pixel_size: f32,
};

struct BaseOrbitBuffer {
    data: array<vec2<f32>>,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: MaterialData;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var<storage, read> base_orbit: BaseOrbitBuffer;

const ESCAPE_RADIUS: f32 = 100.0;

fn mandelbrot(dc: vec2<f32>) -> f32 {
    // マンデルブロ集合の点が発散するかどうかを判定します
    // 発散までにかかる反復回数から脱出時の速度を考慮して補正された値を返します
    // 摂動法を使用して高精度計算を行います
    var dz = vec2<f32>(0.0);
    var i = 0u;
    var ref_i = 0u;
    for (; i < material.num_iterations; i ++) {
        // Z_i
        var base_z = base_orbit.data[ref_i];
        let z = base_z + dz;
        let radius2 = dot(z, z);
        if radius2 > ESCAPE_RADIUS * ESCAPE_RADIUS {
            let log_zn = log2(radius2) / 2.0;
            return f32(i) + 1.0 - log2(log_zn);
        }
        // Rebasing
        let dradius2 = dot(dz, dz);
        let baseradius2 = dot(base_z, base_z);
        if dradius2 > baseradius2 || baseradius2 > ESCAPE_RADIUS * ESCAPE_RADIUS * 0.25 {
            dz = z;
            ref_i = 0u;
            base_z = vec2(0.0, 0.0);
        }

        // dz_(i+1) = 2 * Z_i * dz_i + (dz_i)^2 + dc
        dz = 2.0 * vec2(
            base_z.x * dz.x - base_z.y * dz.y,
            base_z.x * dz.y + base_z.y * dz.x
        ) + vec2(
            dz.x * dz.x - dz.y * dz.y,
            2.0 * dz.x * dz.y
        ) + dc;
        ref_i = ref_i + 1u;
    }
    return f32(material.num_iterations);
}

fn get_color(dc: vec2<f32>, range: f32) -> vec4<f32> {
    let iter = mandelbrot(dc);
    if iter == f32(material.num_iterations) {
        return vec4(0.0, 0.0, 0.0, 1.0); // 集合に属する点は黒
    }

    let s = iter / f32(material.num_iterations);
    let v = pow(s, log2(range) * -0.1 + 0.5);
    // HSV Coloring
    // let hue = 3600.0 * v % 360.0;
    // let saturation = 0.8;
    // let value = (cos(PI * v) * 0.5 + 0.5) * 0.8 + 0.2;
    // return vec4(hsv2rgb(vec3(hue, saturation, value)), 1.0);

    // Cyclic Cosine Coloring
    let alpha = log2(s * 7.0 + 1.0);
    return vec4(
        0.5 + 0.5 * cos(PI *  (2.0 * alpha - vec3(1.0, 0.75, 0.5))),
        1.0
    );
}

// HSVからRGBへの変換
// hsv.x: Hue (0-360), hsv.y: Saturation (0-1), hsv.z: Value (0-1)
fn hsv2rgb(hsv: vec3<f32>) -> vec3<f32> {
    let hue_sector = hsv.x / 60.0;
    let c = hsv.z * hsv.y;
    let x = c * (1.0 - abs(hue_sector % 2.0 - 1.0));
    let m = hsv.z - c;
    var rgb = vec3<f32>(0.0);
    if hue_sector < 1.0 {
        rgb = vec3(c, x, 0.0);
    } else if hue_sector < 2.0 {
        rgb = vec3(x, c, 0.0);
    } else if hue_sector < 3.0 {
        rgb = vec3(0.0, c, x);
    } else if hue_sector < 4.0 {
        rgb = vec3(0.0, x, c);
    } else if hue_sector < 5.0 {
        rgb = vec3(x, 0.0, c);
    } else {
        rgb = vec3(c, 0.0, x);
    }
    return rgb + vec3(m);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // 画素の位置を取得
    // (0, 0) が左上、(1, 1) が右下のUV座標系
    let uv: vec2<f32> = in.uv;

    // UV座標を複素平面上の座標に変換
    // material.offset が中心座標、material.range が表示範囲の高さ、material.ratio がアスペクト比
    let aspect_ratio = material.aspect_ratio;
    let dc0 = vec2(
        (uv.x - 0.5) * material.range * aspect_ratio,
        -(uv.y - 0.5) * material.range
    );

    // アンチエイリアスのために4サンプルを取得して平均化
    let pixel_size = material.pixel_size;
    let samples = array<vec2<f32>, 4>(
        vec2(-0.25 * pixel_size, -0.25 * pixel_size),
        vec2( 0.25 * pixel_size, -0.25 * pixel_size),
        vec2(-0.25 * pixel_size,  0.25 * pixel_size),
        vec2( 0.25 * pixel_size,  0.25 * pixel_size),
    );
    var color = vec4(0.0);
    for (var i = 0u; i < 4u; i = i + 1u) {
        let dc = dc0 + samples[i];
        color = color + get_color(dc, material.range);
    }
    color = color / 4.0;
    return color;
}
