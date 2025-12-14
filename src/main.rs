use bevy::{
    prelude::*,
    reflect::TypePath,
    render::render_resource::AsBindGroup,
    shader::ShaderRef, sprite_render::{Material2d, Material2dPlugin}, window::WindowResized,
};

// シェーダーに渡すデータを保持する構造体
// AsBindGroupをderiveすることで、GPU側のバッファ構成を自動生成します。
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct MandelbrotMaterial {
    #[uniform(0)]
    offset: Vec2, // 複素平面上の中心座標 (WGSL側では vec2<f32> になる)

    #[uniform(0)]
    range: f32,  // 複素平面上の表示範囲の高さ (WGSL側では f32 になる)

    #[uniform(0)]
    ratio: f32,   // ウィンドウのアスペクト比 (WGSL側では f32 になる)
}

// Material2dトレイトを実装して、どのシェーダーファイルを使うか指定します。
impl Material2d for MandelbrotMaterial {
    fn fragment_shader() -> ShaderRef {
        // assets/shaders/mandelbrot.wgsl を読み込む設定
        "shaders/mandelbrot.wgsl".into()
    }
}

fn main() {
    println!("Hello, world!");
    App::new()
        .add_plugins((
            DefaultPlugins,
            Material2dPlugin::<MandelbrotMaterial>::default()
        ))
        .add_systems(Startup, (setup, resize_quad_to_window).chain())
        .add_systems(Update, (
            update_material,
            resize_quad_to_window.run_if(on_message::<WindowResized>)
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
        MeshMaterial2d(materials.add(MandelbrotMaterial {
            offset: Vec2::new(-0.7, 0.0), // 初期位置を少し左にずらす
            range: 2.5,
            ratio: 1.0, // 初期値（ウィンドウサイズに合わせて更新されます）
        })),
        // 位置（Zはカメラより奥であればOK。2Dなので0.0でも-1.0でも大差ありません）
        Transform::from_xyz(0.0, 0.0, -1.0),
    ));
}

// マテリアルのパラメータを時間経過で更新するシステム
fn update_material(
    time: Res<Time>,
    // 現在シーンで使われている MandelbrotMaterial のハンドルを探すクエリ
    material_handle_query: Query<&MeshMaterial2d<MandelbrotMaterial>>,
    // マテリアルの実体データが格納されているアセットストレージへの可変アクセス
    mut material_assets: ResMut<Assets<MandelbrotMaterial>>,
) {
    // シーンにマテリアルがなければ何もしない
    let Ok(material_handle) = material_handle_query.single() else { return; };

    // ハンドルを使って、アセットストレージから実際のマテリアルデータを取得（可変）
    if let Some(material) = material_assets.get_mut(material_handle) {
        // 時間経過を取得 (秒)
        // let t = time.elapsed_secs();
        // material.range = 2.0f32.powf(t * -0.1); // 少しゆっくりめに

        // ここで変更した material の内容は、Bevyが自動的に検知して
        // 次の描画フレームの前にGPUに転送してくれます。
    }
}

fn resize_quad_to_window(
    // メインウィンドウの情報を取得するクエリ
    window: Query<&Window>,
    // 目印を付けたメッシュのTransformを操作するクエリ
    mut quad_query: Query<&mut Transform, With<ScreenQuad>>,
        // 現在シーンで使われている MandelbrotMaterial のハンドルを探すクエリ
    material_handle_query: Query<&MeshMaterial2d<MandelbrotMaterial>>,
    // マテリアルの実体データが格納されているアセットストレージへの可変アクセス
    mut material_assets: ResMut<Assets<MandelbrotMaterial>>,
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

    // シーンにマテリアルがなければ何もしない
    let Ok(material_handle) = material_handle_query.single() else { return; };

    // ハンドルを使って、アセットストレージから実際のマテリアルデータを取得（可変）
    if let Some(material) = material_assets.get_mut(material_handle) {
        // ウィンドウのアスペクト比を計算
        let aspect_ratio = width / height;
        material.ratio = aspect_ratio;
    }
}