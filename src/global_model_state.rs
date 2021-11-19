///モデルの回転,平行移動,倍率を表す
///
/// scale -> rotate -> translate の順番で計算する
struct ModelTransform{
    scale:f32,
    translate:[f32;3],
    rotate:[f32;3],
}
///モデルのプラットフォーム固有のテクスチャ情報
struct PMXTexture{

}
///
struct PMXPart{
   indices:egui_wgpu_backend::wgpu::Buffer,

}
///パーツ固有の情報を表す
/// これらはユニフォームバッファもしくはpush constantで渡す
struct PMXPartUniform{
    ///拡散光
    diffuse:[f32;4],
    ///鏡面光
    specular:[f32;3],
    ///鏡面係数
    specular_factor:f32,
    ///環境光
    ambient_color:[f32;3],
}

///モデルについての情報を表す
struct Model{
    ///モデル名
    name:String,
    ///ワールド変換情報
    transform:ModelTransform,
    ///テクスチャ
    textures:Vec<PMXTexture>,
    ///パーツ
    parts:Vec<PMXPart>,
}