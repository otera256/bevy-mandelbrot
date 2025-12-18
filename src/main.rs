use std::iter;

use bevy::{
    input::mouse::MouseWheel, prelude::*, reflect::TypePath, render::{render_resource::AsBindGroup, storage::ShaderStorageBuffer}, shader::ShaderRef, sprite_render::{Material2d, Material2dPlugin}, window::WindowResized
};
use num_bigfloat::{BigFloat, ZERO};

// シェーダーに渡すデータを保持する構造体
// AsBindGroupをderiveすることで、GPU側のバッファ構成を自動生成します。
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
struct MandelbrotMaterial {
    #[uniform(0)]
    num_iterations: u32,  // マンデルブロ集合の計算に使う反復回数

    #[uniform(0)]
    range: f32,  // 複素平面上の表示範囲の高さ (WGSL側では f32 になる)

    #[uniform(0)]
    aspect_ratio: f32,   // ウィンドウのアスペクト比 (WGSL側では f32 になる)

    #[uniform(0)]
    pixel_size: f32,   // 1ピクセルあたりの複素平面上の距離 (WGSL側では f32 になる)

    #[storage(1, read_only)]
    base_orbit: Handle<ShaderStorageBuffer>
}

// Material2dトレイトを実装して、どのシェーダーファイルを使うか指定します。
impl Material2d for MandelbrotMaterial {
    fn fragment_shader() -> ShaderRef {
        // assets/shaders/mandelbrot.wgsl を読み込む設定
        "shaders/mandelbrot.wgsl".into()
    }
}

impl MandelbrotMaterial {
    // マテリアルのパラメータを一括で更新するメソッド
    fn update_params(&mut self, params: &MandelbrotParams) {
        self.num_iterations = params.num_iterations;
        self.range = params.range;
        self.aspect_ratio = params.aspect_ratio();
        self.pixel_size = params.pixel_size();
    }
}

#[derive(Resource, Debug, Clone, Default)]
struct MandelbrotMaterialHandle(Handle<MandelbrotMaterial>);

// いちいちmaterialにアクセスしないといけないのは面倒なので、CPU側の処理をまとめるためのResourceを用意する
#[derive(Resource, Debug, Clone)]
struct MandelbrotParams {
    num_iterations: u32,
    center: [BigFloat; 2],
    range: f32,
    window_size: Vec2,
}

impl Default for MandelbrotParams {
    fn default() -> Self {
        Self {
            num_iterations: 1000,
            center: [BigFloat::from_f64(-0.5), BigFloat::from_f64(0.0)],
            range: 2.5,
            window_size: Vec2::new(800.0, 600.0),
        }
    }
}

impl MandelbrotParams {
    fn aspect_ratio(&self) -> f32 {
        self.window_size.x / self.window_size.y
    }
    fn pixel_size(&self) -> f32 {
        self.range / self.window_size.y
    }
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            Material2dPlugin::<MandelbrotMaterial>::default()
        ))
        .init_resource::<MandelbrotParams>()
        .init_resource::<MandelbrotMaterialHandle>()
        .add_systems(Startup, (setup, resize_quad_to_window).chain())
        .add_systems(PreUpdate, (
            resize_quad_to_window.run_if(on_message::<WindowResized>),
        ))
        .add_systems(Update, (
            zoom,
            drag,
            update_material,
        ))
        .run();
}

#[derive(Component)]
struct ScreenQuad;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    // 自作したマテリアルのアセットリソース
    mut materials: ResMut<Assets<MandelbrotMaterial>>,
    mut mandelbrot_material_handle: ResMut<MandelbrotMaterialHandle>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    commands.spawn(Camera2d);

    let base_orbit_buffer = buffers.add(ShaderStorageBuffer::from(vec![
        Vec2::ZERO;
        1000  // 初期値として1000要素分のゼロベクトルを用意しておく
    ]));

    let mut mandelbrot_material = MandelbrotMaterial::default();
    mandelbrot_material.base_orbit = base_orbit_buffer;

    let material_handle = materials.add(mandelbrot_material);

    mandelbrot_material_handle.0 = material_handle.clone();

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::default())),
        ScreenQuad,
        // 自作マテリアル
        MeshMaterial2d(material_handle),
        Transform::from_xyz(0.0, 0.0, -1.0),
    ));

}

// マテリアルのパラメータを時間経過で更新するシステム
fn update_material(
    params: Res<MandelbrotParams>,
    mandelblot_material_handle: Res<MandelbrotMaterialHandle>,
    // マテリアルの実体データが格納されているアセットストレージへの可変アクセス
    mut material_assets: ResMut<Assets<MandelbrotMaterial>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    // ハンドルを使って、アセットストレージから実際のマテリアルデータを取得（可変）
    let Some(material) = material_assets.get_mut(&mandelblot_material_handle.0) else {
        return;
    };
    material.update_params(&params);
    let Some(buffer) = buffers.get_mut(&material.base_orbit) else {
        return;
    };
    buffer.set_data(
        iter::once([ZERO; 2]).chain(
            (0..params.num_iterations)
                .scan([ZERO; 2], |z, _| {
                    let x = z[0] * z[0] - z[1] * z[1] + params.center[0];
                    let y = BigFloat::parse("2.0").unwrap() * z[0] * z[1] + params.center[1];
                    *z = [x, y];
                    Some(*z)
                })
            )
            .map(|[x, y]|
                Vec2::new(x.to_f32(), y.to_f32())
            )
            .collect::<Vec<_>>()
    )
}

fn resize_quad_to_window(
    // メインウィンドウの情報を取得するクエリ
    window: Query<&Window>,
    // 目印を付けたメッシュのTransformを操作するクエリ
    mut quad_query: Query<&mut Transform, With<ScreenQuad>>,
    mut mandelbrot_params: ResMut<MandelbrotParams>,
) {

    let window = window.single().expect("Windows does not exist");
    let Ok(mut transform) = quad_query.single_mut() else { return; };

    // ウィンドウの幅と高さを取得
    let width = window.width();
    let height = window.height();

    // メッシュのスケールをウィンドウサイズに設定
    // 元が1x1なので、これで幅width、高さheightのメッシュになります。
    transform.scale = Vec3::new(width, height, 1.0);

    info!("ScreenQuad resized to: {}x{}", width, height);
    mandelbrot_params.window_size = Vec2::new(width, height);
}

fn zoom(
    mut msgr_scroll: MessageReader<MouseWheel>,
    keys: Res<ButtonInput<KeyCode>>,
    window: Single<&Window>,
    mut mandelbrot_params: ResMut<MandelbrotParams>,
){
    use bevy::input::mouse::MouseScrollUnit;
    let Some(mouse_pos) = window.cursor_position() else { return; };
    let world_mouse_pos = [
        BigFloat::from_f32((mouse_pos.x / window.width() - 0.5) * (mandelbrot_params.range * mandelbrot_params.aspect_ratio())) + mandelbrot_params.center[0],
        BigFloat::from_f32((0.5 - mouse_pos.y / window.height()) * mandelbrot_params.range) + mandelbrot_params.center[1],
    ];
    let mut zoom_factor = 1.0;
    for msg in msgr_scroll.read() {
        let scroll_amount = match msg.unit {
            MouseScrollUnit::Line => msg.y * 0.1,
            MouseScrollUnit::Pixel => msg.y * 0.001,
        };
        zoom_factor *= 1.0 - scroll_amount;
    }
    if keys.pressed(KeyCode::KeyZ) {
        zoom_factor *= 0.98;
    }
    if keys.pressed(KeyCode::KeyX) {
        zoom_factor *= 1.02;
    }
    if keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight) {
        zoom_factor = zoom_factor.powf(2.5);
    }
    mandelbrot_params.range *= zoom_factor;
    let zoom_factor = BigFloat::from_f32(zoom_factor);
    mandelbrot_params.center[0] = world_mouse_pos[0] - (world_mouse_pos[0] - mandelbrot_params.center[0]) * zoom_factor;
    mandelbrot_params.center[1] = world_mouse_pos[1] - (world_mouse_pos[1] - mandelbrot_params.center[1]) * zoom_factor;
}

fn drag(
    mut msgr_cursor: MessageReader<CursorMoved>,
    window: Single<&Window>,
    butttons: Res<ButtonInput<MouseButton>>,
    mut mandelbrot_params: ResMut<MandelbrotParams>,
){
    // マウス左ボタンが押されていない場合は何もしない
    if !butttons.pressed(MouseButton::Left) {
        return;
    }
    for msg in msgr_cursor.read() {
        let Some(delta) = msg.delta else { continue; };
        let dx = BigFloat::from_f32(-(delta.x / window.width()) * (mandelbrot_params.range * mandelbrot_params.aspect_ratio()));
        let dy = BigFloat::from_f32((delta.y / window.height()) * mandelbrot_params.range);
        mandelbrot_params.center[0] += dx;
        mandelbrot_params.center[1] += dy;
    }
}