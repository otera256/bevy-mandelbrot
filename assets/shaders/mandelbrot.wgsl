#import bevy_sprite::mesh2d_vertex_output::VertexOutput

const PI: f32 = 3.141592653589793;

struct MaterialData {
    offset: vec2<f32>,
    range: f32,
    aspect_ratio: f32,
    pixel_size: f32,
};

@group(2) @binding(0) var<uniform> material: MaterialData;

const MAX_ITER: u32 = 1000u;
const ESCAPE_RADIUS: f32 = 100.0;

fn mandelbrot(c: vec2<f32>, z0: vec2<f32>) -> f32 {
    // マンデルブロ集合の点が発散するかどうかを判定します
    // 発散までにかかる反復回数から脱出時の速度を考慮して補正された値を返します
    var z = z0;
    var i = 0u;
    for (; i < MAX_ITER; i ++) {
        // z <- z**2 + c
        z = vec2(z.x*z.x - z.y*z.y, 2*z.x*z.y) + c;
        let z_len2 = dot(z, z);
        if z_len2 > ESCAPE_RADIUS * ESCAPE_RADIUS {
            let log_zn = log2(z_len2) / 2.0;
            return f32(i) + 1.0 - log2(log_zn);
        }
    }
    return f32(MAX_ITER);
}

fn get_color(c: vec2<f32>, z0: vec2<f32>) -> vec4<f32> {
    let iter = mandelbrot(c, z0);
    if iter == f32(MAX_ITER) {
        return vec4(0.0, 0.0, 0.0, 1.0); // 集合に属する点は黒
    }

    let alpha = log2(iter / f32(MAX_ITER) + 1.0);
    let shift = 0.0;
    
    return vec4(
        sin((6.0 * alpha - 0.2 + shift) * PI),
        sin((6.0 * alpha + 0.0 + shift) * PI),
        sin((6.0 * alpha + 0.2 + shift) * PI),
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
    let c0 = vec2(
        material.offset.x + (uv.x - 0.5) * material.range * aspect_ratio,
        material.offset.y - (uv.y - 0.5) * material.range
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
        let c = c0 + samples[i];
        color = color + get_color(c, vec2(0.0, 0.0));
    }
    color = color / 4.0;
    return color;
}