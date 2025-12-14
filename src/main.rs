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
    // ここに将来的に、色、ズーム率、オフセット位置などのパラメータを追加します。
    // 例:
    // #[uniform(0)]
    // color: LinearRgba,
    // #[uniform(1)]
    // zoom_center: Vec2,
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
        .add_systems(Update, resize_quad_to_window.run_if(on_message::<WindowResized>))
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
            // 初期パラメータがあればここで設定
        })),
        // 位置（Zはカメラより奥であればOK。2Dなので0.0でも-1.0でも大差ありません）
        Transform::from_xyz(0.0, 0.0, -1.0),
    ));
}

fn resize_quad_to_window(
    // メインウィンドウの情報を取得するクエリ
    window: Query<&Window>,
    // 目印を付けたメッシュのTransformを操作するクエリ
    mut quad_query: Query<&mut Transform, With<ScreenQuad>>,
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
}