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

    let alpha = pow(iter / f32(material.num_iterations) , -log2(range) * 0.15 + 0.6);
    
    return vec4(
        sin((2.0 * alpha + 0.6) * PI),
        sin((2.0 * alpha + 0.0) * PI),
        sin((2.0 * alpha - 0.6) * PI),
        1.0
    );
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
