#import bevy_sprite::mesh2d_vertex_output::VertexOutput

const PI: f32 = 3.141592653589793;

struct MaterialData {
    offset: vec2<f32>,
    range: f32,
    ratio: f32,
};

@group(2) @binding(0) var<uniform> material: MaterialData;

const MAX_ITER: u32 = 100u;

fn mandelbrot(c: vec2<f32>, z0: vec2<f32>) -> u32 {
    // マンデルブロ集合の点が発散するかどうかを判定します
    // 発散までにかかる反復回数を返します
    var z = z0;
    var i = 0u;
    for (; i < MAX_ITER; i ++) {
        if length(z) > 10000.0 {
            break;
        }
        // z <- z**2 + c
        z = vec2(z.x*z.x - z.y*z.y + c.x, 2*z.x*z.y + c.y);
    }
    return i;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // 画素の位置を取得
    // (0, 0) が左上、(1, 1) が右下のUV座標系
    let uv: vec2<f32> = in.uv;

    // UV座標を複素平面上の座標に変換
    // material.offset が中心座標、material.range が表示範囲の高さ、material.ratio がアスペクト比
    let aspect_ratio = material.ratio;
    let c = vec2(
        material.offset.x + (uv.x - 0.5) * material.range * aspect_ratio,
        material.offset.y - (uv.y - 0.5) * material.range
    );

    // --------------------

    // マンデルブロ集合の反復回数を計算
    let iter = mandelbrot(c, vec2(0.0, 0.0));

    // 色を決定（少し見やすく調整）
    if (iter == MAX_ITER) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0); // 収束したら黒
    }
    let color_value = sin((vec3f(f32(iter)/f32(MAX_ITER))*vec3f(0.5, 2.5, 3.5)+vec3f(0.5))*PI);
    // ガンマ補正っぽいことをして少し明るく見やすく
    // let bright_color = pow(color_value, 0.5);
    return vec4(color_value, 1.0);
}