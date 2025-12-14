#import bevy_sprite::mesh2d_vertex_output::VertexOutput

// Rust側で定義したuniform (現在は空ですが、将来的にここに追加されます)
// @group(2) @binding(0) var<uniform> material: MandelbrotMaterial;

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
    let uv: vec2<f32> = in.uv;

    // 複素平面上の点を計算
    let c = vec2(
        (uv.x - 0.5) * 3.0,  // 実部: -1.5 から 1.5
        (uv.y - 0.5) * 3.0   // 虚部: -1.5 から 1.5
    );

    // マンデルブロ集合の反復回数を計算
    let iter = mandelbrot(c, vec2(0.0, 0.0));

    // 色を決定（反復回数に基づくグレースケール）
    let color_value = f32(iter) / f32(MAX_ITER);
    return vec4(color_value, color_value, color_value, 1.0);
}