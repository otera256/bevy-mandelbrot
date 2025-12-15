use bevy::{
    prelude::*,
    reflect::TypePath,
    render::render_resource::AsBindGroup,
    shader::ShaderRef, sprite_render::{Material2d, Material2dPlugin}, window::WindowResized,
};

// シェーダーに渡すデータを保持する構造体
// AsBindGroupをderiveすることで、GPU側のバッファ構成を自動生成します。
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
struct MandelbrotMaterial {
    #[uniform(0)]
    offset: Vec2, // 複素平面上の中心座標 (WGSL側では vec2<f32> になる)

    #[uniform(0)]
    range: f32,  // 複素平面上の表示範囲の高さ (WGSL側では f32 になる)

    #[uniform(0)]
    aspect_ratio: f32,   // ウィンドウのアスペクト比 (WGSL側では f32 になる)

    #[uniform(0)]
    pixel_size: f32,   // 1ピクセルあたりの複素平面上の距離 (WGSL側では f32 になる)
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
        self.offset = params.offset;
        self.range = params.range;
        self.aspect_ratio = params.aspect_ratio();
        self.pixel_size = params.pixel_size();
    }
}

// いちいちmaterialにアクセスしないといけないのは面倒なので、CPU側の処理をまとめるためのResourceを用意する
#[derive(Resource, Debug, Clone)]
struct MandelbrotParams {
    offset: Vec2,
    range: f32,
    window_size: Vec2,
}

impl Default for MandelbrotParams {
    fn default() -> Self {
        Self {
            offset: Vec2::new(-0.7, 0.0),
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
    println!("Hello, world!");
    App::new()
        .add_plugins((
            DefaultPlugins,
            Material2dPlugin::<MandelbrotMaterial>::default()
        ))
        .init_resource::<MandelbrotParams>()
        .add_systems(Startup, (setup, resize_quad_to_window).chain())
        .add_systems(PreUpdate, (
            resize_quad_to_window.run_if(on_message::<WindowResized>),
        ))
        .add_systems(Update, (
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
) {
    // 1. 2Dカメラを配置
    commands.spawn(Camera2d);

    commands.spawn((
        // メッシュ
        Mesh2d(meshes.add(Rectangle::default())),
        ScreenQuad,
        // 自作マテリアル
        MeshMaterial2d(materials.add(MandelbrotMaterial::default())),
        // 位置（Zはカメラより奥であればOK。2Dなので0.0でも-1.0でも大差ありません）
        Transform::from_xyz(0.0, 0.0, -1.0),
    ));
}

// マテリアルのパラメータを時間経過で更新するシステム
fn update_material(
    params: Res<MandelbrotParams>,
    // 現在シーンで使われている MandelbrotMaterial のハンドルを探すクエリ
    material_handle_query: Query<&MeshMaterial2d<MandelbrotMaterial>>,
    // マテリアルの実体データが格納されているアセットストレージへの可変アクセス
    mut material_assets: ResMut<Assets<MandelbrotMaterial>>,
) {
    // シーンにマテリアルがなければ何もしない
    let Ok(material_handle) = material_handle_query.single() else { return; };

    // ハンドルを使って、アセットストレージから実際のマテリアルデータを取得（可変）
    if let Some(material) = material_assets.get_mut(material_handle) {
        material.update_params(&params);
    }
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

    // 【重要】メッシュのスケールをウィンドウサイズに設定
    // 元が1x1なので、これで幅width、高さheightのメッシュになります。
    transform.scale = Vec3::new(width, height, 1.0);

    info!("ScreenQuad resized to: {}x{}", width, height);
    mandelbrot_params.window_size = Vec2::new(width, height);
}